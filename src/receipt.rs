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
pub const RECEIPTS_FILE_NAME: &str = "widget_receipts.json";
const HISTORY_CAP: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedElement {
    #[serde(rename = "type")]
    pub type_name: String,
    pub reason: String,
}

/// What a renderer actually painted for one widget instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetRenderReceipt {
    pub widget_id: String,
    pub group: String,
    /// `appWidgetId` | WidgetFamily | window label
    pub instance: String,
    /// Config-map nonce that was rendered (0 if unknown).
    pub nonce: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
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
    #[serde(default)]
    pub rendered: Vec<String>,
    #[serde(default)]
    pub skipped: Vec<SkippedElement>,
    pub ts: u64,
}

fn default_schema() -> u32 {
    1
}

impl WidgetRenderReceipt {
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
    pub fn new() -> Self {
        Self::default()
    }

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
                let mut v: Vec<_> = m
                    .values()
                    .filter_map(|q| q.back().cloned())
                    .collect();
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

    pub fn live_instances(&self, group: &str, widget_id: &str, max_age_ms: u64) -> Vec<String> {
        let now = now_ms();
        self.list(group)
            .into_iter()
            .filter(|r| r.widget_id == widget_id && now.saturating_sub(r.ts) <= max_age_ms)
            .map(|r| r.instance)
            .collect()
    }

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

    fn sample(nonce: u64, ts: u64) -> WidgetRenderReceipt {
        WidgetRenderReceipt {
            widget_id: "weather".into(),
            group: "group.test".into(),
            instance: "42".into(),
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

    #[test]
    fn upsert_keeps_history() {
        let store = ReceiptStore::new();
        store.upsert(sample(1, 100));
        store.upsert(sample(2, 200));
        let list = store.list("group.test");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].nonce, 2);
        let hist = store.history("group.test");
        assert_eq!(hist.len(), 2);
        assert_eq!(hist[0].nonce, 1);
        assert_eq!(hist[1].nonce, 2);
    }
}
