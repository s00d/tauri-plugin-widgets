use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{
    plugin::PluginApi, AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder,
};

use crate::apply::{config_content_hash, ApplyOutcome, ReloadOutcome};
use crate::config::WidgetsPluginConfig;
use crate::error::Error;
use crate::models::{WidgetConfig, WidgetWindowConfig};
use crate::receipt::{receipts_path, ReceiptStore, WidgetRenderReceipt};
use crate::store::{
    self, config_key, parse_pending_actions, DataMap, PENDING_ACTIONS_KEY,
};
use crate::trace::{trace_path, TraceEvent, TraceSkipReason, TraceStore, WidgetTrace};

#[cfg(target_os = "macos")]
use crate::transport::Transport;
#[cfg(target_os = "macos")]
use std::ffi::CString;

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

pub fn init<R: Runtime>(
    app: &AppHandle<R>,
    api: PluginApi<R, Option<WidgetsPluginConfig>>,
) -> crate::Result<Widget<R>> {
    let cfg = api.config().clone().unwrap_or_default();
    init_with_config(app, cfg)
}

pub(crate) fn init_with_config<R: Runtime>(
    app: &AppHandle<R>,
    cfg: WidgetsPluginConfig,
) -> crate::Result<Widget<R>> {
    let receipts = ReceiptStore::new();
    let trace = TraceStore::new();
    if let Ok(dir) = app.path().app_data_dir() {
        receipts.load_from_path(&receipts_path(&dir));
        trace.load_from_path(&trace_path(&dir));
    }

    #[cfg(target_os = "macos")]
    let macos_driver = crate::transport::resolve_driver(&cfg)?;

    let widget = Widget {
        app: app.clone(),
        cfg,
        store: Mutex::new(HashMap::new()),
        known_groups: Mutex::new(Vec::new()),
        receipts,
        trace,
        #[cfg(target_os = "macos")]
        poller_started: Mutex::new(false),
        #[cfg(target_os = "macos")]
        macos_driver,
    };
    // Seed poller with configured App Group so widget taps work before first set_widget_config.
    if let Some(g) = widget.cfg.app_group.clone() {
        widget.remember_group(&g);
    }
    #[cfg(target_os = "macos")]
    widget.ensure_action_poller();

    Ok(widget)
}

pub struct Widget<R: Runtime> {
    app: AppHandle<R>,
    #[allow(dead_code)]
    cfg: WidgetsPluginConfig,
    /// In-memory data store keyed by group.
    store: Mutex<HashMap<String, DataMap>>,
    known_groups: Mutex<Vec<String>>,
    /// Cross-platform render receipts (diagnostics only).
    receipts: ReceiptStore,
    /// Host delivery journal (debug / `WIDGET_DEBUG=1`).
    trace: TraceStore,
    #[cfg(target_os = "macos")]
    poller_started: Mutex<bool>,
    /// Single Apple host transport (config-chosen).
    #[cfg(target_os = "macos")]
    macos_driver: Arc<dyn Transport>,
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
            if let Some(path) = crate::macos_transport::app_group_data_override() {
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                return Ok(path);
            }
            if let Some(dir) = macos_shared_container(group) {
                if !dir.exists() {
                    fs::create_dir_all(&dir)?;
                }
                return Ok(dir.join("widget_data.json"));
            }
            Ok(crate::macos_transport::sandbox_widget_data_path(group))
        }

        #[cfg(target_os = "windows")]
        {
            // Align with WidgetProvider Store.DefaultPath so Widgets Board sees host writes.
            if let Ok(env_path) = std::env::var("TAURI_WIDGETS_DATA") {
                let p = env_path.trim();
                if !p.is_empty() {
                    let path = if p.ends_with(".json") {
                        PathBuf::from(p)
                    } else {
                        PathBuf::from(p).join("widget_data.json")
                    };
                    if let Some(parent) = path.parent() {
                        if !parent.exists() {
                            fs::create_dir_all(parent)?;
                        }
                    }
                    return Ok(path);
                }
            }
            let local = std::env::var("LOCALAPPDATA")
                .map(PathBuf::from)
                .or_else(|_| {
                    self.app
                        .path()
                        .app_data_dir()
                        .map_err(|e| Error::Io(e.to_string()))
                })?;
            let dir = local.join("tauri-plugin-widgets");
            if !dir.exists() {
                fs::create_dir_all(&dir)?;
            }
            let _ = group; // single shared widget_data.json — group lives in map keys
            Ok(dir.join("widget_data.json"))
        }

        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        {
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
    }

    fn load_map_locked<'a>(
        store: &'a mut HashMap<String, DataMap>,
        path: &PathBuf,
        group: &str,
    ) -> &'a mut DataMap {
        store.entry(group.to_string()).or_insert_with(|| {
            let mut maps = Vec::new();
            if path.exists() {
                if let Some(m) = fs::read_to_string(path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                {
                    maps.push(m);
                }
            }
            #[cfg(target_os = "macos")]
            {
                for t in crate::macos_transport::all_transports(group) {
                    if let Some(m) = t.read() {
                        maps.push(m);
                    }
                }
            }
            store::pick_freshest(maps)
        })
    }

    /// Persist map: one Apple driver on macOS / single file elsewhere.
    fn persist_map(&self, group: &str, map: &DataMap) -> crate::Result<()> {
        #[cfg(target_os = "macos")]
        {
            self.macos_driver.write(map)?;
            // Widget still picks freshest across all transports — keep siblings in sync
            // so a stale UserDefaults/App Group snapshot cannot outrank this write.
            crate::macos_transport::mirror_to_siblings(self.macos_driver.as_ref(), group, map);
        }
        #[cfg(target_os = "windows")]
        {
            let path = self.storage_path(group)?;
            persist_windows_shared_map(&path, map)?;
        }
        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        {
            let path = self.storage_path(group)?;
            let json = serde_json::to_string_pretty(map)?;
            atomic_write(&path, json.as_bytes())?;
        }

        let _ = self.app.emit("widget-update", group);
        Ok(())
    }

    pub fn set_items(&self, key: &str, value: &str, group: &str) -> crate::Result<bool> {
        self.remember_group(group);
        let path = self.storage_path(group)?;
        let mut store = self.store.lock().unwrap();
        let map = Self::load_map_locked(&mut store, &path, group);
        if map.get(key).map(String::as_str) == Some(value) {
            return Ok(true);
        }
        map.insert(key.into(), value.into());
        #[cfg(target_os = "macos")]
        {
            let floor = crate::macos_transport::max_nonce_across(group);
            store::touch_meta_above(map, floor);
        }
        #[cfg(not(target_os = "macos"))]
        {
            store::touch_meta(map);
        }
        let snapshot = map.clone();
        drop(store);
        self.persist_map(group, &snapshot)?;
        Ok(true)
    }

    pub fn get_items(&self, key: &str, group: &str) -> crate::Result<Option<String>> {
        #[cfg(target_os = "macos")]
        {
            let freshest = self.macos_driver_map(group)?;
            Ok(freshest.get(key).cloned())
        }
        #[cfg(not(target_os = "macos"))]
        {
            let path = self.storage_path(group)?;
            let mut store = self.store.lock().unwrap();
            let map = Self::load_map_locked(&mut store, &path, group);
            Ok(map.get(key).cloned())
        }
    }

    pub fn create_widget_window(&self, config: WidgetWindowConfig) -> crate::Result<bool> {
        let app = self.app.clone();
        // Already on the GTK/UI thread (e.g. `setup`) — build inline to avoid deadlock
        // waiting for a scheduled task that cannot run until we return.
        #[cfg(all(target_os = "linux", feature = "linux"))]
        {
            if gtk::glib::MainContext::default().is_owner() {
                return Self::create_widget_window_on_main(&app, config);
            }
        }

        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        self.app
            .run_on_main_thread(move || {
                let _ = tx.send(Self::create_widget_window_on_main(&app, config));
            })
            .map_err(|e| Error::new(format!("main thread dispatch: {e}")))?;

        match rx.recv_timeout(std::time::Duration::from_secs(8)) {
            Ok(result) => result,
            // Setup on non-Linux (or before the loop pumps): task is queued.
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Ok(true),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                Err(Error::new("create_widget_window: main thread dropped"))
            }
        }
    }

    fn create_widget_window_on_main(
        app: &AppHandle<R>,
        config: WidgetWindowConfig,
    ) -> crate::Result<bool> {
        let label_log = config.label.clone();
        let url = match config.url.as_deref() {
            Some(u) if !u.is_empty() => WebviewUrl::App(u.into()),
            _ => {
                let group = config.group.as_deref().unwrap_or("default");
                let size = config.size.as_deref().unwrap_or("small");
                let widget_id = config.widget_id.as_deref().unwrap_or("default");
                builtin_widget_url(group, size, widget_id)
            }
        };
        let skip_taskbar = config.skip_taskbar;
        // Close any prior window with this label so rebuilds (watch/inbox) succeed.
        if let Some(prev) = app.get_webview_window(&config.label) {
            let _ = prev.close();
        }
        let mut builder = WebviewWindowBuilder::new(app, &config.label, url)
            // Label doubles as WM_NAME so harnesses can find the window (xdotool).
            .title(&config.label)
            .inner_size(config.width, config.height)
            .decorations(false)
            .skip_taskbar(skip_taskbar)
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

        let win = builder
            .build()
            .map_err(|e| Error::new(format!("create_widget_window '{}': {e}", config.label)))?;
        #[cfg(all(target_os = "linux", feature = "linux"))]
        crate::linux::pin_widget_window(&win, skip_taskbar);
        #[cfg(not(all(target_os = "linux", feature = "linux")))]
        let _ = win;
        log::debug!("created widget window '{label_log}'");
        Ok(true)
    }

    pub fn close_widget_window(&self, label: &str) -> crate::Result<bool> {
        let app = self.app.clone();
        let label = label.to_string();

        #[cfg(all(target_os = "linux", feature = "linux"))]
        {
            if gtk::glib::MainContext::default().is_owner() {
                return Self::close_widget_window_on_main(&app, &label);
            }
        }

        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        self.app
            .run_on_main_thread(move || {
                let _ = tx.send(Self::close_widget_window_on_main(&app, &label));
            })
            .map_err(|e| Error::new(format!("main thread dispatch: {e}")))?;

        match rx.recv_timeout(std::time::Duration::from_secs(3)) {
            Ok(result) => result,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Ok(true),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                Err(Error::new("close_widget_window: main thread dropped"))
            }
        }
    }

    fn close_widget_window_on_main(app: &AppHandle<R>, label: &str) -> crate::Result<bool> {
        if let Some(win) = app.get_webview_window(label) {
            win.close().map_err(|e| Error::new(e.to_string()))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Register native widget provider ids.
    ///
    /// | Platform | Behaviour |
    /// |---|---|
    /// | Android | stores fully-qualified provider class names |
    /// | iOS / macOS | stores WidgetKit kind strings (advisory) |
    /// | Desktop | **no-op**, accepted for API symmetry |
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

    /// Request that the OS show the "add widget" / pin UI.
    ///
    /// | Platform | Behaviour |
    /// |---|---|
    /// | Android | opens the pin-widget flow |
    /// | iOS / macOS | no native pin API — returns `Ok` from the mobile bridge |
    /// | Desktop | **error** — use [`Self::create_widget_window`] instead |
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
    ) -> crate::Result<ApplyOutcome> {
        if widget_id.is_empty() {
            return Err(Error::new("widget_id must not be empty"));
        }
        self.remember_group(group);

        let json = serde_json::to_string(config)
            .map_err(|e| Error::new(format!("serialize config: {e}")))?;
        let compact: serde_json::Value = serde_json::from_str(&json)
            .map_err(|e| Error::new(format!("serialize config: {e}")))?;
        let hash = config_content_hash(&json);
        let key = config_key(widget_id);

        let existing = self.get_items(&key, group)?;
        let changed = existing.as_deref() != Some(json.as_str());

        // Desktop webview always gets a push so an open window stays in sync
        // even when store bytes were already identical.
        let _ = self.app.emit(
            "widget-config-push",
            serde_json::json!({
                "group": group,
                "widgetId": widget_id,
                "config": compact,
            }),
        );

        if !changed {
            let outcome = ApplyOutcome::unchanged(hash);
            self.trace.push(TraceEvent::ConfigSet {
                widget_id: widget_id.into(),
                nonce: 0,
                bytes: json.len(),
                changed: false,
                skip: Some(TraceSkipReason::Unchanged { hash }),
            });
            self.trace.push(TraceEvent::Reload {
                performed: false,
                reason: outcome.reload.clone(),
            });
            self.maybe_flush_trace();
            return Ok(outcome);
        }

        crate::capabilities::log_capabilities(config);
        let t0 = std::time::Instant::now();
        self.set_items(&key, &json, group)?;
        let write_ms = t0.elapsed().as_millis() as u32;
        let transports = self.written_transport_names(group);
        let nonce = self
            .get_items("__meta_nonce__", group)
            .ok()
            .flatten()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        self.trace.push(TraceEvent::ConfigSet {
            widget_id: widget_id.into(),
            nonce,
            bytes: json.len(),
            changed: true,
            skip: None,
        });
        for name in &transports {
            self.trace.push(TraceEvent::Write {
                transport: name.clone(),
                ok: true,
                duration_ms: write_ms,
                error: None,
            });
        }

        #[cfg(target_os = "windows")]
        {
            // Widgets Board provider reads Adaptive Card blobs from the same store.
            // Desktop webview (widget.html) remains the fallback outside Widget Board.
            if let Some(result) =
                crate::adaptive_card::to_adaptive_card_for_size(config, "medium")
            {
                let template = serde_json::to_string(&result.card)
                    .map_err(|e| Error::new(format!("serialize adaptive card: {e}")))?;
                self.set_items(
                    &crate::adaptive_card::ac_template_key(widget_id),
                    &template,
                    group,
                )?;
                self.set_items(&crate::adaptive_card::ac_data_key(widget_id), "{}", group)?;
            }
        }

        let reload = if skip_reload {
            ReloadOutcome::Skipped {
                why: "skip_reload".into(),
            }
        } else {
            match self.reload_all_timelines() {
                Ok(_) => ReloadOutcome::Ok,
                Err(e) => ReloadOutcome::Failed {
                    error: e.to_string(),
                },
            }
        };
        self.trace.push(TraceEvent::Reload {
            performed: matches!(reload, ReloadOutcome::Ok),
            reason: reload.clone(),
        });

        #[cfg(target_os = "macos")]
        self.ensure_action_poller();

        self.maybe_flush_trace();

        Ok(ApplyOutcome {
            written: true,
            reload,
            transports,
            skip: None,
        })
    }

    /// Names of transports that hold the current map after a write.
    fn written_transport_names(&self, group: &str) -> Vec<String> {
        #[cfg(target_os = "macos")]
        {
            let mut names = vec![self.macos_driver.name().to_string()];
            for t in crate::macos_transport::all_transports(group) {
                if t.name() != self.macos_driver.name() && t.available() {
                    names.push(t.name().to_string());
                }
            }
            names
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = group;
            vec!["file".into()]
        }
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

    /// Read configured driver + merge with in-memory if newer.
    #[cfg(target_os = "macos")]
    fn macos_driver_map(&self, group: &str) -> crate::Result<DataMap> {
        let mut maps = Vec::new();
        if let Some(disk) = self.macos_driver.read() {
            maps.push(disk);
        }
        let path = self.storage_path(group)?;
        let mut store = self.store.lock().unwrap();
        let map = Self::load_map_locked(&mut store, &path, group);
        maps.push(map.clone());
        let freshest = store::pick_freshest(maps);
        if store::map_nonce(&freshest) > store::map_nonce(map) {
            *map = freshest.clone();
        }
        Ok(freshest)
    }

    /// Drain pending actions for a group (CAS clear under store lock).
    pub fn poll_pending_actions(
        &self,
        group: &str,
    ) -> crate::Result<Vec<crate::WidgetActionEnvelope>> {
        self.remember_group(group);

        #[cfg(target_os = "macos")]
        let disk_maps: Vec<DataMap> = {
            let mut maps = Vec::new();
            for t in crate::macos_transport::all_transports(group) {
                if let Some(m) = t.read() {
                    maps.push(m);
                }
            }
            maps
        };
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

        if store::map_nonce(&freshest) > store::map_nonce(map) {
            *map = freshest;
        }

        let actions = parse_pending_actions(map.get(PENDING_ACTIONS_KEY).map(|s| s.as_str()));
        if actions.is_empty() {
            return Ok(Vec::new());
        }

        map.insert(PENDING_ACTIONS_KEY.into(), "[]".into());
        #[cfg(target_os = "macos")]
        {
            let floor = crate::macos_transport::max_nonce_across(group);
            store::touch_meta_above(map, floor);
        }
        #[cfg(not(target_os = "macos"))]
        {
            store::touch_meta(map);
        }
        let snapshot = map.clone();
        drop(store);

        self.persist_map(group, &snapshot)?;

        Ok(actions)
    }

    pub fn report_receipt(&self, receipt: WidgetRenderReceipt) -> crate::Result<bool> {
        self.remember_group(&receipt.group);
        let trigger = receipt
            .trigger
            .clone()
            .unwrap_or_else(|| "timeline".into());
        let lag_ms = {
            let since = self.trace.list_since(None);
            since
                .iter()
                .rev()
                .find_map(|e| match &e.event {
                    TraceEvent::ConfigSet { nonce, .. } if *nonce == receipt.nonce && *nonce > 0 => {
                        Some(receipt.ts.saturating_sub(e.ts))
                    }
                    _ => None,
                })
                .unwrap_or(0)
        };
        self.trace.push(TraceEvent::Render {
            instance: receipt.instance.clone(),
            nonce: receipt.nonce,
            source: receipt.source.clone(),
            trigger,
            lag_ms,
            skipped: receipt.skipped.clone(),
        });
        self.receipts.upsert(receipt);
        if let Ok(dir) = self.app.path().app_data_dir() {
            let _ = self.receipts.save_to_path(&receipts_path(&dir));
        }
        self.maybe_flush_trace();
        Ok(true)
    }

    pub fn get_widget_diagnostics(&self, group: &str) -> crate::Result<Vec<WidgetRenderReceipt>> {
        Ok(self.receipts.list(group))
    }

    pub fn get_widget_trace(
        &self,
        group: &str,
        since_ms: Option<u64>,
    ) -> crate::Result<WidgetTrace> {
        self.maybe_flush_trace();
        Ok(WidgetTrace {
            enabled: crate::trace::trace_enabled(),
            events: self.trace.list_since(since_ms),
            receipts: self.receipts.history(group),
        })
    }

    pub fn flush_widget_trace(&self) -> crate::Result<bool> {
        if let Ok(dir) = self.app.path().app_data_dir() {
            self.trace.flush_to_path(&trace_path(&dir))?;
        }
        Ok(true)
    }

    fn maybe_flush_trace(&self) {
        if !self.trace.needs_timed_flush() {
            return;
        }
        if let Ok(dir) = self.app.path().app_data_dir() {
            let _ = self.trace.flush_to_path(&trace_path(&dir));
        }
    }

    #[cfg(target_os = "macos")]
    fn ensure_action_poller(&self) {
        let mut started = self.poller_started.lock().unwrap();
        if *started {
            return;
        }
        *started = true;

        let groups_handle = self.app.clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let Some(widget) = groups_handle.try_state::<Widget<R>>() else {
                    continue;
                };
                widget.inner().maybe_flush_trace();
                let groups = widget.inner().known_groups.lock().unwrap().clone();
                for group in groups {
                    match widget.inner().poll_pending_actions(&group) {
                        Ok(actions) if !actions.is_empty() => {
                            widget.inner().trace.push(TraceEvent::Poll {
                                count: actions.len(),
                            });
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
}

#[cfg(target_os = "macos")]
fn macos_shared_container(group: &str) -> Option<PathBuf> {
    use std::ffi::CStr;
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

#[cfg(not(target_os = "macos"))]
fn atomic_write(path: &PathBuf, data: &[u8]) -> std::io::Result<()> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = path.with_extension(format!("tmp.{}.{}", std::process::id(), nanos));
    fs::write(&tmp, data)?;
    match fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            Err(e)
        }
    }
}

/// Windows Widgets Board provider and Rust host share one `widget_data.json`.
/// Match `WidgetStore.PersistUnlocked`: exclusive `.lock` + merge under it so
/// provider-enqueued `pending_actions` are not wiped by a concurrent host write.
#[cfg(target_os = "windows")]
fn persist_windows_shared_map(path: &PathBuf, map: &DataMap) -> crate::Result<()> {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;
    use std::thread;
    use std::time::Duration;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::Io(e.to_string()))?;
    }

    // Same path convention as C#: `{widget_data.json}.lock`
    let lock_path = PathBuf::from(format!("{}.lock", path.display()));
    let _lock = {
        let mut last_err = None;
        let mut held = None;
        for _ in 0..100 {
            let mut opts = OpenOptions::new();
            opts.read(true).write(true).create(true).share_mode(0); // FILE_SHARE_NONE
            match opts.open(&lock_path) {
                Ok(f) => {
                    held = Some(f);
                    break;
                }
                Err(e) => {
                    // ERROR_SHARING_VIOLATION (32) while the provider holds the lock.
                    if e.raw_os_error() == Some(32) {
                        last_err = Some(e);
                        thread::sleep(Duration::from_millis(20));
                        continue;
                    }
                    return Err(Error::Io(e.to_string()));
                }
            }
        }
        held.ok_or_else(|| {
            Error::Io(
                last_err
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "widget_data.json.lock busy".into()),
            )
        })?
    };

    let mut merged: DataMap = if path.exists() {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        DataMap::new()
    };

    for (k, v) in map {
        if k == PENDING_ACTIONS_KEY {
            let host_empty = v.trim().is_empty() || v.trim() == "[]";
            let disk_empty = merged
                .get(k)
                .map(|s| s.trim().is_empty() || s.trim() == "[]")
                .unwrap_or(true);
            if host_empty && !disk_empty {
                // Provider enqueued actions after our in-memory snapshot was taken.
                continue;
            }
        }
        merged.insert(k.clone(), v.clone());
    }

    // Keep host meta (already bumped) authoritative for this write.
    if let Some(n) = map.get(store::META_NONCE_KEY) {
        merged.insert(store::META_NONCE_KEY.into(), n.clone());
    }
    if let Some(t) = map.get(store::META_UPDATED_AT_KEY) {
        merged.insert(store::META_UPDATED_AT_KEY.into(), t.clone());
    }

    let json = serde_json::to_string_pretty(&merged).map_err(|e| Error::new(e.to_string()))?;
    atomic_write(path, json.as_bytes()).map_err(|e| Error::Io(e.to_string()))?;
    Ok(())
}
