use serde::de::DeserializeOwned;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{
    plugin::PluginApi, AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder,
};

use crate::error::Error;
use crate::models::{WidgetConfig, WidgetWindowConfig};
use crate::store::{
    self, config_key, parse_pending_actions, touch_meta, DataMap, PENDING_ACTIONS_KEY,
};

#[cfg(target_os = "macos")]
use std::ffi::{CStr, CString};

/// Protocol name registered by the plugin for the built-in widget renderer.
pub(crate) const BUILTIN_PROTOCOL: &str = "widgetview";

fn builtin_widget_url(group: &str, size: &str, widget_id: &str) -> WebviewUrl {
    #[cfg(target_os = "windows")]
    let url_str = format!(
        "https://{}.localhost/?group={}&size={}&widgetId={}",
        BUILTIN_PROTOCOL, group, size, widget_id
    );
    #[cfg(not(target_os = "windows"))]
    let url_str = format!(
        "{}://localhost/?group={}&size={}&widgetId={}",
        BUILTIN_PROTOCOL, group, size, widget_id
    );
    WebviewUrl::External(url_str.parse().expect("invalid built-in widget URL"))
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Widget<R>> {
    Ok(Widget {
        app: app.clone(),
        last_config_hash: Mutex::new(HashMap::new()),
        store: Mutex::new(HashMap::new()),
        known_groups: Mutex::new(Vec::new()),
        #[cfg(target_os = "macos")]
        poller_started: Mutex::new(false),
    })
}

pub struct Widget<R: Runtime> {
    app: AppHandle<R>,
    /// Content hash per (group, widget_id).
    last_config_hash: Mutex<HashMap<(String, String), u64>>,
    /// In-memory data store keyed by group.
    store: Mutex<HashMap<String, DataMap>>,
    known_groups: Mutex<Vec<String>>,
    #[cfg(target_os = "macos")]
    poller_started: Mutex<bool>,
}

impl<R: Runtime> Widget<R> {
    fn remember_group(&self, group: &str) {
        let mut groups = self.known_groups.lock().unwrap();
        if !groups.iter().any(|g| g == group) {
            groups.push(group.to_string());
        }
    }

    fn storage_path(&self, group: &str) -> crate::Result<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            if let Some(path) = macos_shared_container(group) {
                let dir = path;
                if !dir.exists() {
                    fs::create_dir_all(&dir)?;
                }
                return Ok(dir.join("widget_data.json"));
            }
        }

        let base = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| Error::Io(e.to_string()))?;
        let dir = base.join("widgets");
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        let safe: String = group
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '.' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        Ok(dir.join(format!("{safe}.json")))
    }

    fn load_map_locked<'a>(
        store: &'a mut HashMap<String, DataMap>,
        path: &PathBuf,
        group: &str,
    ) -> &'a mut DataMap {
        store.entry(group.to_string()).or_insert_with(|| {
            if path.exists() {
                fs::read_to_string(path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default()
            } else {
                DataMap::new()
            }
        })
    }

    /// Persist map to local path + macOS multi-transport fan-out.
    fn persist_map(&self, group: &str, map: &DataMap) -> crate::Result<()> {
        let path = self.storage_path(group)?;
        let json = serde_json::to_string_pretty(map)?;
        atomic_write(&path, json.as_bytes())?;

        #[cfg(target_os = "macos")]
        {
            macos_fanout_write(group, map, &json);
        }

        let _ = self.app.emit("widget-update", group);
        Ok(())
    }

    pub fn set_items(&self, key: &str, value: &str, group: &str) -> crate::Result<bool> {
        self.remember_group(group);
        let path = self.storage_path(group)?;
        let mut store = self.store.lock().unwrap();
        let map = Self::load_map_locked(&mut store, &path, group);
        map.insert(key.into(), value.into());
        touch_meta(map);
        let snapshot = map.clone();
        drop(store);
        self.persist_map(group, &snapshot)?;
        Ok(true)
    }

    pub fn get_items(&self, key: &str, group: &str) -> crate::Result<Option<String>> {
        let path = self.storage_path(group)?;
        let mut store = self.store.lock().unwrap();
        let map = Self::load_map_locked(&mut store, &path, group);
        Ok(map.get(key).cloned())
    }

    pub fn create_widget_window(&self, config: WidgetWindowConfig) -> crate::Result<bool> {
        let app = self.app.clone();
        let label_log = config.label.clone();

        self.app
            .run_on_main_thread(move || {
                let url = match config.url.as_deref() {
                    Some(u) if !u.is_empty() => WebviewUrl::App(u.into()),
                    _ => {
                        let group = config.group.as_deref().unwrap_or("default");
                        let size = config.size.as_deref().unwrap_or("small");
                        let widget_id = config.widget_id.as_deref().unwrap_or("default");
                        builtin_widget_url(group, size, widget_id)
                    }
                };
                let mut builder = WebviewWindowBuilder::new(&app, &config.label, url)
                    .title("")
                    .inner_size(config.width, config.height)
                    .decorations(false)
                    .skip_taskbar(config.skip_taskbar)
                    .always_on_top(config.always_on_top)
                    .resizable(false)
                    .visible(true);
                // Transparent windows on macOS require the host app's `macos-private-api`.
                #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
                {
                    builder = builder.transparent(true);
                }

                if let (Some(x), Some(y)) = (config.x, config.y) {
                    builder = builder.position(x, y);
                }

                if let Err(e) = builder.build() {
                    log::error!("create_widget_window '{}': {}", config.label, e);
                }
            })
            .map_err(|e| Error::new(format!("main thread dispatch: {e}")))?;

        log::debug!("created widget window '{label_log}'");
        Ok(true)
    }

    pub fn close_widget_window(&self, label: &str) -> crate::Result<bool> {
        if let Some(win) = self.app.get_webview_window(label) {
            win.close().map_err(|e| Error::new(e.to_string()))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn set_register_widget(&self, _widgets: Vec<String>) -> crate::Result<bool> {
        // Desktop has no native provider registry; accept for API symmetry.
        Ok(true)
    }

    pub fn reload_all_timelines(&self) -> crate::Result<bool> {
        #[cfg(target_os = "macos")]
        {
            let _ = unsafe { macos_widget_reload_all() };
        }
        let _ = self.app.emit("widget-reload", "all");
        Ok(true)
    }

    pub fn reload_timelines(&self, of_kind: &str) -> crate::Result<bool> {
        #[cfg(target_os = "macos")]
        {
            let c = CString::new(of_kind).unwrap_or_default();
            let _ = unsafe { macos_widget_reload_kind(c.as_ptr()) };
        }
        let _ = self.app.emit("widget-reload", of_kind);
        Ok(true)
    }

    pub fn request_widget(&self) -> crate::Result<bool> {
        Err(Error::Unsupported(
            "Use create_widget_window on desktop".into(),
        ))
    }

    pub fn set_widget_config(
        &self,
        config: &WidgetConfig,
        group: &str,
        widget_id: &str,
        skip_reload: bool,
    ) -> crate::Result<bool> {
        if widget_id.is_empty() {
            return Err(Error::new("widget_id must not be empty"));
        }
        self.remember_group(group);

        let json = serde_json::to_string(config)
            .map_err(|e| Error::new(format!("serialize config: {e}")))?;
        let compact: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| Error::new(format!("serialize config: {e}")))?;

        let mut hasher = DefaultHasher::new();
        json.hash(&mut hasher);
        let new_hash = hasher.finish();
        let hash_key = (group.to_string(), widget_id.to_string());

        let changed = {
            let mut prev = self.last_config_hash.lock().unwrap();
            if prev.get(&hash_key) == Some(&new_hash) {
                false
            } else {
                prev.insert(hash_key, new_hash);
                true
            }
        };

        if changed {
            crate::capabilities::log_capabilities(config);
            let key = config_key(widget_id);
            self.set_items(&key, &json, group)?;
        }

        let _ = self.app.emit(
            "widget-config-push",
            serde_json::json!({
                "group": group,
                "widgetId": widget_id,
                "config": compact,
            }),
        );

        if changed && !skip_reload {
            self.reload_all_timelines()?;
        }

        #[cfg(target_os = "macos")]
        self.ensure_action_poller();

        Ok(true)
    }

    pub fn get_widget_config(
        &self,
        group: &str,
        widget_id: &str,
    ) -> crate::Result<Option<WidgetConfig>> {
        if widget_id.is_empty() {
            return Err(Error::new("widget_id must not be empty"));
        }
        let raw = self.get_items(&config_key(widget_id), group)?;
        match raw {
            Some(json) => {
                let config: WidgetConfig = serde_json::from_str(&json)
                    .map_err(|e| Error::new(format!("parse config: {e}")))?;
                Ok(Some(config))
            }
            None => Ok(None),
        }
    }

    /// Drain pending actions for a group (CAS clear under store lock).
    pub fn poll_pending_actions(&self, group: &str) -> crate::Result<Vec<serde_json::Value>> {
        self.remember_group(group);

        #[cfg(target_os = "macos")]
        let disk_maps = macos_read_all_transports(group);
        #[cfg(not(target_os = "macos"))]
        let disk_maps: Vec<DataMap> = {
            let path = self.storage_path(group)?;
            if path.exists() {
                fs::read_to_string(&path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .into_iter()
                    .collect()
            } else {
                Vec::new()
            }
        };

        let freshest = store::pick_freshest(disk_maps);

        let mut store = self.store.lock().unwrap();
        let path = self.storage_path(group)?;
        let map = Self::load_map_locked(&mut store, &path, group);

        // Merge freshest disk state if newer than in-memory.
        if store::map_nonce(&freshest) > store::map_nonce(map) {
            *map = freshest;
        }

        let actions = parse_pending_actions(map.get(PENDING_ACTIONS_KEY).map(|s| s.as_str()));
        if actions.is_empty() {
            return Ok(Vec::new());
        }

        map.insert(PENDING_ACTIONS_KEY.into(), "[]".into());
        touch_meta(map);
        let snapshot = map.clone();
        drop(store);

        self.persist_map(group, &snapshot)?;

        Ok(actions
            .into_iter()
            .filter_map(|a| serde_json::to_value(a).ok())
            .collect())
    }

    #[cfg(target_os = "macos")]
    fn ensure_action_poller(&self) {
        let mut started = self.poller_started.lock().unwrap();
        if *started {
            return;
        }
        *started = true;

        let app_handle = self.app.clone();
        // Poller walks known_groups via a shared approach: we clone app and
        // re-read groups from a side channel — use the Widget state through
        // periodic emit after reading all known groups from disk registry file.
        // Simpler: poll by listing groups from in-memory via weak pattern —
        // spawn thread that emits for each group file found is hard without
        // Widget handle. Instead keep groups list by writing a registry.
        let groups_handle = app_handle.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                // Best-effort: ask managed state if available
                let Some(widget) = groups_handle.try_state::<Widget<R>>() else {
                    continue;
                };
                let groups = widget.inner().known_groups.lock().unwrap().clone();
                for group in groups {
                    match widget.inner().poll_pending_actions(&group) {
                        Ok(actions) if !actions.is_empty() => {
                            for action in actions {
                                let _ = groups_handle.emit("widget-action", action);
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}

// ─── macOS helpers ────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
extern "C" {
    fn macos_widget_reload_all() -> bool;
    fn macos_widget_reload_kind(kind: *const std::ffi::c_char) -> bool;
    fn macos_widget_container_path(group: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn macos_widget_free_string(ptr: *mut std::ffi::c_char);
    fn macos_widget_set_defaults(
        group: *const std::ffi::c_char,
        key: *const std::ffi::c_char,
        value: *const std::ffi::c_char,
    ) -> bool;
}

#[cfg(target_os = "macos")]
fn macos_shared_container(group: &str) -> Option<PathBuf> {
    let c_group = CString::new(group).ok()?;
    let ptr = unsafe { macos_widget_container_path(c_group.as_ptr()) };
    if ptr.is_null() {
        return None;
    }
    let path = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { macos_widget_free_string(ptr) };
    Some(PathBuf::from(path))
}

/// Fan-out write to App Group file (already written by caller path when available),
/// UserDefaults suite, and WidgetSandboxFile.
#[cfg(target_os = "macos")]
fn macos_fanout_write(group: &str, map: &DataMap, json: &str) {
    // UserDefaults: write each key (incl. meta) so suite mirrors file.
    for (k, v) in map {
        if let (Ok(c_group), Ok(c_key), Ok(c_value)) =
            (CString::new(group), CString::new(k.as_str()), CString::new(v.as_str()))
        {
            unsafe {
                macos_widget_set_defaults(c_group.as_ptr(), c_key.as_ptr(), c_value.as_ptr());
            }
        }
    }

    // WidgetSandboxFile — required for ad-hoc signing when App Groups don't share.
    macos_write_to_widget_container(group, json);
}

#[cfg(target_os = "macos")]
fn macos_write_to_widget_container(group: &str, json: &str) {
    let app_id = group.strip_prefix("group.").unwrap_or(group);
    let widget_bundle = format!("{app_id}.widgetkit");
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    let container = PathBuf::from(&home)
        .join("Library/Containers")
        .join(&widget_bundle)
        .join("Data");
    let _ = fs::create_dir_all(&container);
    let path = container.join("widget_data.json");
    let _ = atomic_write(&path, json.as_bytes());
}

#[cfg(target_os = "macos")]
fn macos_read_all_transports(group: &str) -> Vec<DataMap> {
    let mut maps = Vec::new();

    // App Group file
    if let Some(dir) = macos_shared_container(group) {
        let path = dir.join("widget_data.json");
        if let Ok(s) = fs::read_to_string(&path) {
            if let Ok(m) = serde_json::from_str::<DataMap>(&s) {
                maps.push(m);
            }
        }
    }

    // WidgetSandboxFile
    let app_id = group.strip_prefix("group.").unwrap_or(group);
    let widget_bundle = format!("{app_id}.widgetkit");
    if let Ok(home) = std::env::var("HOME") {
        let path = PathBuf::from(home)
            .join("Library/Containers")
            .join(&widget_bundle)
            .join("Data")
            .join("widget_data.json");
        if let Ok(s) = fs::read_to_string(&path) {
            if let Ok(m) = serde_json::from_str::<DataMap>(&s) {
                maps.push(m);
            }
        }
    }

    // UserDefaults is written key-by-key; reconstructing full map from FFI is
    // not exposed — file transports cover poll. Nonce pick still works across files.
    maps
}

fn atomic_write(path: &PathBuf, data: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
