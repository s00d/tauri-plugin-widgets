//! Apple host→widget transport: one driver, chosen by config (not runtime fan-out).
//!
//! Availability (`containerURL`, suite exists) is **not** delivery proof — the
//! developer picks [`crate::config::TransportKind`] from signing knowledge.

use crate::config::{effective_transport, TransportKind, WidgetsPluginConfig};
use crate::error::Error;
#[cfg(any(target_os = "macos", test))]
use crate::store::map_nonce;
#[cfg(test)]
use crate::store::touch_meta;
use crate::store::{self, DataMap};
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
    use crate::config::tests::ENV_LOCK;
    use crate::config::WidgetsPluginConfig;

    fn cfg(kind: Option<TransportKind>, group: Option<&str>) -> WidgetsPluginConfig {
        WidgetsPluginConfig {
            transport: kind,
            app_group: group.map(Into::into),
            ..Default::default()
        }
    }

    fn map(nonce: u64, widget: &str) -> DataMap {
        let mut m = DataMap::new();
        m.insert(format!("config:{widget}"), r#"{"version":1}"#.into());
        store::touch_meta_above(&mut m, nonce.saturating_sub(1));
        m
    }

    /// Choosing a driver from config: explicit kinds need an appGroup (blank counts as
    /// missing) → off macOS no Apple driver exists → mobile accepts appGroup / auto only.
    #[test]
    fn driver_choice_from_config() {
        let _env = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("WIDGET_TRANSPORT");
        // 1. appGroup is checked first, for every explicit kind.
        for kind in [
            TransportKind::AppGroup,
            TransportKind::UserDefaults,
            TransportKind::WidgetContainer,
        ] {
            for group in [None, Some("  ")] {
                let err = build_driver(kind, &cfg(Some(kind), group))
                    .err()
                    .expect("no group");
                assert!(err.to_string().contains("appGroup"), "{kind:?}: {err}");
            }
        }
        // 2. Platform gate.
        #[cfg(not(target_os = "macos"))]
        {
            let c = cfg(Some(TransportKind::WidgetContainer), Some("group.t"));
            for err in [
                build_driver(TransportKind::WidgetContainer, &c)
                    .err()
                    .unwrap(),
                resolve_driver(&c).err().unwrap(),
            ] {
                assert!(err.to_string().contains("only available on macOS"), "{err}");
            }
        }
        #[cfg(target_os = "macos")]
        {
            let err = build_driver(TransportKind::Auto, &cfg(None, Some("group.t")))
                .err()
                .unwrap();
            assert!(err.to_string().contains("invalid"), "{err}");
        }
        // 3. Mobile (iOS rules off Android).
        #[cfg(not(target_os = "android"))]
        {
            for kind in [TransportKind::AppGroup, TransportKind::Auto] {
                assert_eq!(
                    validate_mobile_transport(&cfg(Some(kind), Some("group.t"))).unwrap(),
                    TransportKind::AppGroup
                );
            }
            let err =
                validate_mobile_transport(&cfg(Some(TransportKind::AppGroup), None)).unwrap_err();
            assert!(err.to_string().contains("appGroup is required"), "{err}");
            for kind in [TransportKind::UserDefaults, TransportKind::WidgetContainer] {
                let err = validate_mobile_transport(&cfg(Some(kind), Some("group.t"))).unwrap_err();
                assert!(err.to_string().contains("appGroup only"), "{err}");
                assert!(err.to_string().contains(kind.as_str()), "{err}");
            }
        }
    }

    /// File drivers on a macOS host (temp container root): container driver write → read →
    /// receipt → a newer write wins → concurrent writers leave one whole map and no temp
    /// files → a corrupt file reads as nothing and the next write recovers; appGroup fails
    /// loudly without a container and works with the test override.
    #[cfg(target_os = "macos")]
    #[test]
    fn file_driver_write_read_concurrent_and_recovery() {
        let _env = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("WIDGET_TRANSPORT");
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("WIDGET_CONTAINER_ROOT", tmp.path());
        std::env::remove_var("WIDGET_APP_GROUP_DATA_FILE");
        let mut c = cfg(
            Some(TransportKind::WidgetContainer),
            Some("group.test.driver"),
        );
        c.extension_bundle_id = Some("test.driver.widgetkit".into());

        // 1. Explicit container driver; bundle id from config reaches the path helpers.
        let t = resolve_driver(&c).unwrap();
        assert_eq!(t.name(), NAME_CONTAINER);
        assert!(t.available());
        let path = crate::macos_transport::sandbox_widget_data_path("group.test.driver");
        assert!(
            path.starts_with(tmp.path())
                && path.to_string_lossy().contains("test.driver.widgetkit")
        );
        assert!(t.read().is_none());

        // 2. Write → read → receipt.
        let v1 = map(1, "w");
        t.write(&v1).unwrap();
        assert_eq!(t.read(), Some(v1.clone()));
        let r = Receipt {
            read_from: NAME_CONTAINER.into(),
            nonce: 1,
            ts: 5,
        };
        t.write_receipt(&r).unwrap();
        assert_eq!(t.read_receipt(), Some(r));
        let v2 = map(2, "w");
        t.write(&v2).unwrap();
        assert_eq!(store::map_nonce(&t.read().unwrap()), 2);

        // 3. Concurrent writers: atomic rename keeps one complete map, no temp leftovers.
        let writers: Vec<_> = (10..18u64)
            .map(|n| {
                let t = Arc::clone(&t);
                std::thread::spawn(move || {
                    for k in 0..5 {
                        t.write(&map(n * 10 + k, "w")).unwrap();
                    }
                })
            })
            .collect();
        writers.into_iter().for_each(|h| h.join().unwrap());
        let last = t.read().expect("whole map after concurrent writes");
        assert!((100..180).contains(&store::map_nonce(&last)), "{last:?}");
        let leftovers: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp."))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");

        // 4. Corruption → nothing read → next write recovers.
        std::fs::write(&path, "{truncated").unwrap();
        assert!(t.read().is_none());
        t.write(&v1).unwrap();
        assert_eq!(t.read(), Some(v1));

        // 5. appGroup: loud error without a container (some macOS installs return one),
        //    a working file driver with the override.
        match build_driver(
            TransportKind::AppGroup,
            &cfg(None, Some("group.test.missing")),
        ) {
            Err(err) => assert!(err.to_string().contains("App Group"), "{err}"),
            Ok(_) => eprintln!("skip: App Group container available for group.test.missing"),
        }
        let file = tmp.path().join("ag/widget_data.json");
        std::env::set_var("WIDGET_APP_GROUP_DATA_FILE", &file);
        let ag = build_driver(
            TransportKind::AppGroup,
            &cfg(None, Some("group.test.driver")),
        )
        .unwrap();
        assert_eq!(ag.name(), NAME_APPGROUP);
        ag.write(&v2).unwrap();
        assert!(file.exists());
        assert_eq!(ag.read(), Some(v2));

        for k in [
            "WIDGET_CONTAINER_ROOT",
            "WIDGET_APP_GROUP_DATA_FILE",
            "WIDGET_EXTENSION_BUNDLE",
        ] {
            std::env::remove_var(k);
        }
    }

    /// `auto` probe latch rules over fakes: every candidate gets the probe once → the one
    /// whose own receipt acks the probe nonce wins → mismatched name / stale nonce do not
    /// count → no ack falls back → a missing fallback is an error. A later receipt on
    /// another transport does not change an already latched driver.
    #[test]
    fn auto_probe_latches_on_matching_receipt_else_fallback() {
        let pair = || {
            (
                FakeTransport::new(NAME_APPGROUP),
                FakeTransport::new(NAME_CONTAINER),
            )
        };
        let dyns = |a: &Arc<FakeTransport>, b: &Arc<FakeTransport>| {
            vec![
                a.clone() as Arc<dyn Transport>,
                b.clone() as Arc<dyn Transport>,
            ]
        };
        let ack = |from: &str, nonce: u64| Receipt {
            read_from: from.into(),
            nonce,
            ts: store::now_ms(),
        };

        // 1. Matching ack (probe nonce is 1 on a fresh map) → appgroup.
        let (ag, ct) = pair();
        ag.plant_receipt(ack(NAME_APPGROUP, 1));
        let latched = probe_once_among(dyns(&ag, &ct), NAME_CONTAINER).unwrap();
        assert_eq!(latched.name(), NAME_APPGROUP);
        assert_eq!((ag.write_count(), ct.write_count()), (1, 1));
        assert_eq!(
            ag.read().unwrap().get("__probe__").map(String::as_str),
            Some("1")
        );
        ct.plant_receipt(ack(NAME_CONTAINER, 2));
        assert_eq!(latched.name(), NAME_APPGROUP);

        // 2. Receipt names another transport, or acks an older nonce → fallback.
        for planted in [ack(NAME_CONTAINER, 1), ack(NAME_APPGROUP, 0)] {
            let (ag, ct) = pair();
            ag.plant_receipt(planted);
            assert_eq!(
                probe_once_among(dyns(&ag, &ct), NAME_CONTAINER)
                    .unwrap()
                    .name(),
                NAME_CONTAINER
            );
        }
        // 3. No receipt and no fallback candidate.
        let (ag, ct) = pair();
        assert!(probe_once_among(dyns(&ag, &ct), NAME_DEFAULTS).is_err());
    }

    /// Fake driver contract used by the host tests, plus the freshest-map pick: writes are
    /// counted per driver, availability toggles, and the higher nonce wins.
    #[test]
    fn fake_driver_contract_and_freshest_pick() {
        let container = FakeTransport::new(NAME_CONTAINER);
        let appgroup = FakeTransport::new(NAME_APPGROUP);
        container.write(&map(1, "w")).unwrap();
        assert_eq!((container.write_count(), appgroup.write_count()), (1, 0));
        assert!(appgroup.read().is_none() && container.read().is_some());
        container.set_available(false);
        assert!(!container.available() && appgroup.available());
        let r = Receipt {
            read_from: NAME_CONTAINER.into(),
            nonce: 1,
            ts: 1,
        };
        appgroup.write_receipt(&r).unwrap();
        assert_eq!(appgroup.read_receipt(), Some(r));
        let picked = pick_freshest_maps(vec![map(3, "old"), map(7, "new"), map(5, "mid")]);
        assert!(picked.contains_key("config:new"), "{picked:?}");
    }
}
