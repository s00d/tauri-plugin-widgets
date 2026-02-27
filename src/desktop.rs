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

#[cfg(target_os = "macos")]
use std::ffi::{CStr, CString};

type DataMap = HashMap<String, String>;

// ─── macOS: FFI (compiled from macos/WidgetReload.swift) ────

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

/// Protocol name registered by the plugin for the built-in widget renderer.
pub(crate) const BUILTIN_PROTOCOL: &str = "widgetview";

fn builtin_widget_url(group: &str, size: &str) -> WebviewUrl {
    #[cfg(target_os = "windows")]
    let url_str = format!(
        "https://{}.localhost/?group={}&size={}",
        BUILTIN_PROTOCOL, group, size
    );
    #[cfg(not(target_os = "windows"))]
    let url_str = format!(
        "{}://localhost/?group={}&size={}",
        BUILTIN_PROTOCOL, group, size
    );
    WebviewUrl::External(url_str.parse().expect("invalid built-in widget URL"))
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Widget<R>> {
    Ok(Widget {
        app: app.clone(),
        last_config_hash: Mutex::new(0),
        store: Mutex::new(HashMap::new()),
        #[cfg(target_os = "macos")]
        poller_started: Mutex::new(false),
    })
}

pub struct Widget<R: Runtime> {
    app: AppHandle<R>,
    last_config_hash: Mutex<u64>,
    /// In-memory data store keyed by group. Eliminates race conditions on
    /// concurrent writes — the file is updated atomically under the lock.
    store: Mutex<HashMap<String, DataMap>>,
    #[cfg(target_os = "macos")]
    poller_started: Mutex<bool>,
}

impl<R: Runtime> Widget<R> {
    // ── Storage path ─────────────────────────────────────────────────────

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
            .map(|c| if c.is_alphanumeric() || c == '.' { c } else { '_' })
            .collect();
        Ok(dir.join(format!("{safe}.json")))
    }

    // ── Storage ──────────────────────────────────────────────────────────

    pub fn set_items(&self, key: &str, value: &str, group: &str) -> crate::Result<bool> {
        let path = self.storage_path(group)?;
        let mut store = self.store.lock().unwrap();
        let map = store.entry(group.to_string()).or_insert_with(|| {
            if path.exists() {
                fs::read_to_string(&path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default()
            } else {
                DataMap::new()
            }
        });
        map.insert(key.into(), value.into());
        let json = serde_json::to_string_pretty(map)?;
        drop(store);
        atomic_write(&path, json.as_bytes())?;

        #[cfg(target_os = "macos")]
        {
            if let (Ok(c_group), Ok(c_key), Ok(c_value)) = (
                CString::new(group),
                CString::new(key),
                CString::new(value),
            ) {
                unsafe {
                    macos_widget_set_defaults(
                        c_group.as_ptr(),
                        c_key.as_ptr(),
                        c_value.as_ptr(),
                    );
                }
            }
            macos_write_to_widget_container(group, &json);
        }

        let _ = self.app.emit("widget-update", group);
        Ok(true)
    }

    pub fn get_items(&self, key: &str, group: &str) -> crate::Result<Option<String>> {
        let path = self.storage_path(group)?;
        let mut store = self.store.lock().unwrap();
        let map = store.entry(group.to_string()).or_insert_with(|| {
            if path.exists() {
                fs::read_to_string(&path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default()
            } else {
                DataMap::new()
            }
        });
        Ok(map.get(key).cloned())
    }

    // ── Widget windows ──────────────────────────────────────────────────

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
                        builtin_widget_url(group, size)
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

    // ── Stubs / reload ──────────────────────────────────────────────────

    pub fn set_register_widget(&self, _widgets: Vec<String>) -> crate::Result<bool> {
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

    // ── Widget config ─────────────────────────────────────────────────────

    pub fn set_widget_config(
        &self,
        config: &WidgetConfig,
        group: &str,
        skip_reload: bool,
    ) -> crate::Result<bool> {
        let val = serde_json::to_value(config)
            .map_err(|e| Error::new(format!("serialize config: {e}")))?;
        let compact = strip_nulls(val);
        let json = serde_json::to_string(&compact)
            .map_err(|e| Error::new(format!("serialize config: {e}")))?;

        let mut hasher = DefaultHasher::new();
        json.hash(&mut hasher);
        let new_hash = hasher.finish();

        let changed = {
            let mut prev = self.last_config_hash.lock().unwrap();
            if *prev == new_hash {
                false
            } else {
                *prev = new_hash;
                true
            }
        };

        if changed {
            self.set_items("__widget_config__", &json, group)?;
        }

        let _ = self.app.emit("widget-config-push", &compact);

        if changed && !skip_reload {
            self.reload_all_timelines()?;
        }

        #[cfg(target_os = "macos")]
        self.ensure_action_poller(group);

        Ok(true)
    }

    pub fn get_widget_config(&self, group: &str) -> crate::Result<Option<WidgetConfig>> {
        let raw = self.get_items("__widget_config__", group)?;
        match raw {
            Some(json) => {
                let config: WidgetConfig = serde_json::from_str(&json)
                    .map_err(|e| Error::new(format!("parse config: {e}")))?;
                Ok(Some(config))
            }
            None => Ok(None),
        }
    }

    pub fn poll_pending_actions(&self, _group: &str) -> crate::Result<Vec<serde_json::Value>> {
        Ok(Vec::new())
    }

    // ── macOS action poller ──────────────────────────────────────────────

    #[cfg(target_os = "macos")]
    fn ensure_action_poller(&self, group: &str) {
        let mut started = self.poller_started.lock().unwrap();
        if *started {
            return;
        }
        *started = true;

        let app_handle = self.app.clone();
        let group = group.to_string();

        std::thread::spawn(move || {
            let data_path = match macos_shared_container(&group) {
                Some(p) => p.join("widget_data.json"),
                None => return,
            };
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));

                let map: HashMap<String, String> = match fs::read_to_string(&data_path) {
                    Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
                    Err(_) => continue,
                };

                let Some(actions_str) = map.get("__widget_pending_actions__") else {
                    continue;
                };

                let actions: Vec<String> = match serde_json::from_str(actions_str) {
                    Ok(a) => a,
                    Err(_) => continue,
                };

                if actions.is_empty() {
                    continue;
                }

                for action in &actions {
                    let data = serde_json::json!({ "action": action });
                    let _ = app_handle.emit("widget-action", data);
                }

                let mut cleared = map.clone();
                cleared.insert("__widget_pending_actions__".into(), "[]".into());
                if let Ok(json) = serde_json::to_string_pretty(&cleared) {
                    let _ = atomic_write(&data_path, json.as_bytes());
                }
            }
        });
    }
}

// ─── macOS helpers ────────────────────────────────────────────────────────────

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

/// Write config file directly into the widget extension's sandbox container.
/// The non-sandboxed main app can write here; the sandboxed widget reads
/// from NSHomeDirectory() which maps to the same path.
#[cfg(target_os = "macos")]
fn macos_write_to_widget_container(group: &str, json: &str) {
    let app_id = group.strip_prefix("group.").unwrap_or(group);
    let widget_id = format!("{app_id}.widgetkit");
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    let container = PathBuf::from(&home)
        .join("Library/Containers")
        .join(&widget_id)
        .join("Data");
    let _ = fs::create_dir_all(&container);
    let path = container.join("widget_data.json");
    let _ = fs::write(&path, json.as_bytes());
}

/// Write data atomically: write to a temp file, then rename.
fn atomic_write(path: &PathBuf, data: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, data)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn strip_nulls(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(m) => {
            let cleaned: serde_json::Map<String, serde_json::Value> = m
                .into_iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k, strip_nulls(v)))
                .collect();
            serde_json::Value::Object(cleaned)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(strip_nulls).collect())
        }
        other => other,
    }
}
