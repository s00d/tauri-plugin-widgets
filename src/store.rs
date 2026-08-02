//! Shared storage contract for widget data across platforms.
//!
//! Keys (breaking in 0.4 — no legacy `__widget_config__`):
//! - `config:{widgetId}` — serialized [`crate::models::WidgetConfig`]
//! - `pending_actions` — JSON array of [`WidgetActionEnvelope`]
//! - `__meta_nonce__` / `__meta_updated_at__` — freshness for multi-transport pick

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Key prefix for per-widget UI configs.
pub const CONFIG_KEY_PREFIX: &str = "config:";
/// Queue of actions emitted by native widgets for the host to consume.
pub const PENDING_ACTIONS_KEY: &str = "pending_actions";
/// Monotonic freshness counter written with every map mutation.
pub const META_NONCE_KEY: &str = "__meta_nonce__";
/// Unix epoch milliseconds of last map write.
pub const META_UPDATED_AT_KEY: &str = "__meta_updated_at__";

pub type DataMap = HashMap<String, String>;

/// Build the storage key for a widget UI config.
pub fn config_key(widget_id: &str) -> String {
    format!("{CONFIG_KEY_PREFIX}{widget_id}")
}

/// Current time as Unix ms.
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Read nonce from a data map (0 if missing/invalid).
pub fn map_nonce(map: &DataMap) -> u64 {
    map.get(META_NONCE_KEY)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Read updatedAt from a data map (0 if missing/invalid).
pub fn map_updated_at(map: &DataMap) -> u64 {
    map.get(META_UPDATED_AT_KEY)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Bump nonce + updatedAt on the map (call before persisting).
pub fn touch_meta(map: &mut DataMap) {
    let next = map_nonce(map).saturating_add(1);
    map.insert(META_NONCE_KEY.into(), next.to_string());
    map.insert(META_UPDATED_AT_KEY.into(), now_ms().to_string());
}

/// Pick the freshest map among candidates (max nonce, then max updatedAt).
pub fn pick_freshest(maps: impl IntoIterator<Item = DataMap>) -> DataMap {
    let mut best: Option<DataMap> = None;
    let mut best_nonce = 0u64;
    let mut best_ts = 0u64;
    for map in maps {
        let n = map_nonce(&map);
        let t = map_updated_at(&map);
        let better = best.is_none() || n > best_nonce || (n == best_nonce && t > best_ts);
        if better {
            best_nonce = n;
            best_ts = t;
            best = Some(map);
        }
    }
    best.unwrap_or_default()
}

/// Action delivered from a native widget back to the host app.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetActionEnvelope {
    pub action: String,
    #[serde(default)]
    pub payload: Option<String>,
    /// Unix epoch milliseconds when the action was enqueued.
    pub ts: u64,
    pub widget_id: String,
    pub group: String,
}

impl WidgetActionEnvelope {
    pub fn new(
        action: impl Into<String>,
        payload: Option<String>,
        widget_id: impl Into<String>,
        group: impl Into<String>,
    ) -> Self {
        Self {
            action: action.into(),
            payload,
            ts: now_ms(),
            widget_id: widget_id.into(),
            group: group.into(),
        }
    }
}

/// Parse pending actions JSON; empty on missing/invalid.
pub fn parse_pending_actions(raw: Option<&str>) -> Vec<WidgetActionEnvelope> {
    let Some(s) = raw.filter(|s| !s.is_empty()) else {
        return Vec::new();
    };
    serde_json::from_str(s).unwrap_or_default()
}

/// Serialize pending actions for storage.
pub fn encode_pending_actions(actions: &[WidgetActionEnvelope]) -> crate::Result<String> {
    Ok(serde_json::to_string(actions)?)
}
