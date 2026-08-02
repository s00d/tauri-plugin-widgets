//! Concrete Apple transports + path helpers for tests / overrides.
//!
//! Override roots (no global `HOME` mutation):
//! - `WIDGET_CONTAINER_ROOT` — fake `$HOME` for `Library/Containers/…`
//! - `WIDGET_EXTENSION_BUNDLE` — extension id (default `{app_id}.widgetkit`)
//! - `WIDGET_APP_GROUP_DATA_FILE` — optional App Group `widget_data.json` path

use crate::error::Error;
use crate::models::WidgetConfig;
use crate::store::{
    self, config_key, encode_pending_actions, parse_pending_actions, touch_meta, DataMap,
    WidgetActionEnvelope, META_NONCE_KEY, META_UPDATED_AT_KEY, PENDING_ACTIONS_KEY,
};
use crate::transport::{Receipt, Transport, NAME_APPGROUP, NAME_CONTAINER, NAME_DEFAULTS};
use std::ffi::{CStr, CString};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Root used instead of `$HOME` when set (tests).
pub fn container_root() -> PathBuf {
    if let Ok(p) = std::env::var("WIDGET_CONTAINER_ROOT") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

/// Widget extension bundle id for the sandbox Containers path.
pub fn extension_bundle_id(group: &str) -> String {
    if let Ok(b) = std::env::var("WIDGET_EXTENSION_BUNDLE") {
        if !b.is_empty() {
            return b;
        }
    }
    let app_id = group.strip_prefix("group.").unwrap_or(group);
    format!("{app_id}.widgetkit")
}

pub fn sandbox_widget_data_path(group: &str) -> PathBuf {
    container_root()
        .join("Library/Containers")
        .join(extension_bundle_id(group))
        .join("Data")
        .join("widget_data.json")
}

pub fn sandbox_receipt_path(group: &str) -> PathBuf {
    sandbox_widget_data_path(group).with_file_name("widget_receipt.json")
}

pub fn app_group_data_override() -> Option<PathBuf> {
    std::env::var("WIDGET_APP_GROUP_DATA_FILE")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}

pub fn app_group_receipt_path(data_file: &Path) -> PathBuf {
    data_file.with_file_name("widget_receipt.json")
}

pub fn read_map_file(path: &Path) -> Option<DataMap> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

pub fn write_map_file(path: &Path, map: &DataMap) -> crate::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::Io(e.to_string()))?;
    }
    let json = serde_json::to_string_pretty(map).map_err(|e| Error::new(e.to_string()))?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, json.as_bytes()).map_err(|e| Error::Io(e.to_string()))?;
    fs::rename(&tmp, path).map_err(|e| Error::Io(e.to_string()))?;
    Ok(())
}

fn read_receipt_file(path: &Path) -> Option<Receipt> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

fn write_receipt_file(path: &Path, receipt: &Receipt) -> crate::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::Io(e.to_string()))?;
    }
    let json = serde_json::to_string(receipt).map_err(|e| Error::new(e.to_string()))?;
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, json.as_bytes()).map_err(|e| Error::Io(e.to_string()))?;
    fs::rename(&tmp, path).map_err(|e| Error::Io(e.to_string()))?;
    Ok(())
}

pub fn write_sandbox_map(group: &str, map: &DataMap) -> crate::Result<()> {
    write_map_file(&sandbox_widget_data_path(group), map)
}

extern "C" {
    fn macos_widget_container_path(group: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn macos_widget_free_string(ptr: *mut std::ffi::c_char);
    fn macos_widget_set_defaults_map(
        group: *const std::ffi::c_char,
        json_map: *const std::ffi::c_char,
    ) -> bool;
    fn macos_widget_get_defaults_map(group: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn macos_widget_set_defaults_string(
        group: *const std::ffi::c_char,
        key: *const std::ffi::c_char,
        value: *const std::ffi::c_char,
    ) -> bool;
    fn macos_widget_get_defaults_string(
        group: *const std::ffi::c_char,
        key: *const std::ffi::c_char,
    ) -> *mut std::ffi::c_char;
}

fn shared_container_dir(group: &str) -> Option<PathBuf> {
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

fn resolve_app_group_data_file(group: &str) -> Option<PathBuf> {
    app_group_data_override()
        .or_else(|| shared_container_dir(group).map(|d| d.join("widget_data.json")))
}

// ─── Transports ───────────────────────────────────────────────────────────────

struct FileTransport {
    name: &'static str,
    data_path: PathBuf,
    receipt_path: PathBuf,
    /// Local availability: parent exists or can be created / override set.
    available: bool,
}

impl Transport for FileTransport {
    fn name(&self) -> &'static str {
        self.name
    }

    fn available(&self) -> bool {
        self.available
    }

    fn read(&self) -> Option<DataMap> {
        read_map_file(&self.data_path)
    }

    fn write(&self, map: &DataMap) -> crate::Result<()> {
        write_map_file(&self.data_path, map)
    }

    fn read_receipt(&self) -> Option<Receipt> {
        read_receipt_file(&self.receipt_path)
    }

    fn write_receipt(&self, receipt: &Receipt) -> crate::Result<()> {
        write_receipt_file(&self.receipt_path, receipt)
    }
}

struct UserDefaultsTransport {
    group: String,
}

impl Transport for UserDefaultsTransport {
    fn name(&self) -> &'static str {
        NAME_DEFAULTS
    }

    fn available(&self) -> bool {
        // Suite construction usually succeeds even when delivery won't — local only.
        CString::new(self.group.as_str()).is_ok()
    }

    fn read(&self) -> Option<DataMap> {
        let c_group = CString::new(self.group.as_str()).ok()?;
        let ptr = unsafe { macos_widget_get_defaults_map(c_group.as_ptr()) };
        if ptr.is_null() {
            return None;
        }
        let json = unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned();
        unsafe { macos_widget_free_string(ptr) };
        serde_json::from_str(&json).ok()
    }

    fn write(&self, map: &DataMap) -> crate::Result<()> {
        let compact = serde_json::to_string(map).map_err(|e| Error::new(e.to_string()))?;
        let c_group = CString::new(self.group.as_str()).map_err(|e| Error::new(e.to_string()))?;
        let c_json = CString::new(compact).map_err(|e| Error::new(e.to_string()))?;
        let ok = unsafe { macos_widget_set_defaults_map(c_group.as_ptr(), c_json.as_ptr()) };
        if ok {
            Ok(())
        } else {
            Err(Error::new("UserDefaults map write failed"))
        }
    }

    fn read_receipt(&self) -> Option<Receipt> {
        let c_group = CString::new(self.group.as_str()).ok()?;
        let c_key = CString::new("widget_receipt").ok()?;
        let ptr = unsafe { macos_widget_get_defaults_string(c_group.as_ptr(), c_key.as_ptr()) };
        if ptr.is_null() {
            return None;
        }
        let json = unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned();
        unsafe { macos_widget_free_string(ptr) };
        serde_json::from_str(&json).ok()
    }

    fn write_receipt(&self, receipt: &Receipt) -> crate::Result<()> {
        let json = serde_json::to_string(receipt).map_err(|e| Error::new(e.to_string()))?;
        let c_group = CString::new(self.group.as_str()).map_err(|e| Error::new(e.to_string()))?;
        let c_key = CString::new("widget_receipt").map_err(|e| Error::new(e.to_string()))?;
        let c_val = CString::new(json).map_err(|e| Error::new(e.to_string()))?;
        let ok = unsafe {
            macos_widget_set_defaults_string(c_group.as_ptr(), c_key.as_ptr(), c_val.as_ptr())
        };
        if ok {
            Ok(())
        } else {
            Err(Error::new("UserDefaults receipt write failed"))
        }
    }
}

/// App Group shared-container file transport.
///
/// Fails loud when the OS does not return a container URL (typical for ad-hoc
/// signing). Set `transport = "widgetContainer"` for local ad-hoc builds, or
/// `WIDGET_APP_GROUP_DATA_FILE` in tests.
pub fn app_group_transport(group: &str) -> crate::Result<Arc<dyn Transport>> {
    let path = resolve_app_group_data_file(group).ok_or_else(|| {
        Error::new(format!(
            "App Group '{group}' is unavailable (containerURL returned nil).\n\
             • Enable App Groups on App + Widget Extension targets\n\
             • Sign with a real Team ID — ad-hoc does not share the App Group container\n\
             • For local/ad-hoc development set plugins.widgets.transport = \"widgetContainer\"\n\
             • Tests may set WIDGET_APP_GROUP_DATA_FILE to a writable path"
        ))
    })?;
    let receipt = app_group_receipt_path(&path);
    Ok(Arc::new(FileTransport {
        name: NAME_APPGROUP,
        data_path: path,
        receipt_path: receipt,
        available: true,
    }))
}

/// App Group UserDefaults suite transport (same Team ID requirements as App Group file).
pub fn user_defaults_transport(group: &str) -> Arc<dyn Transport> {
    Arc::new(UserDefaultsTransport {
        group: group.to_string(),
    })
}

/// Widget extension sandbox container file (ad-hoc friendly; host must not be sandboxed).
pub fn widget_container_transport(group: &str) -> Arc<dyn Transport> {
    let sandbox = sandbox_widget_data_path(group);
    Arc::new(FileTransport {
        name: NAME_CONTAINER,
        receipt_path: sandbox_receipt_path(group),
        available: true,
        data_path: sandbox,
    })
}

/// All host-writable Apple transports for `group` (best-effort App Group).
pub fn all_transports(group: &str) -> Vec<Arc<dyn Transport>> {
    let mut out = Vec::new();
    match app_group_transport(group) {
        Ok(t) => out.push(t),
        Err(e) => log::debug!("all_transports: appGroup skipped: {e}"),
    }
    out.push(user_defaults_transport(group));
    out.push(widget_container_transport(group));
    out
}

/// Max `__meta_nonce__` across every readable sibling transport.
pub fn max_nonce_across(group: &str) -> u64 {
    all_transports(group)
        .into_iter()
        .filter_map(|t| t.read())
        .map(|m| store::map_nonce(&m))
        .max()
        .unwrap_or(0)
}

/// Write `map` to every available transport except `primary` (same bytes / nonce).
///
/// The widget still multi-reads and picks by nonce. Mirroring keeps inactive
/// channels from serving a stale higher-nonce snapshot.
///
/// Concurrent enqueues: merge `pending_actions` from the sibling's current map
/// into the outgoing mirror so a host config write cannot wipe a just-appended action.
pub fn mirror_to_siblings(primary: &dyn Transport, group: &str, map: &DataMap) {
    for t in all_transports(group) {
        if t.name() == primary.name() || !t.available() {
            continue;
        }
        let mut out = map.clone();
        if let Some(existing) = t.read() {
            if let Some(pa) = existing.get(store::PENDING_ACTIONS_KEY) {
                // Prefer the longer / non-empty queue when the outgoing map cleared it.
                let outgoing_empty = out
                    .get(store::PENDING_ACTIONS_KEY)
                    .map(|s| s.trim().is_empty() || s.trim() == "[]")
                    .unwrap_or(true);
                if outgoing_empty && !pa.trim().is_empty() && pa.trim() != "[]" {
                    out.insert(store::PENDING_ACTIONS_KEY.into(), pa.clone());
                }
            }
        }
        if let Err(e) = t.write(&out) {
            log::debug!("mirror_to_siblings({}): {e}", t.name());
        }
    }
}

// ─── Helpers used by integration tests ────────────────────────────────────────

pub fn read_file_transports(group: &str, app_group_file: Option<&Path>) -> Vec<DataMap> {
    let mut maps = Vec::new();
    if let Some(m) = read_map_file(&sandbox_widget_data_path(group)) {
        maps.push(m);
    }
    if let Some(p) = app_group_file {
        if let Some(m) = read_map_file(p) {
            maps.push(m);
        }
    } else if let Some(p) = app_group_data_override() {
        if let Some(m) = read_map_file(&p) {
            maps.push(m);
        }
    }
    maps
}

pub fn get_config_freshest(
    group: &str,
    widget_id: &str,
    app_group_file: Option<&Path>,
    extra: impl IntoIterator<Item = DataMap>,
) -> crate::Result<Option<WidgetConfig>> {
    let mut maps = read_file_transports(group, app_group_file);
    maps.extend(extra);
    let freshest = store::pick_freshest(maps);
    let Some(raw) = freshest.get(&config_key(widget_id)) else {
        return Ok(None);
    };
    let config: WidgetConfig =
        serde_json::from_str(raw).map_err(|e| Error::new(format!("parse config: {e}")))?;
    Ok(Some(config))
}

pub fn put_config_in_map(
    map: &mut DataMap,
    widget_id: &str,
    config: &WidgetConfig,
) -> crate::Result<()> {
    let json = serde_json::to_string(config).map_err(|e| Error::new(e.to_string()))?;
    map.insert(config_key(widget_id), json);
    touch_meta(map);
    Ok(())
}

pub fn enqueue_action_like_extension(
    action: &str,
    payload: Option<&str>,
    widget_id: &str,
    group: &str,
) -> crate::Result<()> {
    let path = sandbox_widget_data_path(group);
    let mut map = read_map_file(&path).unwrap_or_default();
    let mut actions = parse_pending_actions(map.get(PENDING_ACTIONS_KEY).map(|s| s.as_str()));
    actions.push(WidgetActionEnvelope::new(
        action,
        payload.map(|s| s.to_string()),
        widget_id,
        group,
    ));
    map.insert(
        PENDING_ACTIONS_KEY.into(),
        encode_pending_actions(&actions)?,
    );
    touch_meta(&mut map);
    write_sandbox_map(group, &map)
}

pub fn poll_pending_actions_files(
    group: &str,
    app_group_file: Option<&Path>,
) -> crate::Result<Vec<WidgetActionEnvelope>> {
    let maps = read_file_transports(group, app_group_file);
    let mut freshest = store::pick_freshest(maps);
    let actions = parse_pending_actions(freshest.get(PENDING_ACTIONS_KEY).map(|s| s.as_str()));
    if actions.is_empty() {
        return Ok(Vec::new());
    }
    freshest.insert(PENDING_ACTIONS_KEY.into(), "[]".into());
    touch_meta(&mut freshest);
    write_sandbox_map(group, &freshest)?;
    if let Some(p) = app_group_file {
        let _ = write_map_file(p, &freshest);
    } else if let Some(p) = app_group_data_override() {
        let _ = write_map_file(&p, &freshest);
    }
    Ok(actions)
}

pub fn map_with_nonce(
    nonce: u64,
    widget_id: &str,
    config: &WidgetConfig,
) -> crate::Result<DataMap> {
    let mut map = DataMap::new();
    put_config_in_map(&mut map, widget_id, config)?;
    map.insert(META_NONCE_KEY.into(), nonce.to_string());
    map.insert(META_UPDATED_AT_KEY.into(), store::now_ms().to_string());
    Ok(map)
}
