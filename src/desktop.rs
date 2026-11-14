use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{
    plugin::PluginApi, AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder,
};

use crate::apply::{config_content_hash, ApplyOutcome, ReloadOutcome, SkipReason};
use crate::config::WidgetsPluginConfig;
use crate::error::Error;
use crate::models::{WidgetConfig, WidgetWindowConfig};
use crate::receipt::{receipts_path, ReceiptStore, WidgetRenderReceipt};
use crate::store::{self, config_key, parse_pending_actions, DataMap, PENDING_ACTIONS_KEY};
use crate::trace::{trace_path, TraceEvent, TraceStore, WidgetTrace};

#[cfg(target_os = "macos")]
use crate::transport::Transport;
#[cfg(target_os = "macos")]
use std::ffi::CString;

pub(crate) mod island;
mod notify;
pub(crate) mod placement;
mod sources;
mod window;

/// Protocol name registered by the plugin for the built-in widget renderer.
pub(crate) const BUILTIN_PROTOCOL: &str = "widgetview";

/// Percent-encode a query component (RFC 3986 unreserved left as-is).
fn encode_query_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char);
            }
            _ => {
                use std::fmt::Write as _;
                let _ = write!(out, "%{b:02X}");
            }
        }
    }
    out
}

fn builtin_widget_url(group: &str, size: &str, widget_id: &str, instance: &str) -> WebviewUrl {
    let group = encode_query_component(group);
    let size = encode_query_component(size);
    let widget_id = encode_query_component(widget_id);
    let instance = encode_query_component(instance);
    #[cfg(target_os = "windows")]
    let url_str = format!(
        "https://{}.localhost/?group={}&size={}&widgetId={}&instance={}",
        BUILTIN_PROTOCOL, group, size, widget_id, instance
    );
    #[cfg(not(target_os = "windows"))]
    let url_str = format!(
        "{}://localhost/?group={}&size={}&widgetId={}&instance={}",
        BUILTIN_PROTOCOL, group, size, widget_id, instance
    );
    WebviewUrl::External(url_str.parse().expect("invalid built-in widget URL"))
}

/// Device language for the Widgets Board card: `LC_ALL` / `LC_MESSAGES` / `LANG` when set
/// (developer override), else the Windows user locale (`GetUserDefaultLocaleName`, BCP-47).
#[cfg(target_os = "windows")]
fn windows_locale() -> String {
    if let Some(v) = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|k| std::env::var(k).ok().filter(|v| !v.is_empty()))
    {
        return v.split('.').next().unwrap_or_default().to_string();
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetUserDefaultLocaleName(name: *mut u16, len: i32) -> i32;
    }
    let mut buf = [0u16; 85]; // LOCALE_NAME_MAX_LENGTH
                              // SAFETY: the buffer is valid for `len` UTF-16 units; the call writes at most that.
    let n = unsafe { GetUserDefaultLocaleName(buf.as_mut_ptr(), buf.len() as i32) };
    if n > 1 {
        String::from_utf16_lossy(&buf[..(n - 1) as usize])
    } else {
        String::new()
    }
}

/// GET/POST a source with the shared limits; `None` on any failure or non-JSON body.
#[cfg(all(target_os = "windows", feature = "image-prefetch"))]
fn fetch_source_text(req: &crate::resolve::SourceRequest) -> Option<String> {
    use std::io::Read;
    let mut r = ureq::request(&req.method, &req.url).timeout(std::time::Duration::from_secs(10));
    for (k, v) in &req.headers {
        r = r.set(k, v);
    }
    let resp = match &req.body {
        Some(b) => r.send_string(b),
        None => r.call(),
    }
    .ok()?;
    let mut buf = Vec::new();
    resp.into_reader()
        .take(crate::resolve::MAX_SOURCE_BYTES as u64 + 1)
        .read_to_end(&mut buf)
        .ok()?;
    if buf.len() > crate::resolve::MAX_SOURCE_BYTES {
        return None;
    }
    let text = String::from_utf8(buf).ok()?;
    serde_json::from_str::<serde_json::Value>(&text).ok()?;
    Some(text)
}

/// Desktop live-activity window size (logical px).
const ACTIVITY_WINDOW_SIZE: (f64, f64) = (340.0, 128.0);

/// Window label for an activity id. `.` is not allowed in Tauri labels; `_` is doubled
/// first so `a.b` (`a_db`) and `a_b` (`a__b`) never share a window.
/// Logical top-left of the `n`-th activity window: stacked down the work area's right edge.
fn activity_slot(area_x: f64, area_y: f64, area_w: f64, n: usize) -> (f64, f64) {
    const MARGIN: f64 = 16.0;
    let (w, h) = ACTIVITY_WINDOW_SIZE;
    (
        area_x + area_w - w - MARGIN,
        area_y + MARGIN + n as f64 * (h + 8.0),
    )
}

fn activity_window_label(id: &str) -> String {
    format!("activity-{}", id.replace('_', "__").replace('.', "_d"))
}

/// The process main thread runs the event loop on every desktop platform.
fn on_main_thread() -> bool {
    std::thread::current().name() == Some("main")
}

pub fn init<R: Runtime>(
    app: &AppHandle<R>,
    api: PluginApi<R, Option<WidgetsPluginConfig>>,
) -> crate::Result<Widget<R>> {
    let cfg = api
        .config()
        .clone()
        .unwrap_or_default()
        .with_defaults(&app.config().identifier);
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

    // An unsigned / ad-hoc dev build has no App Group container: fall back to the widget
    // extension's own container instead of aborting the whole app at startup.
    #[cfg(target_os = "macos")]
    let macos_driver = match crate::transport::resolve_driver(&cfg) {
        Ok(driver) => driver,
        Err(e) => {
            log::warn!(
                "widgets: {e}\nFalling back to transport=widgetContainer (works for ad-hoc signed \
                 builds). Sign with a Team ID and App Groups for transport=appGroup."
            );
            crate::transport::build_driver(crate::config::TransportKind::WidgetContainer, &cfg)?
        }
    };

    let widget = Widget {
        app: app.clone(),
        cfg,
        store: Mutex::new(HashMap::new()),
        known_groups: Mutex::new(Vec::new()),
        receipts,
        trace,
        windows: Mutex::new(HashMap::new()),
        activities: Mutex::new(HashMap::new()),
        activity_ts: Mutex::new(HashMap::new()),
        island: Mutex::new(island::Island::default()),
        listener_gate: store::ListenerGate::new(
            store::now_ms(),
            store::ListenerGate::DEFAULT_FALLBACK_MS,
        ),
        #[cfg(target_os = "windows")]
        board_timers: Mutex::new(HashMap::new()),
        #[cfg(target_os = "windows")]
        board_fetches: Mutex::new(HashMap::new()),
        #[cfg(target_os = "macos")]
        poller_started: Mutex::new(false),
        #[cfg(target_os = "macos")]
        persist: Arc::new(PersistQueue::default()),
        #[cfg(target_os = "macos")]
        sibling_sweeps: Mutex::new(HashMap::new()),
        #[cfg(target_os = "macos")]
        macos_driver,
        window_store: placement::WindowStore::load(
            app.path()
                .app_data_dir()
                .ok()
                .map(|d| d.join("widget-windows.json")),
        ),
        notifier: notify::Notifier::default(),
        sources: Mutex::new(HashMap::new()),
    };
    // Seed poller with configured App Group so widget taps work before first set_widget_config.
    if let Some(g) = widget.cfg.app_group.clone() {
        widget.remember_group(&g);
    }
    #[cfg(target_os = "macos")]
    widget.ensure_action_poller();

    // Queued: runs once the plugin state is managed and the event loop turns.
    if widget.cfg.restore_windows_on_start {
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            if let Some(w) = handle.try_state::<Widget<R>>() {
                w.restore_windows();
            }
        });
    }

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
    /// Widget windows this plugin opened: label → placement.
    windows: Mutex<HashMap<String, crate::store::ActiveWidget>>,
    /// Running live activities: id → group.
    activities: Mutex<HashMap<String, String>>,
    /// Last applied `timestamp` per running activity (out-of-order guard, as iOS / Android).
    activity_ts: Mutex<HashMap<String, u64>>,
    /// Activities on the top-centre island window (`activitySurface: "island"`).
    island: Mutex<island::Island>,
    /// Hold queued taps until JavaScript listens (see [`store::ListenerGate`]).
    #[allow(dead_code)]
    listener_gate: store::ListenerGate,
    /// Widgets Board rebuild timers: widget id → generation of the live one.
    #[cfg(target_os = "windows")]
    board_timers: Mutex<HashMap<String, u64>>,
    /// Widgets Board source fetches: widget id → last attempt (Unix ms).
    #[cfg(target_os = "windows")]
    board_fetches: Mutex<HashMap<String, u64>>,
    #[cfg(target_os = "macos")]
    poller_started: Mutex<bool>,
    /// Host writes not yet in the App Group files (see [`PersistQueue`]).
    #[cfg(target_os = "macos")]
    persist: Arc<PersistQueue>,
    /// Last sweep of the sibling transports per group (see [`Self::sweep_due`]).
    #[cfg(target_os = "macos")]
    sibling_sweeps: Mutex<HashMap<String, std::time::Instant>>,
    /// Single Apple host transport (config-chosen).
    #[cfg(target_os = "macos")]
    macos_driver: Arc<dyn Transport>,
    /// Remembered window positions + open window set (`widget-windows.json`).
    window_store: placement::WindowStore,
    /// Live activities shown as notifications.
    notifier: notify::Notifier,
    /// Poll / listen data sources: widget id → generation of the live one.
    sources: Mutex<HashMap<String, u64>>,
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
            for key in ["TAURI_WIDGETS_DATA", "WIDGET_DATA_DIR"] {
                if let Ok(env_path) = std::env::var(key) {
                    let p = env_path.trim();
                    if !p.is_empty() {
                        let path = if p.to_ascii_lowercase().ends_with(".json") {
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
            // validate_group already limits the charset; keep '-' so `a-b` and `a_b` differ.
            let safe: String = group
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '.' || c == '-' {
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
        #[cfg(not(target_os = "macos"))]
        let _ = group; // macOS also reads the group's transports below
                       // Keyed by file, not group: groups that share a file (Windows: one widget_data.json
                       // for Widgets Board) must share the map, or one group's cached copy overwrites the
                       // other's writes and misses them when comparing.
        store
            .entry(path.to_string_lossy().into_owned())
            .or_insert_with(|| {
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

    /// The cached map is the host's last write; a higher nonce on disk means the widget
    /// extension wrote since (a `set` tap's `state:` key). Take that map before changing
    /// it, or the host's next write would drop the extension's keys.
    #[cfg(target_os = "macos")]
    fn adopt_newer_disk_map(&self, map: &mut DataMap, group: &str) {
        let disk = store::pick_freshest(
            self.hot_transports(group)
                .iter()
                .filter_map(|t| t.read())
                .collect::<Vec<DataMap>>(),
        );
        if store::map_nonce(&disk) > store::map_nonce(map) {
            *map = disk;
        }
    }

    /// Persist map on macOS: merge pending from all channels, clear leftover
    /// maps on **sibling** transports only, then write the configured driver.
    ///
    /// The primary is never wiped before a successful write (failed write must
    /// not erase the last good map). Sibling clear re-merges pending on each
    /// wipe attempt so a mid-clear tap is not dropped.
    fn persist_map(&self, group: &str, map: &DataMap) -> crate::Result<()> {
        #[cfg(target_os = "macos")]
        crate::macos_transport::with_store_lock(group, || {
            let mut out = map.clone();
            if self.sweep_due(group) {
                crate::macos_transport::merge_pending_into_map(&mut out, group);
                crate::macos_transport::clear_sibling_transports(
                    group,
                    self.macos_driver.name(),
                    &mut out,
                );
            } else {
                crate::macos_transport::merge_pending_in(&mut out, &self.hot_transports(group));
            }
            self.macos_driver.write(&out)
        })?;
        #[cfg(target_os = "windows")]
        {
            let path = self.storage_path(group)?;
            persist_windows_shared_map(&path, map, &[])?;
        }
        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        {
            let path = self.storage_path(group)?;
            let json = serde_json::to_string_pretty(map)?;
            atomic_write(&path, json.as_bytes())?;
        }
        Ok(())
    }

    /// Transports a hot path (every write, every tap poll) reads: the working channel, chosen
    /// once at startup, for its App Group; every transport for another group.
    #[cfg(target_os = "macos")]
    fn hot_transports(&self, group: &str) -> Vec<Arc<dyn Transport>> {
        if crate::group::default_group() == Some(group) {
            vec![self.macos_driver.clone()]
        } else {
            crate::macos_transport::all_transports(group)
        }
    }

    /// Whether the sibling transports of `group` are due a sweep (fold in their taps, then
    /// empty them). The extension fans each tap out to every transport, so siblings collect
    /// copies the host never reads; sweeping them at most every 30 s keeps them small without
    /// touching every channel at every write / poll.
    #[cfg(target_os = "macos")]
    fn sweep_due(&self, group: &str) -> bool {
        const EVERY: std::time::Duration = std::time::Duration::from_secs(30);
        let mut last = self.sibling_sweeps.lock().unwrap();
        let now = std::time::Instant::now();
        if last
            .get(group)
            .is_some_and(|t| now.duration_since(*t) < EVERY)
        {
            return false;
        }
        last.insert(group.to_string(), now);
        true
    }

    pub fn set_items(&self, key: &str, value: &str, group: &str) -> crate::Result<bool> {
        self.remember_group(group);
        // macOS: memory now, files from the writer thread (`PersistQueue`): any file
        // operation in a Group Container can stall for 5 s, which must not block a
        // command (an island frame, a live widget). The writer merges the extension's
        // newer writes under the lock before it writes.
        #[cfg(target_os = "macos")]
        {
            let path = self.storage_path(group)?;
            let mut store = self.store.lock().unwrap();
            let map = Self::load_map_locked(&mut store, &path, group);
            self.adopt_newer_disk_map(map, group);
            // What the host last wrote: a queued value, else the map.
            let current = self
                .persist
                .unwritten(group)
                .into_iter()
                .rev()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v)
                .or_else(|| map.get(key).cloned());
            if current.as_deref() == Some(value) {
                return Ok(true);
            }
            map.insert(key.into(), value.into());
            store::touch_meta_above(
                map,
                crate::macos_transport::max_nonce_in(&self.hot_transports(group)),
            );
            drop(store);
            self.queue_persist(group, Some((key, value)), false);
            let _ = self.app.emit("widget-update", group);
            Ok(true)
        }
        #[cfg(not(target_os = "macos"))]
        self.set_items_locked(key, value, group)
    }

    /// Hand a host write (and / or a WidgetKit reload) to the writer thread.
    #[cfg(target_os = "macos")]
    fn queue_persist(&self, group: &str, kv: Option<(&str, &str)>, reload: bool) {
        self.persist.push(group, kv, reload);
        if !std::mem::replace(&mut *self.persist.started.lock().unwrap(), true) {
            let app = self.app.clone();
            let queue = self.persist.clone();
            std::thread::spawn(move || loop {
                let (group, job) = queue.next();
                let Some(widget) = app.try_state::<Widget<R>>() else {
                    continue;
                };
                widget.inner().write_job(&group, job);
            });
        }
    }

    /// Write every queued host key of `group` to the App Group files, then reload.
    #[cfg(target_os = "macos")]
    fn write_job(&self, group: &str, job: PersistJob) {
        if !job.keys.is_empty() {
            let res = crate::macos_transport::with_store_lock(group, || {
                let path = self.storage_path(group)?;
                let mut store = self.store.lock().unwrap();
                let map = Self::load_map_locked(&mut store, &path, group);
                // The extension may have written since (state after a tap): take its map
                // and put our keys back on top.
                self.adopt_newer_disk_map(map, group);
                // Read under the lock: a drain may have superseded a key since (see `forget`).
                for (k, v) in self.persist.in_flight(group) {
                    map.insert(k, v);
                }
                store::touch_meta_above(
                    map,
                    crate::macos_transport::max_nonce_in(&self.hot_transports(group)),
                );
                let snapshot = map.clone();
                drop(store);
                self.persist_map(group, &snapshot)
            });
            if let Err(e) = res {
                log::warn!("widget store write for '{group}' failed: {e}");
            }
        }
        self.persist.done(group);
        if job.reload {
            let _ = self.reload_all_timelines_now();
        }
    }

    /// Wait until every queued write is in the files (tests, app exit, explicit reloads).
    #[cfg(target_os = "macos")]
    pub fn flush_store(&self) {
        self.persist.wait_idle();
    }

    #[cfg_attr(target_os = "macos", allow(dead_code))]
    fn set_items_locked(&self, key: &str, value: &str, group: &str) -> crate::Result<bool> {
        let path = self.storage_path(group)?;
        let mut store = self.store.lock().unwrap();
        let map = Self::load_map_locked(&mut store, &path, group);
        #[cfg(target_os = "macos")]
        self.adopt_newer_disk_map(map, group);
        if map.get(key).map(String::as_str) == Some(value) {
            return Ok(true);
        }
        map.insert(key.into(), value.into());
        #[cfg(target_os = "macos")]
        {
            let floor = crate::macos_transport::max_nonce_in(&self.hot_transports(group));
            store::touch_meta_above(map, floor);
        }
        #[cfg(not(target_os = "macos"))]
        {
            store::touch_meta(map);
        }
        let snapshot = map.clone();
        drop(store);
        self.persist_map(group, &snapshot)?;
        let _ = self.app.emit("widget-update", group);
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
        if config.url.as_deref().map_or(true, str::is_empty) {
            self.windows.lock().unwrap().insert(
                config.label.clone(),
                crate::store::ActiveWidget {
                    widget_id: config.widget_id.clone().unwrap_or_else(|| "default".into()),
                    instance: config
                        .instance
                        .clone()
                        .or_else(|| Some(config.label.clone())),
                    size: Some(config.size.clone().unwrap_or_else(|| "small".into())),
                    last_seen_ms: None,
                },
            );
        }
        let app = self.app.clone();
        // Already on the GTK/UI thread (e.g. `setup`) — build inline to avoid deadlock
        // waiting for a scheduled task that cannot run until we return.
        #[cfg(all(target_os = "linux", feature = "linux"))]
        {
            if gtk::glib::MainContext::default().is_owner() {
                return Self::create_widget_window_on_main(&app, config);
            }
        }

        // Called from the main thread (e.g. `setup`): the queued task cannot run until we
        // return, so waiting would only stall. Report the build error in the log.
        if on_main_thread() {
            self.app
                .run_on_main_thread(move || {
                    if let Err(e) = Self::create_widget_window_on_main(&app, config) {
                        log::error!("{e}");
                    }
                })
                .map_err(|e| Error::new(format!("main thread dispatch: {e}")))?;
            return Ok(true);
        }

        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        self.app
            .run_on_main_thread(move || {
                let _ = tx.send(Self::create_widget_window_on_main(&app, config));
            })
            .map_err(|e| Error::new(format!("main thread dispatch: {e}")))?;
        rx.recv()
            .map_err(|_| Error::new("create_widget_window: main thread dropped"))?
    }

    fn create_widget_window_on_main(
        app: &AppHandle<R>,
        config: WidgetWindowConfig,
    ) -> crate::Result<bool> {
        let label_log = config.label.clone();
        let url = match config.url.as_deref() {
            Some(u) if !u.is_empty() => WebviewUrl::App(u.into()),
            _ => {
                let group = config
                    .group
                    .clone()
                    .or_else(|| crate::group::default_group().map(String::from))
                    .unwrap_or_default();
                let group = group.as_str();
                let size = config.size.as_deref().unwrap_or("small");
                let widget_id = config.widget_id.as_deref().unwrap_or("default");
                // The listed placement id is the one the window reads: instance or label.
                let instance = config.instance.as_deref().unwrap_or(&config.label);
                builtin_widget_url(group, size, widget_id, instance)
            }
        };
        let skip_taskbar = config.skip_taskbar;
        // Linux with the `linux` feature: build hidden, pin before the first map, show.
        let linux_pin = cfg!(all(target_os = "linux", feature = "linux"));
        let modes = placement::window_modes(&config, linux_pin);
        let target = window::resolve_target(app, &config);
        let move_to = |w: &tauri::WebviewWindow<R>, t: &window::Target| {
            let r = match t.physical {
                Some(p) => w.set_position(p),
                None => w.set_position(tauri::LogicalPosition::new(t.logical.0, t.logical.1)),
            };
            r.map_err(|e| Error::new(e.to_string()))
        };
        // Same label again (watch / rebuild): reuse the window. `close()` is asynchronous,
        // so building a new one right after it fails with "label already exists".
        if let Some(prev) = app.get_webview_window(&config.label) {
            let url = match url {
                WebviewUrl::External(u) | WebviewUrl::CustomProtocol(u) => u,
                WebviewUrl::App(path) => prev
                    .url()
                    .map_err(|e| Error::new(e.to_string()))?
                    .join(&path.to_string_lossy())
                    .map_err(|e| Error::new(e.to_string()))?,
                _ => return Err(Error::new("create_widget_window: unsupported url")),
            };
            prev.navigate(url).map_err(|e| Error::new(e.to_string()))?;
            prev.set_size(tauri::LogicalSize::new(config.width, config.height))
                .map_err(|e| Error::new(e.to_string()))?;
            if let Some(t) = &target {
                move_to(&prev, t)?;
            }
            window::apply_modes(&prev, &modes, config.effects, true);
            #[cfg(target_os = "windows")]
            if modes.layer == crate::models::WindowLayer::Desktop {
                if let Err(e) = crate::windows::attach_window(&prev) {
                    log::warn!("'{label_log}': {e}");
                }
            }
            prev.show().map_err(|e| Error::new(e.to_string()))?;
            Self::remember_open(app, &config);
            log::debug!("reused widget window '{label_log}'");
            return Ok(true);
        }
        // macOS clamps a new window below the menu bar; the island is only allowed up
        // there after `apply_modes` raises its level. Shown once it sits at the notch.
        let island_first_show = cfg!(target_os = "macos") && config.label == island::LABEL;
        let mut builder = WebviewWindowBuilder::new(app, &config.label, url)
            // Label doubles as WM_NAME so harnesses can find the window (xdotool).
            .title(&config.label)
            .inner_size(config.width, config.height)
            .decorations(false)
            .skip_taskbar(skip_taskbar)
            .always_on_top(modes.on_top())
            .resizable(false)
            // Widgets and activities appear without taking focus from the user's window.
            .focused(false)
            .visible(!linux_pin && !island_first_show);
        // Transparent windows on macOS require the host app's `macos-private-api`.
        #[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]
        {
            builder = builder.transparent(true);
        }
        // Opaque macOS window: at least black like the pill (square corners).
        #[cfg(all(target_os = "macos", not(feature = "macos-private-api")))]
        if config.label == island::LABEL {
            builder = builder.background_color(tauri::window::Color(0, 0, 0, 255));
        }

        if let Some(t) = &target {
            builder = builder.position(t.logical.0, t.logical.1);
        }

        let win = builder
            .build()
            .map_err(|e| Error::new(format!("create_widget_window '{}': {e}", config.label)))?;
        window::apply_modes(&win, &modes, config.effects, false);
        if let Some(t) = target.as_ref().filter(|t| t.physical.is_some()) {
            move_to(&win, t)?;
        }
        if island_first_show {
            if let Some(t) = &target {
                let _ = win.set_position(tauri::LogicalPosition::new(t.logical.0, t.logical.1));
            }
            win.show().map_err(|e| Error::new(e.to_string()))?;
        }
        // Behind the desktop icons (WorkerW / 24H2 Progman), kept across Explorer restarts.
        #[cfg(target_os = "windows")]
        if modes.layer == crate::models::WindowLayer::Desktop {
            if let Err(e) = crate::windows::attach_window(&win) {
                log::warn!("'{label_log}': {e}");
            }
        }
        #[cfg(all(target_os = "linux", feature = "linux"))]
        if !crate::linux::pin_widget_window(
            &win,
            &config,
            &modes,
            target.as_ref().map(|t| t.logical),
        ) {
            win.show().map_err(|e| Error::new(e.to_string()))?;
            // After the map: WMs often ignore the requested spot for DESKTOP windows.
            if let Some(t) = &target {
                move_to(&win, t)?;
            }
        }
        if config.remember_position != Some(false) {
            window::track_position(&win);
        }
        Self::remember_open(app, &config);
        log::debug!("created widget window '{label_log}'");
        Ok(true)
    }

    /// Add a widget window to the open set reopened by `restoreWindowsOnStart`.
    fn remember_open(app: &AppHandle<R>, config: &WidgetWindowConfig) {
        if config.label.starts_with("activity-") {
            return;
        }
        if let Some(w) = app.try_state::<Widget<R>>() {
            w.window_store.set_open(&config.label, Some(config));
        }
    }

    /// Reopen the widget windows that were open when the app last ran (closed with
    /// `close_widget_window` = removed from the set; quitting the app keeps them).
    /// `plugins.widgets.restoreWindowsOnStart: true` calls this at startup. Returns how
    /// many windows were requested.
    pub fn restore_windows(&self) -> usize {
        let list = self.window_store.open_windows();
        let n = list.len();
        for cfg in list {
            let label = cfg.label.clone();
            if let Err(e) = self.create_widget_window(cfg) {
                log::warn!("widgets: restore window '{label}': {e}");
            }
        }
        n
    }

    pub fn close_widget_window(&self, label: &str) -> crate::Result<bool> {
        self.windows.lock().unwrap().remove(label);
        if !label.starts_with("activity-") {
            self.window_store.set_open(label, None);
        }
        let app = self.app.clone();
        let label = label.to_string();

        #[cfg(all(target_os = "linux", feature = "linux"))]
        {
            if gtk::glib::MainContext::default().is_owner() {
                return Self::close_widget_window_on_main(&app, &label);
            }
        }

        if on_main_thread() {
            return Self::close_widget_window_on_main(&app, &label);
        }
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        self.app
            .run_on_main_thread(move || {
                let _ = tx.send(Self::close_widget_window_on_main(&app, &label));
            })
            .map_err(|e| Error::new(format!("main thread dispatch: {e}")))?;

        rx.recv()
            .map_err(|_| Error::new("close_widget_window: main thread dropped"))?
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
        // WidgetKit reads the files: they must hold every write made before this call.
        #[cfg(target_os = "macos")]
        self.flush_store();
        self.reload_all_timelines_now()
    }

    fn reload_all_timelines_now(&self) -> crate::Result<bool> {
        let _ = self.app.emit("widget-reload", "all");
        #[cfg(target_os = "macos")]
        if !unsafe { macos_widget_reload_all() } {
            return Err(Error::Unsupported(
                "WidgetKit reload needs macOS 11+".into(),
            ));
        }
        Ok(true)
    }

    pub fn reload_timelines(&self, of_kind: &str) -> crate::Result<bool> {
        #[cfg(target_os = "macos")]
        self.flush_store();
        let _ = self.app.emit("widget-reload", of_kind);
        #[cfg(target_os = "macos")]
        {
            let c = CString::new(of_kind)
                .map_err(|_| Error::new("reload_timelines: kind contains a NUL byte"))?;
            if !unsafe { macos_widget_reload_kind(c.as_ptr()) } {
                return Err(Error::Unsupported(
                    "WidgetKit reload needs macOS 11+".into(),
                ));
            }
        }
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

    /// Load a [`WidgetConfig`] JSON file from disk, then apply it like [`Self::set_widget_config`].
    pub fn set_widget_config_from_path(
        &self,
        path: impl AsRef<std::path::Path>,
        group: &str,
        widget_id: &str,
        skip_reload: bool,
    ) -> crate::Result<ApplyOutcome> {
        if widget_id.is_empty() {
            return Err(Error::new("widget_id must not be empty"));
        }
        let path = path.as_ref();
        let path_str = path
            .to_str()
            .ok_or_else(|| Error::new(format!("invalid UTF-8 path: {}", path.display())))?;
        let config = crate::commands::load_widget_config_from_path(path_str)?;
        self.set_widget_config(&config, group, widget_id, skip_reload)
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

        let mut config =
            crate::normalize::normalize(config, crate::capabilities::WidgetPlatform::current())
                .config;
        crate::image_prefetch::prefetch_remote_images(&mut config);

        let json = serde_json::to_string(&config)
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
                skip: Some(SkipReason::Unchanged { hash }),
            });
            self.trace.push(TraceEvent::Reload {
                performed: false,
                reason: outcome.reload.clone(),
            });
            self.maybe_flush_trace();
            return Ok(outcome);
        }

        crate::capabilities::log_capabilities(&config);
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
        self.write_board_card(&config, group, widget_id)?;

        let reload = if skip_reload {
            ReloadOutcome::Skipped {
                why: "skip_reload".into(),
            }
        } else {
            // macOS: the writer reloads right after this config is in the files.
            #[cfg(target_os = "macos")]
            let reloaded = {
                self.queue_persist(group, None, true);
                Ok::<bool, Error>(true)
            };
            #[cfg(not(target_os = "macos"))]
            let reloaded = self.reload_all_timelines();
            match reloaded {
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

    /// Widgets Board: one Adaptive Card with a body per size (`$when` on
    /// `$host.widgetSize`), resolved for now (entries, `{{…}}`, `setWidgetData`). While the
    /// app runs, the card is rebuilt at the next scheduled entry.
    #[cfg(target_os = "windows")]
    pub(crate) fn write_board_card(
        &self,
        config: &WidgetConfig,
        group: &str,
        widget_id: &str,
    ) -> crate::Result<()> {
        let (id, instance) = match widget_id.split_once(store::INSTANCE_SEPARATOR) {
            Some((id, inst)) => (id, Some(inst)),
            None => (widget_id, None),
        };
        let read_json = |prefix: &str| {
            store::lookup_keys(prefix, id, instance)
                .into_iter()
                .find_map(|k| self.get_items(&k, group).ok().flatten())
                .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        };
        let host_data = read_json(store::DATA_KEY_PREFIX);
        let fetched = read_json(store::FETCHED_KEY_PREFIX);
        let data = crate::resolve::merge_data([host_data.as_ref(), fetched.as_ref()]);
        let locale = windows_locale();
        let now = store::now_ms();
        let raw = serde_json::to_value(config).unwrap_or_default();
        let last_fetch = self.board_fetch_source(&raw, group, widget_id, instance, &locale, now);
        let Some((result, _)) =
            crate::adaptive_card::board_card(config, data.as_ref(), now, &locale, instance)
        else {
            return Ok(());
        };
        let template = serde_json::to_string(&result.card)
            .map_err(|e| Error::new(format!("serialize adaptive card: {e}")))?;
        self.set_items(
            &crate::adaptive_card::ac_template_key(widget_id),
            &template,
            group,
        )?;
        self.set_items(&crate::adaptive_card::ac_data_key(widget_id), "{}", group)?;
        // One timer per widget: every rebuild bumps the generation, an older sleeper exits.
        let gen = {
            let mut timers = self.board_timers.lock().unwrap();
            let g = timers.entry(widget_id.to_string()).or_insert(0);
            *g += 1;
            *g
        };
        if let Some(at) = crate::resolve::next_wake_ms(&raw, now, last_fetch) {
            let app = self.app.clone();
            let (config, group, widget_id) =
                (config.clone(), group.to_string(), widget_id.to_string());
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(
                    at.saturating_sub(store::now_ms()) + 50,
                ));
                if let Some(w) = app.try_state::<Widget<R>>() {
                    if w.board_timers.lock().unwrap().get(&widget_id) != Some(&gen) {
                        return;
                    }
                    let current = w.get_widget_config(&group, &widget_id).ok().flatten();
                    let cfg = current.unwrap_or(config);
                    let _ = w.write_board_card(&cfg, &group, &widget_id);
                }
            });
        }
        Ok(())
    }

    /// Widgets Board cannot fetch: while the app runs, the host fetches the config's
    /// `source` when due (in a thread), stores it as `fetched:{id}` and rebuilds the card.
    /// Returns the time of the last fetch attempt (for the wake-up timer).
    #[cfg(target_os = "windows")]
    fn board_fetch_source(
        &self,
        raw: &serde_json::Value,
        group: &str,
        widget_id: &str,
        instance: Option<&str>,
        locale: &str,
        now: u64,
    ) -> Option<u64> {
        let ctx = crate::resolve::ResolveContext {
            now_ms: now,
            locale,
            instance,
            ..Default::default()
        };
        let req = crate::resolve::source_request(raw, &ctx)?;
        let mut attempts = self.board_fetches.lock().unwrap();
        let last = attempts.get(widget_id).copied();
        let due = last.map_or(true, |t| now >= t + u64::from(req.refresh_minutes) * 60_000);
        if !due {
            return last;
        }
        attempts.insert(widget_id.to_string(), now);
        drop(attempts);
        #[cfg(feature = "image-prefetch")]
        {
            let app = self.app.clone();
            let (group, widget_id) = (group.to_string(), widget_id.to_string());
            std::thread::spawn(move || {
                let Some(text) = fetch_source_text(&req) else {
                    log::warn!("widget source fetch failed for {widget_id}");
                    return;
                };
                if let Some(w) = app.try_state::<Widget<R>>() {
                    let key = format!("{}{widget_id}", store::FETCHED_KEY_PREFIX);
                    if w.set_items(&key, &text, &group).is_ok() {
                        if let Ok(Some(cfg)) = w.get_widget_config(&group, &widget_id) {
                            let _ = w.write_board_card(&cfg, &group, &widget_id);
                        }
                    }
                }
            });
        }
        #[cfg(not(feature = "image-prefetch"))]
        let _ = (group, req);
        Some(now)
    }

    /// `setWidgetData` changed `{{data.…}}`: rebuild what the host renders itself.
    pub fn data_changed(&self, group: &str, widget_id: &str, instance: Option<&str>) {
        #[cfg(target_os = "windows")]
        {
            let id = store::scoped_id(widget_id, instance);
            let cfg = self
                .get_widget_config(group, &id)
                .ok()
                .flatten()
                .or_else(|| self.get_widget_config(group, widget_id).ok().flatten());
            if let Some(cfg) = cfg {
                let _ = self.write_board_card(&cfg, group, &id);
            }
        }
        #[cfg(not(target_os = "windows"))]
        let _ = (group, widget_id, instance);
    }

    /// Names of transports that hold the current map after a write.
    fn written_transport_names(&self, group: &str) -> Vec<String> {
        #[cfg(target_os = "macos")]
        {
            let _ = group;
            vec![self.macos_driver.name().to_string()]
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
        let mut freshest = store::pick_freshest(maps);
        if store::map_nonce(&freshest) > store::map_nonce(map) {
            *map = freshest.clone();
        }
        // Host writes still on their way to the files win over an extension map read
        // in between.
        self.persist.overlay(group, &mut freshest);
        Ok(freshest)
    }

    /// Drain pending actions for a group. On macOS the whole read-drain-write runs under
    /// the App Group file lock the widget extension appends under.
    pub fn poll_pending_actions(
        &self,
        group: &str,
    ) -> crate::Result<Vec<crate::WidgetActionEnvelope>> {
        #[cfg(target_os = "macos")]
        return crate::macos_transport::with_store_lock(group, || {
            self.drain_pending_actions(group)
        });
        #[cfg(not(target_os = "macos"))]
        self.drain_pending_actions(group)
    }

    fn drain_pending_actions(
        &self,
        group: &str,
    ) -> crate::Result<Vec<crate::WidgetActionEnvelope>> {
        self.remember_group(group);

        // Every 30 s also the siblings (taps the extension fanned out there); else only the
        // working channel, so a 100 ms poll is a `stat` of one file.
        #[cfg(target_os = "macos")]
        let sweep = self.sweep_due(group);
        #[cfg(target_os = "macos")]
        let polled: Vec<Arc<dyn Transport>> = if sweep {
            crate::macos_transport::all_transports(group)
        } else {
            self.hot_transports(group)
        };
        #[cfg(target_os = "macos")]
        let disk_maps: Vec<DataMap> = {
            let mut maps = Vec::new();
            for t in polled.iter() {
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

        #[cfg(target_os = "macos")]
        let actions = {
            let harvested = crate::macos_transport::harvest_pending_in(&polled);
            if !harvested.is_empty() {
                harvested
            } else {
                parse_pending_actions(map.get(PENDING_ACTIONS_KEY).map(|s| s.as_str()))
            }
        };
        #[cfg(not(target_os = "macos"))]
        let actions = parse_pending_actions(map.get(PENDING_ACTIONS_KEY).map(|s| s.as_str()));

        if actions.is_empty() {
            return Ok(Vec::new());
        }

        map.insert(PENDING_ACTIONS_KEY.into(), "[]".into());
        // A queued host write of the queue would bring the drained taps back.
        #[cfg(target_os = "macos")]
        self.persist.forget(group, PENDING_ACTIONS_KEY);
        #[cfg(target_os = "macos")]
        {
            let floor = crate::macos_transport::max_nonce_in(&self.hot_transports(group));
            store::touch_meta_above(map, floor);
        }
        #[cfg(not(target_os = "macos"))]
        {
            store::touch_meta(map);
        }
        let snapshot = map.clone();
        drop(store);

        #[cfg(target_os = "windows")]
        persist_windows_shared_map(&self.storage_path(group)?, &snapshot, &actions)?;
        #[cfg(not(target_os = "windows"))]
        self.persist_map(group, &snapshot)?;
        let _ = self.app.emit("widget-update", group);
        #[cfg(target_os = "macos")]
        {
            crate::macos_transport::clear_pending_actions_in(&polled, &actions);
        }

        Ok(actions)
    }

    pub fn report_receipt(&self, receipt: WidgetRenderReceipt) -> crate::Result<bool> {
        self.remember_group(&receipt.group);
        let trigger = receipt.trigger.clone().unwrap_or_else(|| "timeline".into());
        let lag_ms = {
            let since = self.trace.list_since(None);
            since
                .iter()
                .rev()
                .find_map(|e| match &e.event {
                    TraceEvent::ConfigSet { nonce, .. }
                        if *nonce == receipt.nonce && *nonce > 0 =>
                    {
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

    /// JavaScript drains the queue itself: queued taps may be emitted from now on.
    pub fn mark_listener_ready(&self) {
        self.listener_gate.open();
    }

    /// Open widget windows, plus placements that rendered recently (macOS WidgetKit
    /// extension, Windows Widgets Board) according to their receipts.
    pub fn get_active_widgets(
        &self,
        group: &str,
    ) -> crate::Result<Vec<crate::store::ActiveWidget>> {
        let open: Vec<crate::store::ActiveWidget> = {
            let mut windows = self.windows.lock().unwrap();
            windows.retain(|label, _| self.app.get_webview_window(label).is_some());
            // Live-activity windows are not placed widgets.
            windows
                .values()
                .filter(|a| !a.widget_id.starts_with("activity."))
                .cloned()
                .collect()
        };
        #[allow(unused_mut)]
        let mut open = open;
        // Widgets Board instances the provider tracks (`board_widgets`, Store.cs).
        #[cfg(target_os = "windows")]
        if let Ok(Some(raw)) = self.get_items(store::BOARD_WIDGETS_KEY, group) {
            open.extend(store::parse_board_widgets(&raw));
        }
        let seen = store::active_from_receipts(
            &self.receipts.list(group),
            store::now_ms(),
            store::ACTIVE_RECEIPT_MAX_AGE_MS,
        );
        Ok(store::merge_active(open, seen))
    }

    /// Desktop surface, by `plugins.widgets.activitySurface`:
    /// - `"island"` (default): a black pill at the top centre of the primary display that
    ///   wraps the notch on notched Macs and expands on hover ([`island`]);
    /// - `"window"`: a small always-on-top window in the top-right corner drawing the
    ///   activity layout;
    /// - `"notification"` (feature `notify`, Linux / Windows): a system notification.
    ///
    /// When no window can open, a system notification instead (feature `notify`).
    pub fn start_activity(
        &self,
        content: crate::models::ActivityContent,
        group: &str,
    ) -> crate::Result<crate::models::ActivityHandle> {
        let id = content.id.clone();
        self.write_activity(&content, group)?;
        // A start always applies; a fresh one forgets a timestamp left by a failed start.
        let running = self.activities.lock().unwrap().contains_key(&id);
        match content.timestamp {
            Some(ts) => {
                self.activity_ts.lock().unwrap().insert(id.clone(), ts);
            }
            None if !running => {
                self.activity_ts.lock().unwrap().remove(&id);
            }
            None => {}
        }
        let handle = crate::models::ActivityHandle {
            id: id.clone(),
            surface: "desktop".into(),
            push_token: None,
        };
        let notified = crate::models::ActivityHandle {
            surface: "notification".into(),
            ..handle.clone()
        };
        // Already running: the config push redraws its window in place.
        if self.activities.lock().unwrap().contains_key(&id) {
            if self.island.lock().unwrap().contains(&id) {
                self.island.lock().unwrap().upsert(content, group);
                self.island_refresh()?;
                return Ok(handle);
            }
            if self.notifier.has(&id) {
                self.notify_activity(&content)?;
                return Ok(notified);
            }
            return Ok(handle);
        }
        let surface = self.cfg.activity_surface.as_deref().unwrap_or("island");
        if surface == "notification" && notify::Notifier::AVAILABLE {
            match self.notify_activity(&content) {
                Ok(()) => {
                    self.activities
                        .lock()
                        .unwrap()
                        .insert(id, group.to_string());
                    return Ok(notified);
                }
                Err(e) => log::warn!("widgets: activity notification failed ({e}); using a window"),
            }
        }
        if surface != "window" && surface != "notification" {
            self.island.lock().unwrap().upsert(content.clone(), group);
            self.activities
                .lock()
                .unwrap()
                .insert(id.clone(), group.to_string());
            match self.island_refresh() {
                Ok(()) => return Ok(handle),
                Err(e) => {
                    self.island.lock().unwrap().remove(&id);
                    self.activities.lock().unwrap().remove(&id);
                    if !notify::Notifier::AVAILABLE {
                        return Err(e);
                    }
                    log::warn!("widgets: activity island failed ({e}); using a notification");
                    self.notify_activity(&content)?;
                    self.activities
                        .lock()
                        .unwrap()
                        .insert(id, group.to_string());
                    return Ok(notified);
                }
            }
        }
        let (x, y) = self.activity_origin();
        let opened = self.create_widget_window(WidgetWindowConfig {
            label: activity_window_label(&id),
            url: None,
            width: ACTIVITY_WINDOW_SIZE.0,
            height: ACTIVITY_WINDOW_SIZE.1,
            x,
            y,
            always_on_top: true,
            skip_taskbar: true,
            group: Some(group.to_string()),
            widget_id: Some(crate::models::activity_widget_id(&id)),
            size: Some("medium".into()),
            // Stacked by `activity_origin`, not where a previous run was dragged.
            remember_position: Some(false),
            ..Default::default()
        });
        let handle = match opened {
            Ok(_) => handle,
            // No window possible (headless session, …): fall back to a notification.
            Err(e) if notify::Notifier::AVAILABLE => {
                log::warn!("widgets: activity window failed ({e}); using a notification");
                self.notify_activity(&content)?;
                notified
            }
            Err(e) => return Err(e),
        };
        self.activities
            .lock()
            .unwrap()
            .insert(id, group.to_string());
        Ok(handle)
    }

    pub fn update_activity(
        &self,
        content: crate::models::ActivityContent,
        group: &str,
    ) -> crate::Result<bool> {
        if !self.activities.lock().unwrap().contains_key(&content.id) {
            return Err(Error::new(format!(
                "activity \"{}\" is not running",
                content.id
            )));
        }
        // Out of order: an update older than the last applied one is dropped (`false`).
        if let Some(ts) = content.timestamp {
            let mut last = self.activity_ts.lock().unwrap();
            if last.get(&content.id).is_some_and(|prev| ts < *prev) {
                return Ok(false);
            }
            last.insert(content.id.clone(), ts);
        }
        // Cheap on macOS too: the store write is queued (see `PersistQueue`).
        self.write_activity(&content, group)?;
        if self.notifier.has(&content.id) {
            self.notify_activity(&content)?;
        }
        let on_island = self.island.lock().unwrap().contains(&content.id);
        if on_island {
            self.island.lock().unwrap().upsert(content, group);
            self.island_refresh()?;
        }
        Ok(true)
    }

    fn notify_activity(&self, content: &crate::models::ActivityContent) -> crate::Result<()> {
        let config = self.app.config();
        let app_name = if cfg!(target_os = "windows") {
            // Toast AppUserModelID.
            config.identifier.as_str()
        } else {
            config.product_name.as_deref().unwrap_or(&config.identifier)
        };
        self.notifier.show(app_name, content)
    }

    pub fn end_activity(&self, id: &str, dismiss_after_ms: Option<u64>) -> crate::Result<bool> {
        if self.activities.lock().unwrap().remove(id).is_none() {
            return Ok(false);
        }
        self.activity_ts.lock().unwrap().remove(id);
        let label = activity_window_label(id);
        let on_island = self.island.lock().unwrap().contains(id);
        match dismiss_after_ms.filter(|ms| *ms > 0) {
            None if on_island => {
                self.island.lock().unwrap().remove(id);
                self.island_refresh()?;
                Ok(true)
            }
            None => {
                self.notifier.close(id);
                self.close_widget_window(&label)
            }
            Some(ms) => {
                let app = self.app.clone();
                let id = id.to_string();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(ms));
                    // Started again during the delay: the window belongs to the new run.
                    if app
                        .try_state::<Widget<R>>()
                        .is_some_and(|w| w.activities.lock().unwrap().contains_key(&id))
                    {
                        return;
                    }
                    if let Some(w) = app.try_state::<Widget<R>>() {
                        w.notifier.close(&id);
                        if on_island {
                            w.island.lock().unwrap().remove(&id);
                            if let Err(e) = w.island_refresh() {
                                log::warn!("widgets: activity island: {e}");
                            }
                            return;
                        }
                    }
                    let handle = app.clone();
                    let _ = app.run_on_main_thread(move || {
                        if let Some(w) = handle.get_webview_window(&label) {
                            let _ = w.close();
                        }
                    });
                });
                Ok(true)
            }
        }
    }

    /// Live Updates promotion is an Android gate; desktop activities need no permission.
    pub fn can_promote_activities(&self) -> crate::Result<bool> {
        Ok(true)
    }

    pub fn open_activity_promotion_settings(&self) -> crate::Result<bool> {
        Ok(false)
    }

    /// Push-to-start is an ActivityKit (iOS) feature.
    pub fn activity_push_to_start_token(&self) -> crate::Result<Option<String>> {
        Ok(None)
    }

    pub fn list_activities(&self) -> crate::Result<Vec<String>> {
        let mut ids: Vec<String> = self.activities.lock().unwrap().keys().cloned().collect();
        ids.sort();
        Ok(ids)
    }

    fn write_activity(
        &self,
        content: &crate::models::ActivityContent,
        group: &str,
    ) -> crate::Result<()> {
        let layout = content.effective_layout();
        let cfg = WidgetConfig {
            small: Some(layout.clone()),
            medium: Some(layout),
            ..Default::default()
        };
        self.set_widget_config(
            &cfg,
            group,
            &crate::models::activity_widget_id(&content.id),
            true,
        )?;
        Ok(())
    }

    /// Run `f` on the main thread (AppKit / GTK calls) and wait for its result.
    fn on_main<T: Send + 'static>(
        &self,
        f: impl FnOnce(&AppHandle<R>) -> T + Send + 'static,
    ) -> Option<T> {
        #[cfg(all(target_os = "linux", feature = "linux"))]
        if gtk::glib::MainContext::default().is_owner() {
            return Some(f(&self.app));
        }
        if on_main_thread() {
            return Some(f(&self.app));
        }
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let app = self.app.clone();
        self.app
            .run_on_main_thread(move || {
                let _ = tx.send(f(&app));
            })
            .ok()?;
        rx.recv().ok()
    }

    /// Bring the island window in line with [`Self::island`]: open it, resize / move it
    /// to the current shapes and send the page the new view (`activity-island`), or close
    /// it when no activity is left.
    fn island_refresh(&self) -> crate::Result<()> {
        let screen = self
            .on_main(island::screen)
            .flatten()
            .unwrap_or(island::Screen::FALLBACK);
        let (layout, view, group) = {
            let isl = self.island.lock().unwrap();
            if isl.is_empty() {
                drop(isl);
                return self.close_widget_window(island::LABEL).map(|_| ());
            }
            (isl.layout(&screen), isl.view(&screen), isl.front_group())
        };
        // Before the early return: the poller ends with the last activity, and a new one
        // can arrive while the old window still exists (a quick switch).
        let start_poller = !std::mem::replace(&mut self.island.lock().unwrap().polling, true);
        if start_poller {
            let app = self.app.clone();
            std::thread::spawn(move || Self::island_hover_poller(app));
        }
        if let Some(win) = self.app.get_webview_window(island::LABEL) {
            island::place(&win, layout.window);
            // Global: the page listens with target `Any`, which `emit_to(label)` skips.
            let _ = self.app.emit("activity-island", &view);
            return Ok(());
        }
        let r = layout.window;
        self.create_widget_window(WidgetWindowConfig {
            label: island::LABEL.into(),
            url: None,
            width: r.w,
            height: r.h,
            x: Some(r.x),
            y: Some(r.y),
            always_on_top: true,
            skip_taskbar: true,
            focusable: Some(false),
            all_workspaces: Some(true),
            shadow: Some(false),
            group,
            widget_id: Some(island::WIDGET_ID.into()),
            size: Some("medium".into()),
            remember_position: Some(false),
            ..Default::default()
        })
        .map(|_| ())
    }

    /// Hover from the cursor position: a webview in an inactive app gets no mouse-enter
    /// on macOS (WKWebView tracks only in the key window), so the host samples the global
    /// cursor every 80 ms while the island exists. Ends with the last activity.
    fn island_hover_poller(app: AppHandle<R>) {
        let mut outside = 0;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(80));
            let Some(w) = app.try_state::<Widget<R>>() else {
                return;
            };
            let expanded = {
                let mut isl = w.island.lock().unwrap();
                if isl.is_empty() {
                    isl.polling = false;
                    return;
                }
                isl.expanded
            };
            let Some(win) = app.get_webview_window(island::LABEL) else {
                continue;
            };
            // No global cursor (Wayland): the page's own hover / click handles it.
            let (Ok(p), Ok(pos), Ok(size)) = (
                app.cursor_position(),
                win.outer_position(),
                win.outer_size(),
            ) else {
                continue;
            };
            let inside = p.x >= pos.x as f64
                && p.y >= pos.y as f64
                && p.x < pos.x as f64 + size.width as f64
                && p.y < pos.y as f64 + size.height as f64;
            let r = match island::hover_step(inside, expanded, &mut outside) {
                island::Hover::Nothing => Ok(None),
                island::Hover::Expand => w.activity_island(Some(true), None),
                island::Hover::AskCollapse => {
                    let screen = w
                        .on_main(island::screen)
                        .flatten()
                        .unwrap_or(island::Screen::FALLBACK);
                    let mut view = w.island.lock().unwrap().view(&screen);
                    view.expanded = false;
                    let _ = app.emit("activity-island", &view);
                    Ok(None)
                }
                island::Hover::Collapse => w.activity_island(Some(false), None),
            };
            if let Err(e) = r {
                log::debug!("widgets: island hover: {e}");
            }
        }
    }

    /// Island page → host: expand / collapse (hover, click) or bring a minimal activity to
    /// the pill. Returns what the island shows (`None` without an island).
    pub fn activity_island(
        &self,
        expanded: Option<bool>,
        focus: Option<&str>,
    ) -> crate::Result<Option<serde_json::Value>> {
        let changed = {
            let mut isl = self.island.lock().unwrap();
            if isl.is_empty() {
                return Ok(None);
            }
            let mut changed = focus.is_some_and(|id| isl.focus(id));
            if let Some(e) = expanded.filter(|e| *e != isl.expanded) {
                isl.expanded = e;
                changed = true;
            }
            changed
        };
        if changed {
            self.island_refresh()?;
        }
        let screen = self
            .on_main(island::screen)
            .flatten()
            .unwrap_or(island::Screen::FALLBACK);
        let view = self.island.lock().unwrap().view(&screen);
        Ok(Some(serde_json::to_value(view)?))
    }

    /// Top-right corner of the primary monitor, below the menu bar, stacked per activity.
    fn activity_origin(&self) -> (Option<f64>, Option<f64>) {
        let Some(m) = island::primary_monitor(&self.app) else {
            return (None, None);
        };
        // Work area: below the macOS menu bar, clear of the Windows taskbar / Linux panels,
        // and offset by the monitor's own position in the virtual desktop.
        let scale = m.scale_factor();
        let area = m.work_area();
        let n = self.activities.lock().unwrap().len();
        let (x, y) = activity_slot(
            area.position.x as f64 / scale,
            area.position.y as f64 / scale,
            area.size.width as f64 / scale,
            n,
        );
        (Some(x), Some(y))
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
        std::thread::spawn(move || loop {
            // A tap in the WidgetKit extension waits for this poll: keep it short. Cheap,
            // the store files are only re-read when `stat` says they changed.
            std::thread::sleep(std::time::Duration::from_millis(100));
            let Some(widget) = groups_handle.try_state::<Widget<R>>() else {
                continue;
            };
            widget.inner().maybe_flush_trace();
            if !widget.inner().listener_gate.is_open(store::now_ms()) {
                continue;
            }
            let groups = widget.inner().known_groups.lock().unwrap().clone();
            for group in groups {
                match widget.inner().poll_pending_actions(&group) {
                    Ok(actions) if !actions.is_empty() => {
                        widget.inner().trace.push(TraceEvent::Poll {
                            count: actions.len(),
                        });
                        for action in actions {
                            // A `set` tap in the WidgetKit extension: state, not an action.
                            match store::state_event(&action) {
                                Some(ev) => {
                                    let _ = groups_handle.emit("widget-state-changed", ev);
                                }
                                None => {
                                    let _ = groups_handle.emit("widget-action", action);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}

// ─── macOS helpers ────────────────────────────────────────────────────────────

/// Host writes waiting for the writer thread, per group. Writes made while the writer
/// is busy (a stalled Group Container call) fold into one file write: latest value per
/// key, one reload.
#[cfg(target_os = "macos")]
#[derive(Default)]
struct PersistQueue {
    jobs: Mutex<HashMap<String, PersistJob>>,
    /// Keys the writer has taken but not written yet, per group.
    inflight: Mutex<HashMap<String, Vec<(String, String)>>>,
    changed: std::sync::Condvar,
    started: Mutex<bool>,
}

#[cfg(target_os = "macos")]
#[derive(Default)]
struct PersistJob {
    keys: Vec<(String, String)>,
    reload: bool,
}

#[cfg(target_os = "macos")]
impl PersistQueue {
    fn push(&self, group: &str, kv: Option<(&str, &str)>, reload: bool) {
        let mut jobs = self.jobs.lock().unwrap();
        let job = jobs.entry(group.to_string()).or_default();
        if let Some((k, v)) = kv {
            job.keys.retain(|(old, _)| old != k);
            job.keys.push((k.to_string(), v.to_string()));
        }
        job.reload |= reload;
        self.changed.notify_all();
    }

    /// Block until a group has work; it counts as busy until [`Self::done`].
    fn next(&self) -> (String, PersistJob) {
        let mut jobs = self.jobs.lock().unwrap();
        loop {
            if let Some(group) = jobs.keys().next().cloned() {
                let job = jobs.remove(&group).unwrap_or_default();
                self.inflight
                    .lock()
                    .unwrap()
                    .insert(group.clone(), job.keys.clone());
                return (group, job);
            }
            jobs = self.changed.wait(jobs).unwrap();
        }
    }

    fn done(&self, group: &str) {
        let _jobs = self.jobs.lock().unwrap();
        self.inflight.lock().unwrap().remove(group);
        self.changed.notify_all();
    }

    /// Unwritten host keys of `group`, oldest first (in flight, then queued).
    fn unwritten(&self, group: &str) -> Vec<(String, String)> {
        let jobs = self.jobs.lock().unwrap();
        let mut out = self
            .inflight
            .lock()
            .unwrap()
            .get(group)
            .cloned()
            .unwrap_or_default();
        if let Some(job) = jobs.get(group) {
            out.extend(job.keys.iter().cloned());
        }
        out
    }

    /// Keys the writer took for `group` and has not written, minus forgotten ones.
    fn in_flight(&self, group: &str) -> Vec<(String, String)> {
        self.inflight
            .lock()
            .unwrap()
            .get(group)
            .cloned()
            .unwrap_or_default()
    }

    /// A newer host write of `key` went to the files directly: drop the queued one.
    fn forget(&self, group: &str, key: &str) {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.get_mut(group) {
            job.keys.retain(|(k, _)| k != key);
        }
        if let Some(keys) = self.inflight.lock().unwrap().get_mut(group) {
            keys.retain(|(k, _)| k != key);
        }
    }

    fn overlay(&self, group: &str, map: &mut DataMap) {
        for (k, v) in self.unwritten(group) {
            map.insert(k, v);
        }
    }

    fn wait_idle(&self) {
        let mut jobs = self.jobs.lock().unwrap();
        while !jobs.is_empty() || !self.inflight.lock().unwrap().is_empty() {
            jobs = self.changed.wait(jobs).unwrap();
        }
    }
}

#[cfg(target_os = "macos")]
extern "C" {
    fn macos_widget_reload_all() -> bool;
    fn macos_widget_reload_kind(kind: *const std::ffi::c_char) -> bool;
}

#[cfg(target_os = "macos")]
fn macos_shared_container(group: &str) -> Option<PathBuf> {
    crate::macos_transport::shared_container_dir(group)
}

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
/// Match `WidgetStore.WithFileLock`: exclusive `.lock` + merge under it so
/// provider-enqueued `pending_actions` are not wiped by a concurrent host write.
#[cfg(target_os = "windows")]
fn persist_windows_shared_map(
    path: &PathBuf,
    map: &DataMap,
    drained: &[crate::WidgetActionEnvelope],
) -> crate::Result<()> {
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
            // The provider owns the queue: keep the disk copy minus what we drained.
            let rest = store::pending_minus_drained(merged.get(k).map(String::as_str), drained)?;
            merged.insert(k.clone(), rest);
            continue;
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

#[cfg(test)]
mod builtin_url_tests {
    use super::*;
    use proptest::prelude::*;

    /// Inverse of `activity_window_label` (`__` → `_`, `_d` → `.`), None if not a label.
    fn decode_label(label: &str) -> Option<String> {
        let mut out = String::new();
        let mut it = label.strip_prefix("activity-")?.chars();
        while let Some(c) = it.next() {
            match c {
                '_' => match it.next()? {
                    '_' => out.push('_'),
                    'd' => out.push('.'),
                    _ => return None,
                },
                c => out.push(c),
            }
        }
        Some(out)
    }

    proptest! {
        /// Query components: unreserved bytes pass through, everything else is `%XX`, so a
        /// value can never inject `&` / `=` and percent-decoding gives the input back; the
        /// built-in URL parses with exactly the four parameters given.
        /// Activity window labels decode back to the id, so two ids never share a window.
        #[test]
        fn query_encoding_and_activity_labels_are_reversible(
            v in "[a-zA-Z0-9._~&=/?# %-]{0,16}",
            id in "[a-z._]{0,10}",
        ) {
            let enc = encode_query_component(&v);
            prop_assert!(!enc.contains(['&', '=', '?', '#', ' ', '/']), "{}", enc);
            let decoded: Vec<u8> = {
                let b = enc.as_bytes();
                let mut out = Vec::new();
                let mut i = 0;
                while i < b.len() {
                    if b[i] == b'%' {
                        out.push(u8::from_str_radix(&enc[i + 1..i + 3], 16).unwrap());
                        i += 3;
                    } else {
                        out.push(b[i]);
                        i += 1;
                    }
                }
                out
            };
            prop_assert_eq!(String::from_utf8(decoded).unwrap(), v.clone());
            let WebviewUrl::External(url) = builtin_widget_url(&v, "small", "w", "1") else {
                panic!("external url")
            };
            let pairs: Vec<(String, String)> = url.query_pairs().into_owned().collect();
            prop_assert_eq!(pairs.len(), 4);
            prop_assert_eq!(&pairs[0], &("group".to_string(), v.clone()));
            prop_assert_eq!(&pairs[2], &("widgetId".to_string(), "w".to_string()));

            prop_assert_eq!(decode_label(&activity_window_label(&id)), Some(id));
        }
    }

    /// Fixed examples of the same rules, plus where stacked activity windows go.
    #[test]
    fn encoding_label_and_slot_examples() {
        assert_eq!(
            encode_query_component("small&widgetId=hijack"),
            "small%26widgetId%3Dhijack"
        );
        assert_eq!(encode_query_component("a=b&c"), "a%3Db%26c");
        assert_eq!(
            encode_query_component("group.com.example.app"),
            "group.com.example.app"
        );
        assert_eq!(encode_query_component("weather_1"), "weather_1");
        assert_eq!(activity_window_label("a.b"), "activity-a_db");
        assert_eq!(activity_window_label("a_b"), "activity-a__b");
        assert_ne!(activity_window_label("a_d"), activity_window_label("a.d"));
        assert_ne!(activity_window_label("x_.y"), activity_window_label("x._y"));
        let (w, h) = ACTIVITY_WINDOW_SIZE;
        // Second monitor at x=1920, menu bar 25 pt: inside that work area, stacked.
        assert_eq!(
            activity_slot(1920.0, 25.0, 1440.0, 1),
            (1920.0 + 1440.0 - w - 16.0, 25.0 + 16.0 + h + 8.0)
        );
    }
}
