//! Cross-platform render receipts (diagnostics, not a critical path).
//!
//! Renderers write after paint; host reads when it wants. Separate from the
//! config map so receipt writes never bump `__meta_nonce__`.
//!
//! History is a ring per instance (cap 32) so intermittent failures stay
//! visible after the latest upsert.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::store::now_ms;

/// Prefs / file basename for the receipt bag (outside config DataMap).
pub const RECEIPTS_STORE_NAME: &str = "__tauri_widget_receipts__";
/// On-disk filename under the app data / widgets dir.
pub const RECEIPTS_FILE_NAME: &str = "widget_receipts.json";
const HISTORY_CAP: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Element the renderer could not paint (capability / parse).
pub struct SkippedElement {
    /// IR type name (`chart`, `canvas`, …).
    #[serde(rename = "type")]
    pub type_name: String,
    /// Why it was skipped.
    pub reason: String,
}

/// What a renderer actually painted for one widget instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetRenderReceipt {
    /// Logical widget id from JS / IR.
    pub widget_id: String,
    /// App Group / prefs group.
    pub group: String,
    /// `appWidgetId` | WidgetFamily | window label
    pub instance: String,
    /// Config-map nonce that was rendered (0 if unknown).
    pub nonce: u64,
    /// Family / slot size label when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Light / dark when reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    /// IR schema version the renderer understands.
    #[serde(default = "default_schema")]
    pub schema: u32,
    /// `prefs` | `state` | `appgroup` | `defaults` | `container` | `push` | `pull`
    pub source: String,
    /// Why this paint ran: `reload` | `timeline` | `action` | `added` | `resize` | `snapshot`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    /// IR type names that painted successfully.
    #[serde(default)]
    pub rendered: Vec<String>,
    /// Elements skipped with reasons.
    #[serde(default)]
    pub skipped: Vec<SkippedElement>,
    /// Unix ms when this receipt was written.
    pub ts: u64,
}

fn default_schema() -> u32 {
    1
}

impl WidgetRenderReceipt {
    /// Fill `ts` with [`now_ms`] if still zero.
    pub fn touch_ts(mut self) -> Self {
        if self.ts == 0 {
            self.ts = now_ms();
        }
        self
    }
}

/// In-memory + optional disk bag of receipts, keyed by group → instance → history.
#[derive(Default)]
pub struct ReceiptStore {
    by_group: Mutex<HashMap<String, HashMap<String, VecDeque<WidgetRenderReceipt>>>>,
}

impl ReceiptStore {
    /// Empty in-memory store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a receipt to the per-instance history ring.
    pub fn upsert(&self, receipt: WidgetRenderReceipt) {
        let receipt = receipt.touch_ts();
        let mut guard = self.by_group.lock().unwrap();
        let map = guard.entry(receipt.group.clone()).or_default();
        let q = map.entry(receipt.instance.clone()).or_default();
        q.push_back(receipt);
        while q.len() > HISTORY_CAP {
            q.pop_front();
        }
    }

    /// Latest receipt per instance (newest first).
    pub fn list(&self, group: &str) -> Vec<WidgetRenderReceipt> {
        self.by_group
            .lock()
            .unwrap()
            .get(group)
            .map(|m| {
                let mut v: Vec<_> = m.values().filter_map(|q| q.back().cloned()).collect();
                v.sort_by_key(|r| std::cmp::Reverse(r.ts));
                v
            })
            .unwrap_or_default()
    }

    /// Full history for a group (oldest → newest), capped by ring.
    pub fn history(&self, group: &str) -> Vec<WidgetRenderReceipt> {
        self.by_group
            .lock()
            .unwrap()
            .get(group)
            .map(|m| {
                let mut v: Vec<_> = m.values().flat_map(|q| q.iter().cloned()).collect();
                v.sort_by_key(|r| r.ts);
                v
            })
            .unwrap_or_default()
    }

    /// Instance ids with a recent receipt for `widget_id` (age ≤ `max_age_ms`).
    pub fn live_instances(&self, group: &str, widget_id: &str, max_age_ms: u64) -> Vec<String> {
        let now = now_ms();
        self.list(group)
            .into_iter()
            .filter(|r| r.widget_id == widget_id && now.saturating_sub(r.ts) <= max_age_ms)
            .map(|r| r.instance)
            .collect()
    }

    /// Persist all groups to JSON at `path`.
    pub fn save_to_path(&self, path: &Path) -> crate::Result<()> {
        let all: HashMap<String, Vec<WidgetRenderReceipt>> = self
            .by_group
            .lock()
            .unwrap()
            .iter()
            .map(|(g, m)| {
                let mut list: Vec<_> = m.values().flat_map(|q| q.iter().cloned()).collect();
                list.sort_by_key(|r| r.ts);
                (g.clone(), list)
            })
            .collect();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&all)?;
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, json.as_bytes())?;
        fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Load receipts from disk (best-effort; ignores corrupt files).
    pub fn load_from_path(&self, path: &Path) {
        let Ok(raw) = fs::read_to_string(path) else {
            return;
        };
        let Ok(all): Result<HashMap<String, Vec<WidgetRenderReceipt>>, _> =
            serde_json::from_str(&raw)
        else {
            return;
        };
        for (_group, list) in all {
            for r in list {
                self.upsert(r);
            }
        }
    }
}

/// Default disk path under an app data dir.
pub fn receipts_path(app_data: &Path) -> PathBuf {
    app_data.join("widgets").join(RECEIPTS_FILE_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest_state_machine::{prop_state_machine, ReferenceStateMachine, StateMachineTest};
    use std::collections::BTreeMap;

    const GROUPS: [&str; 2] = ["group.a", "group.b"];
    const INSTANCES: [&str; 2] = ["1", "2"];

    fn sample(group: &str, instance: &str, nonce: u64, ts: u64) -> WidgetRenderReceipt {
        WidgetRenderReceipt {
            widget_id: "weather".into(),
            group: group.into(),
            instance: instance.into(),
            nonce,
            size: Some("small".into()),
            theme: None,
            schema: 1,
            source: "prefs".into(),
            trigger: Some("timeline".into()),
            rendered: vec!["text".into()],
            skipped: vec![],
            ts,
        }
    }

    #[derive(Clone, Debug)]
    enum Op {
        /// A renderer reports a paint (ts strictly increasing, so ordering is total).
        Upsert {
            group: usize,
            instance: usize,
            nonce: u64,
            ts: u64,
        },
        /// Persist, then load into a fresh store (app restart).
        SaveAndRestart,
        /// A corrupt file on disk is ignored by load.
        LoadCorrupt,
    }

    /// Model: (group, instance) → history of (nonce, ts), oldest first, capped.
    #[derive(Clone, Debug, Default)]
    struct Model {
        rings: BTreeMap<(usize, usize), Vec<(u64, u64)>>,
        next_ts: u64,
    }

    impl ReferenceStateMachine for Model {
        type State = Model;
        type Transition = Op;

        fn init_state() -> BoxedStrategy<Model> {
            Just(Model {
                next_ts: 1,
                ..Default::default()
            })
            .boxed()
        }

        fn transitions(state: &Model) -> BoxedStrategy<Op> {
            let ts = state.next_ts;
            prop_oneof![
                // Most upserts hit one placement, so its ring passes the cap of 32 within
                // ~55 steps (uniform over 4 placements needed ~150).
                10 => (prop_oneof![8 => Just((0, 0)), 3 => (0..GROUPS.len(), 0..INSTANCES.len())], 0u64..1000)
                    .prop_map(move |((group, instance), nonce)| Op::Upsert { group, instance, nonce, ts }),
                1 => Just(Op::SaveAndRestart),
                1 => Just(Op::LoadCorrupt),
            ]
            .boxed()
        }

        fn apply(mut state: Model, op: &Op) -> Model {
            if let Op::Upsert {
                group,
                instance,
                nonce,
                ts,
            } = *op
            {
                let ring = state.rings.entry((group, instance)).or_default();
                ring.push((nonce, ts));
                if ring.len() > HISTORY_CAP {
                    ring.remove(0);
                }
                state.next_ts = ts + 1;
            }
            state
        }
    }

    struct Sut {
        store: ReceiptStore,
        dir: tempfile::TempDir,
    }

    struct ReceiptMachine;

    impl StateMachineTest for ReceiptMachine {
        type SystemUnderTest = Sut;
        type Reference = Model;

        fn init_test(_: &Model) -> Sut {
            Sut {
                store: ReceiptStore::new(),
                dir: tempfile::tempdir().unwrap(),
            }
        }

        fn apply(mut sut: Sut, _: &Model, op: Op) -> Sut {
            let path = receipts_path(sut.dir.path());
            match op {
                Op::Upsert {
                    group,
                    instance,
                    nonce,
                    ts,
                } => sut
                    .store
                    .upsert(sample(GROUPS[group], INSTANCES[instance], nonce, ts)),
                Op::SaveAndRestart => {
                    sut.store.save_to_path(&path).unwrap();
                    assert!(
                        !path.with_extension("tmp").exists(),
                        "temp file left behind"
                    );
                    let fresh = ReceiptStore::new();
                    fresh.load_from_path(&path);
                    sut.store = fresh;
                }
                Op::LoadCorrupt => {
                    let bad = sut.dir.path().join("corrupt.json");
                    fs::write(&bad, "{not json").unwrap();
                    sut.store.load_from_path(&bad);
                    sut.store
                        .load_from_path(&sut.dir.path().join("missing.json"));
                }
            }
            sut
        }

        fn check_invariants(sut: &Sut, model: &Model) {
            for (g, group) in GROUPS.iter().enumerate() {
                // history: every kept paint of the group, oldest → newest.
                let mut want: Vec<(u64, u64)> = model
                    .rings
                    .iter()
                    .filter(|((mg, _), _)| *mg == g)
                    .flat_map(|(_, r)| r.iter().copied())
                    .collect();
                want.sort_by_key(|&(_, ts)| ts);
                let got: Vec<(u64, u64)> = sut
                    .store
                    .history(group)
                    .iter()
                    .map(|r| (r.nonce, r.ts))
                    .collect();
                assert_eq!(got, want, "history({group})");
                // list: latest paint per instance, newest first.
                let mut latest: Vec<(String, u64, u64)> = model
                    .rings
                    .iter()
                    .filter(|((mg, _), _)| *mg == g)
                    .filter_map(|((_, i), r)| {
                        r.last().map(|&(n, ts)| (INSTANCES[*i].to_string(), n, ts))
                    })
                    .collect();
                latest.sort_by_key(|&(_, _, ts)| std::cmp::Reverse(ts));
                let got: Vec<(String, u64, u64)> = sut
                    .store
                    .list(group)
                    .into_iter()
                    .map(|r| (r.instance, r.nonce, r.ts))
                    .collect();
                assert_eq!(got, latest, "list({group})");
            }
            assert!(sut.store.list("group.none").is_empty());
        }
    }

    prop_state_machine! {
        #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]
        /// Receipt bag vs a model: per-instance rings capped at 32, `list` = latest per
        /// instance newest first, `history` = all oldest first, restart via disk keeps both,
        /// corrupt / missing files are ignored.
        #[test]
        fn receipt_store_follows_the_model(sequential 1..80 => ReceiptMachine);
    }

    /// Liveness: a receipt without `ts` is stamped now → `live_instances` keeps recent
    /// paints of that widget only (old paints and other widgets are dropped).
    #[test]
    fn untimed_receipts_are_stamped_and_count_as_live() {
        let store = ReceiptStore::new();
        store.upsert(sample("g", "fresh", 1, 0));
        store.upsert(sample("g", "stale", 2, 1));
        let mut other = sample("g", "other", 3, 0);
        other.widget_id = "clock".into();
        store.upsert(other);
        let fresh = store
            .list("g")
            .into_iter()
            .find(|r| r.instance == "fresh")
            .unwrap();
        assert!(
            now_ms().saturating_sub(fresh.ts) < 60_000,
            "ts {}",
            fresh.ts
        );
        assert_eq!(store.live_instances("g", "weather", 60_000), ["fresh"]);
        assert_eq!(sample("g", "x", 0, 7).touch_ts().ts, 7);
    }
}
