//! Apple multi-transport: fan-out → receipt → narrow → self-heal.
//!
//! Config **writes** may narrow to a confirmed transport. **Reads** and receipts
//! always fan out. Receipts live outside the config map so they never bump
//! `__meta_nonce__`.
//!
//! Availability (`containerURL`, suite exists) is **not** delivery proof — only
//! a widget-side receipt with matching nonce confirms a channel.

use crate::store::{self, map_nonce, DataMap};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub const NAME_APPGROUP: &str = "appgroup";
pub const NAME_DEFAULTS: &str = "defaults";
pub const NAME_CONTAINER: &str = "container";

pub const CONFIRM_STREAK: u32 = 3;
pub const STALE_AFTER_MS: u64 = 5 * 60_000;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    Unknown,
    Confirmed { streak: u32 },
    Stale { since_ms: u64 },
}

impl Health {
    pub fn tag(self) -> &'static str {
        match self {
            Health::Unknown => "unknown",
            Health::Confirmed { .. } => "confirmed",
            Health::Stale { .. } => "stale",
        }
    }
}

pub trait Transport: Send + Sync {
    fn name(&self) -> &'static str;
    /// Cheap local probe — must NOT be treated as delivery proof.
    fn available(&self) -> bool;
    fn read(&self) -> Option<DataMap>;
    fn write(&self, map: &DataMap) -> crate::Result<()>;
    fn read_receipt(&self) -> Option<Receipt>;
    fn write_receipt(&self, receipt: &Receipt) -> crate::Result<()>;
}

type Clock = Arc<Mutex<Option<u64>>>;

pub struct TransportSet {
    transports: Vec<Arc<dyn Transport>>,
    /// Kept so tests can wipe receipts without downcasting dyn Transport.
    fakes: Vec<Arc<FakeTransport>>,
    health: Mutex<HashMap<&'static str, Health>>,
    last_written_nonce: Mutex<u64>,
    clock: Clock,
    /// `(bundle_version, team_id_hash)` — mismatch resets health.
    install: Mutex<(String, String)>,
    /// Test-only: names that auto-plant matching receipts after each write.
    auto_ack: Mutex<Vec<&'static str>>,
}

impl TransportSet {
    pub fn new(transports: Vec<Arc<dyn Transport>>) -> Self {
        Self {
            transports,
            fakes: Vec::new(),
            health: Mutex::new(HashMap::new()),
            last_written_nonce: Mutex::new(0),
            clock: Arc::new(Mutex::new(None)),
            install: Mutex::new((String::new(), String::new())),
            auto_ack: Mutex::new(Vec::new()),
        }
    }

    /// In-memory transports for Level C unit tests (no FS / no widget process).
    pub fn fake(names: &[&'static str]) -> Self {
        let clock = Arc::new(Mutex::new(Some(1_000)));
        let mut fakes = Vec::new();
        let mut transports = Vec::new();
        for n in names {
            let fake = Arc::new(FakeTransport {
                name: n,
                available: AtomicBool::new(true),
                map: Mutex::new(None),
                receipt: Mutex::new(None),
            });
            fakes.push(Arc::clone(&fake));
            transports.push(fake as Arc<dyn Transport>);
        }
        Self {
            transports,
            fakes,
            health: Mutex::new(HashMap::new()),
            last_written_nonce: Mutex::new(0),
            clock,
            install: Mutex::new((String::new(), String::new())),
            auto_ack: Mutex::new(Vec::new()),
        }
    }

    pub fn now_ms(&self) -> u64 {
        self.clock
            .lock()
            .unwrap()
            .unwrap_or_else(store::now_ms)
    }

    pub fn set_clock_ms(&self, ms: u64) {
        *self.clock.lock().unwrap() = Some(ms);
    }

    pub fn advance_clock_ms(&self, delta: u64) {
        let mut c = self.clock.lock().unwrap();
        let base = c.unwrap_or_else(store::now_ms);
        *c = Some(base.saturating_add(delta));
    }

    /// Reset health when signing / version identity changes.
    pub fn invalidate_if_install_changed(&self, bundle_version: &str, team_id_hash: &str) {
        let mut inst = self.install.lock().unwrap();
        if inst.0.is_empty() && inst.1.is_empty() {
            *inst = (bundle_version.to_string(), team_id_hash.to_string());
            return;
        }
        if inst.0 != bundle_version || inst.1 != team_id_hash {
            *inst = (bundle_version.to_string(), team_id_hash.to_string());
            let mut health = self.health.lock().unwrap();
            for h in health.values_mut() {
                *h = Health::Unknown;
            }
            log::debug!("transport install identity changed — health reset to fan-out");
        }
    }

    /// Where config should be written right now (narrowed or full fan-out).
    pub fn writers(&self) -> Vec<&dyn Transport> {
        let health = self.health.lock().unwrap();
        let winner = self.transports.iter().find(|t| {
            matches!(
                health.get(t.name()),
                Some(Health::Confirmed { streak }) if *streak >= CONFIRM_STREAK
            )
        });
        match winner {
            Some(t) => vec![t.as_ref()],
            None => self
                .transports
                .iter()
                .filter(|t| t.available())
                .map(|t| t.as_ref())
                .collect(),
        }
    }

    pub fn writer_names(&self) -> Vec<&'static str> {
        self.writers().into_iter().map(|t| t.name()).collect()
    }

    pub fn health_of(&self, name: &str) -> Health {
        self.health
            .lock()
            .unwrap()
            .get(name)
            .copied()
            .unwrap_or(Health::Unknown)
    }

    /// Read config maps from every available transport (never narrowed).
    pub fn read_all(&self) -> Vec<DataMap> {
        self.transports
            .iter()
            .filter(|t| t.available())
            .filter_map(|t| t.read())
            .collect()
    }

    pub fn pick_freshest_map(&self) -> DataMap {
        store::pick_freshest(self.read_all())
    }

    /// Write config via current writers(); update expected nonce; optional test auto-ack.
    pub fn write(&self, map: &DataMap) -> crate::Result<()> {
        let nonce = map_nonce(map);
        *self.last_written_nonce.lock().unwrap() = nonce;

        for t in self.writers() {
            t.write(map)?;
        }

        let acks = self.auto_ack.lock().unwrap().clone();
        let ts = self.now_ms();
        for t in &self.transports {
            if acks.iter().any(|n| *n == t.name()) {
                t.write_receipt(&Receipt {
                    read_from: t.name().to_string(),
                    nonce,
                    ts,
                })?;
            }
        }
        Ok(())
    }

    /// Fan-out a receipt to all available transports (widget / tests).
    pub fn write_receipt_everywhere(&self, receipt: &Receipt) -> crate::Result<()> {
        for t in &self.transports {
            if t.available() {
                t.write_receipt(receipt)?;
            }
        }
        Ok(())
    }

    /// Inspect receipts and update health. Call before writes and on a timer.
    pub fn reconcile(&self) {
        let expected = *self.last_written_nonce.lock().unwrap();
        let mut health = self.health.lock().unwrap();
        let now = self.now_ms();

        for t in &self.transports {
            let entry = health.entry(t.name()).or_insert(Health::Unknown);
            match t.read_receipt() {
                // Stale first — an old matching receipt must not keep growing streak.
                Some(r) if now.saturating_sub(r.ts) > STALE_AFTER_MS => {
                    *entry = Health::Stale { since_ms: r.ts };
                }
                Some(r) if r.read_from == t.name() && expected > 0 && r.nonce >= expected => {
                    *entry = match *entry {
                        Health::Confirmed { streak } => Health::Confirmed {
                            streak: streak.saturating_add(1),
                        },
                        _ => Health::Confirmed { streak: 1 },
                    };
                }
                None => {
                    if matches!(*entry, Health::Confirmed { .. }) {
                        *entry = Health::Unknown;
                    }
                }
                _ => {}
            }
        }

        log::debug!(
            "transports: {:?}",
            health
                .iter()
                .map(|(k, v)| (*k, v.tag()))
                .collect::<Vec<_>>()
        );
    }

    /// Confirmations only from `name` (test). Availability alone never narrows.
    pub fn fake_receipts_from(&self, name: &'static str) {
        *self.auto_ack.lock().unwrap() = vec![name];
    }

    /// Stop planting receipts and clear existing ones (test silence).
    pub fn fake_silence(&self) {
        *self.auto_ack.lock().unwrap() = Vec::new();
        for fake in &self.fakes {
            *fake.receipt.lock().unwrap() = None;
        }
    }
}

struct FakeTransport {
    name: &'static str,
    available: AtomicBool,
    map: Mutex<Option<DataMap>>,
    receipt: Mutex<Option<Receipt>>,
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
    use crate::store::touch_meta;

    fn map_v1() -> DataMap {
        let mut m = DataMap::new();
        m.insert("config:probe".into(), r#"{"version":1}"#.into());
        touch_meta(&mut m);
        m
    }

    fn names(set: &TransportSet) -> Vec<&'static str> {
        set.writer_names()
    }

    #[test]
    fn narrows_after_three_confirmations_and_recovers() {
        let set = TransportSet::fake(&[NAME_APPGROUP, NAME_DEFAULTS, NAME_CONTAINER]);
        set.fake_receipts_from(NAME_CONTAINER);

        for _ in 0..CONFIRM_STREAK {
            set.write(&map_v1()).unwrap();
            set.reconcile();
        }
        assert_eq!(
            names(&set),
            vec![NAME_CONTAINER],
            "should narrow onto the working transport"
        );

        set.fake_silence();
        set.reconcile();
        assert_eq!(
            names(&set).len(),
            3,
            "should return to fan-out after silence"
        );

        // Stale path: re-narrow, leave receipts, age past STALE_AFTER_MS
        set.fake_receipts_from(NAME_CONTAINER);
        for _ in 0..CONFIRM_STREAK {
            set.write(&map_v1()).unwrap();
            set.reconcile();
        }
        assert_eq!(names(&set), vec![NAME_CONTAINER]);
        *set.auto_ack.lock().unwrap() = Vec::new();
        set.advance_clock_ms(STALE_AFTER_MS + 1);
        set.reconcile();
        assert_eq!(
            names(&set).len(),
            3,
            "should return to fan-out after stale receipts"
        );
    }

    #[test]
    fn never_narrows_on_local_availability_alone() {
        let set = TransportSet::fake(&[NAME_APPGROUP]);
        for _ in 0..10 {
            set.write(&map_v1()).unwrap();
            set.reconcile();
        }
        assert!(
            matches!(set.health_of(NAME_APPGROUP), Health::Unknown),
            "availability alone must not count as delivery (ad-hoc App Group trap)"
        );
    }

    #[test]
    fn install_identity_change_resets_narrowing() {
        let set = TransportSet::fake(&[NAME_APPGROUP, NAME_CONTAINER]);
        set.invalidate_if_install_changed("1.0", "teamA");
        set.fake_receipts_from(NAME_CONTAINER);
        for _ in 0..CONFIRM_STREAK {
            set.write(&map_v1()).unwrap();
            set.reconcile();
        }
        assert_eq!(names(&set), vec![NAME_CONTAINER]);
        set.invalidate_if_install_changed("1.1", "teamA");
        assert_eq!(names(&set).len(), 2, "version bump must re-open fan-out");
    }

    #[test]
    fn receipt_nonce_must_match_last_write() {
        let set = TransportSet::fake(&[NAME_CONTAINER]);
        let m = map_v1();
        set.write(&m).unwrap();
        // Wrong nonce via auto-ack off + manual plant on the only fake
        set.fakes[0]
            .write_receipt(&Receipt {
                read_from: NAME_CONTAINER.into(),
                nonce: 0,
                ts: set.now_ms(),
            })
            .unwrap();
        set.reconcile();
        assert!(matches!(set.health_of(NAME_CONTAINER), Health::Unknown));

        set.fakes[0]
            .write_receipt(&Receipt {
                read_from: NAME_CONTAINER.into(),
                nonce: map_nonce(&m),
                ts: set.now_ms(),
            })
            .unwrap();
        set.reconcile();
        assert!(matches!(
            set.health_of(NAME_CONTAINER),
            Health::Confirmed { streak: 1 }
        ));
    }
}
