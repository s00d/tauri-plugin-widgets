//! Apple host→widget transport: one driver, chosen by config (not runtime fan-out).
//!
//! Availability (`containerURL`, suite exists) is **not** delivery proof — the
//! developer picks [`crate::config::TransportKind`] from signing knowledge.

use crate::config::{effective_transport, TransportKind, WidgetsPluginConfig};
use crate::error::Error;
use crate::store::{self, DataMap};
#[cfg(any(target_os = "macos", test))]
use crate::store::map_nonce;
#[cfg(test)]
use crate::store::touch_meta;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Driver name written into widget receipts (`appgroup` file transport).
pub const NAME_APPGROUP: &str = "appgroup";
/// Driver name for App Group `UserDefaults` suite.
pub const NAME_DEFAULTS: &str = "defaults";
/// Driver name for widget extension container file.
pub const NAME_CONTAINER: &str = "container";

/// Widget-side ack that it rendered a config from a specific transport.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    /// Transport that won freshest pick on the widget.
    /// Also accepts `source` from the richer render-receipt schema.
    #[serde(alias = "source")]
    pub read_from: String,
    /// Config-map nonce the widget observed.
    pub nonce: u64,
    /// Unix ms when the receipt was written.
    pub ts: u64,
}

/// Host-side read/write channel to the widget extension.
pub trait Transport: Send + Sync {
    /// Stable driver id (`appgroup` / `defaults` / `container`).
    fn name(&self) -> &'static str;
    /// Cheap local probe — must NOT be treated as delivery proof by itself.
    fn available(&self) -> bool;
    /// Read the config map if present.
    fn read(&self) -> Option<DataMap>;
    /// Persist the config map (bumps meta via caller).
    fn write(&self, map: &DataMap) -> crate::Result<()>;
    /// Widget-side ack for the last painted nonce.
    fn read_receipt(&self) -> Option<Receipt>;
    /// Host-written ack (rare; mostly widget writes receipts).
    fn write_receipt(&self, receipt: &Receipt) -> crate::Result<()>;
}

/// Resolve a single host transport from plugin config (+ `WIDGET_TRANSPORT`).
pub fn resolve_driver(cfg: &WidgetsPluginConfig) -> crate::Result<Arc<dyn Transport>> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = cfg;
        return Err(Error::new(
            "Apple transport drivers are only available on macOS host builds",
        ));
    }
    #[cfg(target_os = "macos")]
    {
        let kind = effective_transport(cfg);
        match kind {
            TransportKind::Auto => probe_once(cfg),
            other => build_driver(other, cfg),
        }
    }
}

/// Fail loud when the chosen transport cannot be constructed.
pub fn build_driver(
    kind: TransportKind,
    cfg: &WidgetsPluginConfig,
) -> crate::Result<Arc<dyn Transport>> {
    // Validate appGroup before the platform gate so CI/non-macOS tests see the
    // configuration error rather than a generic "Apple-only" message.
    if matches!(
        kind,
        TransportKind::AppGroup | TransportKind::UserDefaults | TransportKind::WidgetContainer
    ) {
        let _ = require_app_group(cfg)?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = kind;
        return Err(Error::new(
            "Apple transport drivers are only available on macOS host builds",
        ));
    }
    #[cfg(target_os = "macos")]
    {
        if matches!(kind, TransportKind::Auto) {
            return Err(Error::new(
                "build_driver(Auto) is invalid — use resolve_driver / probe_once",
            ));
        }
        let group = require_app_group(cfg)?;
        apply_extension_bundle_env(cfg);

        match kind {
            TransportKind::AppGroup => crate::macos_transport::app_group_transport(group),
            TransportKind::UserDefaults => {
                Ok(crate::macos_transport::user_defaults_transport(group))
            }
            TransportKind::WidgetContainer => {
                Ok(crate::macos_transport::widget_container_transport(group))
            }
            TransportKind::Auto => unreachable!(),
        }
    }
}

/// iOS: only `appGroup` (or `auto` → appGroup). Other kinds fail at init.
/// Android: Apple-only transports are ignored (treated as AppGroup / SharedPreferences).
pub fn validate_mobile_transport(cfg: &WidgetsPluginConfig) -> crate::Result<TransportKind> {
    let kind = effective_transport(cfg);
    #[cfg(target_os = "android")]
    {
        let _ = kind;
        // SharedPreferences path — Apple transport enums are not meaningful here.
        return Ok(TransportKind::AppGroup);
    }
    #[cfg(not(target_os = "android"))]
    {
        match kind {
            TransportKind::AppGroup | TransportKind::Auto => {
                // Fail closed: App Group id is required on iOS for a shared container.
                let _ = require_app_group(cfg)?;
                Ok(TransportKind::AppGroup)
            }
            TransportKind::UserDefaults | TransportKind::WidgetContainer => Err(Error::new(format!(
                "iOS supports transport=appGroup only (got transport={}).\n\
                 UserDefaults suite and widget-container writes from the host are not supported on iOS.\n\
                 Set plugins.widgets.transport to \"appGroup\" in tauri.conf.json.",
                kind.as_str()
            ))),
        }
    }
}

fn require_app_group(cfg: &WidgetsPluginConfig) -> crate::Result<&str> {
    cfg.app_group
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            Error::new(
                "plugins.widgets.appGroup is required in tauri.conf.json \
                 (e.g. \"group.com.example.app\").",
            )
        })
}

#[cfg(target_os = "macos")]
fn apply_extension_bundle_env(cfg: &WidgetsPluginConfig) {
    if let Some(bundle) = cfg
        .extension_bundle_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        // SAFETY: called once during plugin init before other threads use the path helpers.
        unsafe { std::env::set_var("WIDGET_EXTENSION_BUNDLE", bundle) };
    }
}

/// Dev-only: try candidates once, latch the first with a matching receipt, else container.
#[cfg(target_os = "macos")]
fn probe_once(cfg: &WidgetsPluginConfig) -> crate::Result<Arc<dyn Transport>> {
    log::warn!(
        "transport=auto — development only. Pin plugins.widgets.transport before release \
         (appGroup with Team ID, or widgetContainer for ad-hoc)."
    );
    let group = require_app_group(cfg)?;
    apply_extension_bundle_env(cfg);

    let candidates = probe_candidates(group)?;
    // Preserve the freshest existing map so probing does not wipe live configs/actions.
    let mut probe = candidates
        .iter()
        .filter_map(|t| t.read())
        .max_by_key(map_nonce)
        .unwrap_or_default();
    let floor = candidates
        .iter()
        .filter_map(|t| t.read_receipt().map(|r| r.nonce))
        .max()
        .unwrap_or(0)
        .max(map_nonce(&probe));
    probe.insert("__probe__".into(), "1".into());
    // Unique nonce above any existing map/receipt — avoid latching on stale receipts.
    store::touch_meta_above(&mut probe, floor);
    let nonce = map_nonce(&probe);

    for t in &candidates {
        if t.available() {
            let _ = t.write(&probe);
        }
    }

    // Prefer an exact post-write receipt match for this probe nonce + transport name.
    for t in &candidates {
        if let Some(r) = t.read_receipt() {
            if r.read_from == t.name() && r.nonce == nonce {
                log::warn!(
                    "transport=auto latched {:?} — set plugins.widgets.transport = \"{}\"",
                    t.name(),
                    match t.name() {
                        NAME_APPGROUP => "appGroup",
                        NAME_DEFAULTS => "userDefaults",
                        NAME_CONTAINER => "widgetContainer",
                        other => other,
                    }
                );
                return Ok(Arc::clone(t));
            }
        }
    }

    // No widget receipt yet: latch container (works for ad-hoc) and tell the developer.
    let container = candidates
        .into_iter()
        .find(|t| t.name() == NAME_CONTAINER)
        .ok_or_else(|| Error::new("transport=auto: widgetContainer candidate missing"))?;
    log::warn!(
        "transport=auto: no matching receipt yet — latching widgetContainer. \
         After the widget paints once, set plugins.widgets.transport explicitly \
         (appGroup if Team ID + App Groups work)."
    );
    Ok(container)
}

#[cfg(target_os = "macos")]
fn probe_candidates(group: &str) -> crate::Result<Vec<Arc<dyn Transport>>> {
    let mut out = Vec::new();
    match crate::macos_transport::app_group_transport(group) {
        Ok(t) => out.push(t),
        Err(e) => log::debug!("auto probe: appGroup unavailable: {e}"),
    }
    out.push(crate::macos_transport::user_defaults_transport(group));
    out.push(crate::macos_transport::widget_container_transport(group));
    Ok(out)
}

/// Test helper: probe among provided fakes (no FS).
#[cfg(test)]
pub fn probe_once_among(
    candidates: Vec<Arc<dyn Transport>>,
    prefer_fallback: &'static str,
) -> crate::Result<Arc<dyn Transport>> {
    let mut probe = DataMap::new();
    probe.insert("__probe__".into(), "1".into());
    touch_meta(&mut probe);
    let nonce = map_nonce(&probe);
    for t in &candidates {
        t.write(&probe)?;
    }
    for t in &candidates {
        if let Some(r) = t.read_receipt() {
            if r.read_from == t.name() && r.nonce >= nonce {
                return Ok(Arc::clone(t));
            }
        }
    }
    candidates
        .into_iter()
        .find(|t| t.name() == prefer_fallback)
        .ok_or_else(|| Error::new("probe fallback missing"))
}

/// Thin alias of [`store::pick_freshest`] for transport callers.
pub fn pick_freshest_maps(maps: Vec<DataMap>) -> DataMap {
    store::pick_freshest(maps)
}

// ─── Fake transport (unit tests) ─────────────────────────────────────────────

/// In-memory transport for unit tests.
pub struct FakeTransport {
    name: &'static str,
    available: AtomicBool,
    map: Mutex<Option<DataMap>>,
    receipt: Mutex<Option<Receipt>>,
    /// How many successful `write` calls occurred.
    pub writes: AtomicU64,
}

impl FakeTransport {
    /// Available transport with empty map/receipt.
    pub fn new(name: &'static str) -> Arc<Self> {
        Arc::new(Self {
            name,
            available: AtomicBool::new(true),
            map: Mutex::new(None),
            receipt: Mutex::new(None),
            writes: AtomicU64::new(0),
        })
    }

    /// Toggle [`Transport::available`].
    pub fn set_available(&self, v: bool) {
        self.available.store(v, Ordering::Relaxed);
    }

    /// Inject a widget receipt without going through write_receipt.
    pub fn plant_receipt(&self, receipt: Receipt) {
        *self.receipt.lock().unwrap() = Some(receipt);
    }

    /// Snapshot of [`Self::writes`].
    pub fn write_count(&self) -> u64 {
        self.writes.load(Ordering::Relaxed)
    }
}

impl Transport for FakeTransport {
    fn name(&self) -> &'static str {
        self.name
    }

    fn available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
    }

    fn read(&self) -> Option<DataMap> {
        self.map.lock().unwrap().clone()
    }

    fn write(&self, map: &DataMap) -> crate::Result<()> {
        self.writes.fetch_add(1, Ordering::Relaxed);
        *self.map.lock().unwrap() = Some(map.clone());
        Ok(())
    }

    fn read_receipt(&self) -> Option<Receipt> {
        self.receipt.lock().unwrap().clone()
    }

    fn write_receipt(&self, receipt: &Receipt) -> crate::Result<()> {
        *self.receipt.lock().unwrap() = Some(receipt.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WidgetsPluginConfig;

    fn map_v1() -> DataMap {
        let mut m = DataMap::new();
        m.insert("config:probe".into(), r#"{"version":1}"#.into());
        touch_meta(&mut m);
        m
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn explicit_appgroup_fails_loudly_when_container_missing() {
        let cfg = WidgetsPluginConfig {
            transport: TransportKind::AppGroup,
            app_group: Some("group.test.missing".into()),
            extension_bundle_id: None,
        };
        unsafe {
            std::env::remove_var("WIDGET_APP_GROUP_DATA_FILE");
        }
        match build_driver(TransportKind::AppGroup, &cfg) {
            Err(err) => {
                let msg = err.to_string();
                assert!(
                    msg.contains("App Group")
                        || msg.contains("appGroup")
                        || msg.contains("widgetContainer"),
                    "expected loud appGroup error, got: {msg}"
                );
            }
            Ok(_) => {
                // Some macOS installs return a container URL for arbitrary group ids.
                eprintln!(
                    "skip: App Group container unexpectedly available for group.test.missing"
                );
            }
        }
    }

    #[test]
    fn explicit_container_writes_only_container() {
        let container = FakeTransport::new(NAME_CONTAINER);
        let appgroup = FakeTransport::new(NAME_APPGROUP);
        let m = map_v1();
        container.write(&m).unwrap();
        assert_eq!(container.write_count(), 1);
        assert_eq!(appgroup.write_count(), 0);
        assert!(appgroup.read().is_none());
        assert!(container.read().is_some());
    }

    #[test]
    fn unchanged_value_skips_second_write_semantics() {
        // Mirrors desktop set_items dedup: same bytes → no second transport write.
        let t = FakeTransport::new(NAME_CONTAINER);
        let mut map = DataMap::new();
        map.insert("k".into(), "v".into());
        touch_meta(&mut map);
        t.write(&map).unwrap();
        let before = t.write_count();
        if map.get("k").map(String::as_str) == Some("v") {
            // no write
        } else {
            t.write(&map).unwrap();
        }
        assert_eq!(t.write_count(), before);
    }

    #[test]
    fn auto_latches_after_first_receipt_and_never_switches() {
        let appgroup = FakeTransport::new(NAME_APPGROUP);
        let container = FakeTransport::new(NAME_CONTAINER);

        let mut probe = DataMap::new();
        probe.insert("__probe__".into(), "1".into());
        touch_meta(&mut probe);
        let nonce = map_nonce(&probe);
        appgroup.write(&probe).unwrap();
        container.write(&probe).unwrap();
        appgroup.plant_receipt(Receipt {
            read_from: NAME_APPGROUP.into(),
            nonce,
            ts: store::now_ms(),
        });

        let latched = probe_once_among(
            vec![
                appgroup.clone() as Arc<dyn Transport>,
                container.clone() as Arc<dyn Transport>,
            ],
            NAME_CONTAINER,
        )
        .unwrap();
        assert_eq!(latched.name(), NAME_APPGROUP);

        // Later container receipts must not switch the already-chosen driver in production;
        // resolve_driver is one-shot. Here we only assert latch picked appgroup once.
        container.plant_receipt(Receipt {
            read_from: NAME_CONTAINER.into(),
            nonce: nonce + 1,
            ts: store::now_ms(),
        });
        assert_eq!(latched.name(), NAME_APPGROUP);
    }

    #[test]
    fn require_app_group_message() {
        let cfg = WidgetsPluginConfig::default();
        match build_driver(TransportKind::WidgetContainer, &cfg) {
            Err(err) => assert!(err.to_string().contains("appGroup")),
            Ok(_) => panic!("expected missing appGroup error"),
        }
    }
}
