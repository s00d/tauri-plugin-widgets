//! Shared storage contract for widget data across platforms.
//!
//! Keys (breaking in 0.4 — no legacy `__widget_config__`):
//! - `config:{widgetId}` — serialized [`crate::models::WidgetConfig`]
//! - `config:{widgetId}#{instance}` — config of one placement (falls back to `config:{widgetId}`)
//! - `data:{widgetId}` / `data:{widgetId}#{instance}` — values for `{{data.…}}` (`setWidgetData`)
//! - `state:{widgetId}` / `state:{widgetId}#{instance}` — local state written by `set` taps
//!   and `setWidgetState`, merged over the config's `state` (widget key, then instance key)
//! - `activity-template:{name}` — Live Activity layout (`registerActivityTemplate`)
//! - `fetched:{widgetId}` / `fetched:{widgetId}#{instance}` — last `source` response (renderer-owned)
//! - `instances:{widgetId}` — `[{id, title}]` the Apple Edit Widget sheet offers
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

/// String key/value bag persisted by transports.
pub type DataMap = HashMap<String, String>;

/// Build the storage key for a widget UI config.
pub fn config_key(widget_id: &str) -> String {
    format!("{CONFIG_KEY_PREFIX}{widget_id}")
}

/// Separator between widget id and placement id in store keys. Not in the widget id
/// charset, so `a#b` can never be another widget's id.
pub const INSTANCE_SEPARATOR: char = '#';

/// `widgetId` or `widgetId#instance`: the id a placement's keys are built from.
pub fn scoped_id(widget_id: &str, instance: Option<&str>) -> String {
    match instance.filter(|i| !i.is_empty()) {
        Some(i) => format!("{widget_id}{INSTANCE_SEPARATOR}{i}"),
        None => widget_id.to_string(),
    }
}

/// Key prefix of host-set data for `{{data.…}}`.
pub const DATA_KEY_PREFIX: &str = "data:";
/// Key prefix of the cached `source` response a renderer fetched.
pub const FETCHED_KEY_PREFIX: &str = "fetched:";
/// Key prefix of the instance choices offered in the Apple Edit Widget sheet.
pub const INSTANCES_KEY_PREFIX: &str = "instances:";

/// `data:{scoped id}`.
pub fn data_key(widget_id: &str, instance: Option<&str>) -> String {
    format!("{DATA_KEY_PREFIX}{}", scoped_id(widget_id, instance))
}

/// Key prefix of a placement's local state (`set`, `setWidgetState`).
pub const STATE_KEY_PREFIX: &str = "state:";
/// Key prefix of a stored Live Activity template.
pub const ACTIVITY_TEMPLATE_KEY_PREFIX: &str = "activity-template:";

/// Pending-action name a native widget queues after a `set` tap; payload
/// `{widgetId, instance, state}`. The host emits it as `widget-state-changed`.
pub const STATE_ACTION: &str = "$state";

/// `widget-state-changed` payload `{group, widgetId, instance, state}` for a
/// [`STATE_ACTION`] envelope; `None` for any other action.
pub fn state_event(env: &WidgetActionEnvelope) -> Option<serde_json::Value> {
    use serde_json::Value;
    if env.action != STATE_ACTION {
        return None;
    }
    let p: Value = env
        .payload
        .as_deref()
        .and_then(|p| serde_json::from_str(p).ok())
        .unwrap_or_default();
    Some(serde_json::json!({
        "group": env.group,
        "widgetId": p.get("widgetId").cloned().unwrap_or(Value::String(env.widget_id.clone())),
        "instance": p.get("instance").cloned().unwrap_or(Value::Null),
        "state": p.get("state").cloned().unwrap_or(Value::Null),
    }))
}

/// `state:{scoped id}`.
pub fn state_key(widget_id: &str, instance: Option<&str>) -> String {
    format!("{STATE_KEY_PREFIX}{}", scoped_id(widget_id, instance))
}

/// `activity-template:{name}`.
pub fn activity_template_key(name: &str) -> String {
    format!("{ACTIVITY_TEMPLATE_KEY_PREFIX}{name}")
}

/// A placement's state: config `state` defaults, then the widget's stored layer, then the
/// placement's own (later wins per top-level key). Non-object layers are ignored.
pub fn merged_state(
    layers: &[Option<&serde_json::Value>],
) -> serde_json::Map<String, serde_json::Value> {
    let mut out = serde_json::Map::new();
    for layer in layers.iter().flatten() {
        if let Some(m) = layer.as_object() {
            out.extend(m.iter().map(|(k, v)| (k.clone(), v.clone())));
        }
    }
    out
}

/// Apply a `set` / `setWidgetState` patch to the stored `layer`. Operators read the
/// placement's merged state `current`: `"$toggle"` flips a boolean (missing → `true`),
/// `"$inc"` / `"$dec"` add / subtract 1 (missing → 0 first). `null` removes the key from
/// the layer (the default shows again); any other value is stored as is.
pub fn apply_state_patch(
    layer: &mut serde_json::Map<String, serde_json::Value>,
    current: &serde_json::Map<String, serde_json::Value>,
    patch: &serde_json::Map<String, serde_json::Value>,
) {
    use serde_json::Value;
    for (k, v) in patch {
        let now = current.get(k);
        let step = |d: i64| -> Value {
            match now {
                // Integers stay integers while they fit `i64`; floats and overflow add as f64.
                Some(Value::Number(n)) => n
                    .as_i64()
                    .and_then(|i| i.checked_add(d))
                    .map(Value::from)
                    .unwrap_or_else(|| Value::from(n.as_f64().unwrap_or(0.0) + d as f64)),
                _ => Value::from(d),
            }
        };
        let next = match v {
            Value::Null => {
                layer.remove(k);
                continue;
            }
            Value::String(op) if op == "$toggle" => {
                Value::Bool(!matches!(now, Some(Value::Bool(true))))
            }
            Value::String(op) if op == "$inc" => step(1),
            Value::String(op) if op == "$dec" => step(-1),
            other => other.clone(),
        };
        layer.insert(k.clone(), next);
    }
}

/// `instances:{widgetId}`.
pub fn instances_key(widget_id: &str) -> String {
    format!("{INSTANCES_KEY_PREFIX}{widget_id}")
}

/// Keys a renderer reads for one placement, most specific first: the placement's own,
/// then the widget's.
pub fn lookup_keys(prefix: &str, widget_id: &str, instance: Option<&str>) -> Vec<String> {
    let mut keys = Vec::with_capacity(2);
    if instance.is_some_and(|i| !i.is_empty()) {
        keys.push(format!("{prefix}{}", scoped_id(widget_id, instance)));
    }
    keys.push(format!("{prefix}{widget_id}"));
    keys
}

/// One choice in the Apple Edit Widget sheet (and a label for an Android placement).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct WidgetInstanceOption {
    /// Placement id (`{{instance}}`, `config:{widgetId}#{id}`).
    pub id: String,
    /// Text shown to the user.
    pub title: String,
}

/// A widget currently placed on a home screen, desktop or board.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveWidget {
    /// Logical widget id (WidgetKit kind, `widgetId` of the config).
    pub widget_id: String,
    /// Placement id: Android `appWidgetId`, Apple Edit Widget choice, window label,
    /// Widgets Board id. `None` when the placement has no own identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    /// `small` | `medium` | `large` when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Unix ms of the last render, when the list comes from render receipts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen_ms: Option<u64>,
}

/// Live placements first (open windows, native lists), then receipt-only ones not already
/// listed; a receipt's `lastSeenMs` / `size` fill gaps of the live entry.
pub fn merge_active(live: Vec<ActiveWidget>, seen: Vec<ActiveWidget>) -> Vec<ActiveWidget> {
    let mut out = live;
    for s in seen {
        match out
            .iter_mut()
            .find(|a| a.widget_id == s.widget_id && a.instance == s.instance)
        {
            Some(a) => {
                a.last_seen_ms = a.last_seen_ms.or(s.last_seen_ms);
                a.size = a.size.take().or(s.size);
            }
            None => out.push(s),
        }
    }
    out.sort_by(|a, b| (&a.widget_id, &a.instance).cmp(&(&b.widget_id, &b.instance)));
    out
}

/// Native placement list + receipts.
///
/// `exact` (Android): the native list names every placement; receipts only fill size and
/// last render, never add. Not exact (iOS: WidgetCenter knows kinds, the extension knows
/// the Edit Widget choice): a kind's receipts with an instance replace its bare entries;
/// kinds no longer placed are dropped even if they rendered recently.
pub fn merge_native_active(
    live: Vec<ActiveWidget>,
    seen: Vec<ActiveWidget>,
    exact: bool,
) -> Vec<ActiveWidget> {
    if exact {
        let mut merged = merge_active(live.clone(), seen);
        merged.retain(|a| {
            live.iter()
                .any(|l| l.widget_id == a.widget_id && l.instance == a.instance)
        });
        return merged;
    }
    let mut out: Vec<ActiveWidget> = Vec::new();
    for l in &live {
        let instances: Vec<&ActiveWidget> = seen
            .iter()
            .filter(|s| s.widget_id == l.widget_id && s.instance.is_some())
            .collect();
        if instances.is_empty() {
            let receipt = seen.iter().find(|s| s.widget_id == l.widget_id);
            out.push(ActiveWidget {
                last_seen_ms: receipt.and_then(|r| r.last_seen_ms),
                ..l.clone()
            });
        } else {
            for s in instances {
                if !out.contains(s) {
                    out.push(s.clone());
                }
            }
        }
    }
    out.sort_by(|a, b| {
        (&a.widget_id, &a.instance, &a.size).cmp(&(&b.widget_id, &b.instance, &b.size))
    });
    out
}

/// Shared-map key where the Windows Widgets Board provider lists its placed instances.
pub const BOARD_WIDGETS_KEY: &str = "board_widgets";

/// `[{widgetId, instance}]` written by `templates/windows-widget` (Store.cs); empty on junk.
pub fn parse_board_widgets(raw: &str) -> Vec<ActiveWidget> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Board {
        widget_id: String,
        instance: String,
    }
    serde_json::from_str::<Vec<Board>>(raw)
        .unwrap_or_default()
        .into_iter()
        .map(|b| ActiveWidget {
            widget_id: b.widget_id,
            instance: Some(b.instance),
            size: Some("medium".into()),
            last_seen_ms: None,
        })
        .collect()
}

/// Receipts older than this do not count as "placed" (WidgetKit renders at least daily).
pub const ACTIVE_RECEIPT_MAX_AGE_MS: u64 = 48 * 60 * 60 * 1000;

/// Active placements from render receipts: one per (widget, instance), newest render
/// wins, older than `max_age_ms` dropped, sorted by widget id then instance.
pub fn active_from_receipts(
    receipts: &[crate::receipt::WidgetRenderReceipt],
    now_ms: u64,
    max_age_ms: u64,
) -> Vec<ActiveWidget> {
    let mut best: HashMap<(String, String), &crate::receipt::WidgetRenderReceipt> = HashMap::new();
    for r in receipts {
        if now_ms.saturating_sub(r.ts) > max_age_ms {
            continue;
        }
        let key = (r.widget_id.clone(), r.instance.clone());
        match best.get(&key) {
            Some(prev) if prev.ts >= r.ts => {}
            _ => {
                best.insert(key, r);
            }
        }
    }
    let mut out: Vec<ActiveWidget> = best
        .into_values()
        .map(|r| ActiveWidget {
            widget_id: r.widget_id.clone(),
            instance: Some(r.instance.clone()).filter(|i| !i.is_empty()),
            size: r.size.clone(),
            last_seen_ms: Some(r.ts),
        })
        .collect();
    out.sort_by(|a, b| (&a.widget_id, &a.instance).cmp(&(&b.widget_id, &b.instance)));
    out
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

/// Like [`touch_meta`], but the new nonce is at least `floor + 1`.
///
/// Needed on Apple: the widget still picks the freshest map across *all*
/// transports. A stale sibling with a higher nonce would otherwise win forever
/// after the host switches to a single writer.
pub fn touch_meta_above(map: &mut DataMap, floor: u64) {
    let next = map_nonce(map)
        .saturating_add(1)
        .max(floor.saturating_add(1));
    map.insert(META_NONCE_KEY.into(), next.to_string());
    map.insert(META_UPDATED_AT_KEY.into(), now_ms().to_string());
}

/// Whether this map carries at least one `config:{widgetId}` payload.
pub fn map_has_config(map: &DataMap) -> bool {
    map.keys().any(|k| k.starts_with(CONFIG_KEY_PREFIX))
}

/// Pick the freshest map among candidates.
///
/// Maps that carry a `config:*` key always beat probe-only / empty siblings,
/// even when the empty sibling has a higher nonce (pending-clear bumps).
/// Among equals on that axis: max nonce, then max updatedAt.
pub fn pick_freshest(maps: impl IntoIterator<Item = DataMap>) -> DataMap {
    let mut best: Option<DataMap> = None;
    let mut best_has_config = false;
    let mut best_nonce = 0u64;
    let mut best_ts = 0u64;
    for map in maps {
        let has_config = map_has_config(&map);
        let n = map_nonce(&map);
        let t = map_updated_at(&map);
        let better = match &best {
            None => true,
            Some(_) if has_config && !best_has_config => true,
            Some(_) if !has_config && best_has_config => false,
            Some(_) => n > best_nonce || (n == best_nonce && t > best_ts),
        };
        if better {
            best_has_config = has_config;
            best_nonce = n;
            best_ts = t;
            best = Some(map);
        }
    }
    best.unwrap_or_default()
}

/// Action delivered from a native widget back to the host app.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetActionEnvelope {
    /// Action verb (button / toggle / list row).
    pub action: String,
    /// Optional opaque payload string.
    #[serde(default)]
    pub payload: Option<String>,
    /// Unix epoch milliseconds when the action was enqueued.
    pub ts: u64,
    /// Widget id that emitted the action.
    pub widget_id: String,
    /// App Group / prefs group key.
    pub group: String,
}

impl WidgetActionEnvelope {
    /// Build an envelope with `ts = now`.
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

/// Gate for emitting queued widget taps as `widget-action`.
///
/// A tap that launches the app is queued before any JavaScript listener exists; emitting
/// it then loses it. The host keeps the queue until JS shows it listens (its first
/// `poll_pending_actions`, done by `onWidgetAction`), or until `fallback_ms` after start
/// for apps that only `listen("widget-action")`.
#[derive(Debug)]
pub struct ListenerGate {
    ready: std::sync::atomic::AtomicBool,
    started_ms: u64,
    fallback_ms: u64,
}

impl ListenerGate {
    /// Default fallback: a webview is listening well within this after start.
    pub const DEFAULT_FALLBACK_MS: u64 = 10_000;

    /// Closed gate started at `now_ms`.
    pub fn new(now_ms: u64, fallback_ms: u64) -> Self {
        Self {
            ready: std::sync::atomic::AtomicBool::new(false),
            started_ms: now_ms,
            fallback_ms,
        }
    }

    /// JavaScript listens: open for good.
    pub fn open(&self) {
        self.ready.store(true, std::sync::atomic::Ordering::SeqCst);
    }

    /// Milliseconds until the fallback opens the gate (0 when open).
    pub fn remaining_ms(&self, now_ms: u64) -> u64 {
        if self.is_open(now_ms) {
            0
        } else {
            (self.started_ms + self.fallback_ms).saturating_sub(now_ms)
        }
    }

    /// May queued actions be emitted at `now_ms`?
    pub fn is_open(&self, now_ms: u64) -> bool {
        self.ready.load(std::sync::atomic::Ordering::SeqCst)
            || now_ms.saturating_sub(self.started_ms) >= self.fallback_ms
    }
}

/// Parse pending actions JSON; empty on missing/invalid.
pub fn parse_pending_actions(raw: Option<&str>) -> Vec<WidgetActionEnvelope> {
    let Some(s) = raw.filter(|s| !s.is_empty()) else {
        return Vec::new();
    };
    serde_json::from_str(s).unwrap_or_default()
}

/// Shared-file queue owned by another process (Windows provider): keep what is on disk,
/// minus the actions this host just drained. The host never writes its own snapshot of
/// the queue, so a stale snapshot cannot resurrect drained actions or drop new ones.
pub fn pending_minus_drained(
    disk_raw: Option<&str>,
    drained: &[WidgetActionEnvelope],
) -> crate::Result<String> {
    let rest: Vec<_> = parse_pending_actions(disk_raw)
        .into_iter()
        .filter(|a| !drained.contains(a))
        .collect();
    encode_pending_actions(&rest)
}

/// Serialize pending actions for storage.
pub fn encode_pending_actions(actions: &[WidgetActionEnvelope]) -> crate::Result<String> {
    Ok(serde_json::to_string(actions)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn receipt(id: &str, inst: &str, ts: u64) -> crate::receipt::WidgetRenderReceipt {
        crate::receipt::WidgetRenderReceipt {
            widget_id: id.into(),
            group: "g".into(),
            instance: inst.into(),
            nonce: 0,
            size: Some("small".into()),
            theme: None,
            schema: 1,
            source: "prefs".into(),
            trigger: None,
            rendered: vec![],
            skipped: vec![],
            ts,
        }
    }

    fn aw(id: &str, inst: Option<&str>, size: Option<&str>, seen: Option<u64>) -> ActiveWidget {
        ActiveWidget {
            widget_id: id.into(),
            instance: inst.map(Into::into),
            size: size.map(Into::into),
            last_seen_ms: seen,
        }
    }

    /// The order a renderer reads a placement's keys in (the keys commands write are
    /// checked over IPC in tests/offline_model.rs).
    #[test]
    fn lookup_keys_scenario() {
        assert_eq!(scoped_id("w", Some("")), "w");
        assert_eq!(config_key("w"), "config:w");
        assert_eq!(
            lookup_keys(CONFIG_KEY_PREFIX, "w", Some("1")),
            ["config:w#1", "config:w"]
        );
        assert_eq!(lookup_keys(CONFIG_KEY_PREFIX, "w", Some("")), ["config:w"]);
        assert_eq!(lookup_keys(FETCHED_KEY_PREFIX, "w", None), ["fetched:w"]);
    }

    /// Which widgets are placed: render receipts aged and deduped, merged with live
    /// windows, with native lists (exact Android, inexact iOS) and the Widgets Board.
    #[test]
    fn placement_listing_scenario() {
        let rs = [
            receipt("w", "2", 900),
            receipt("w", "1", 500),
            receipt("w", "1", 800),
            receipt("w", "1", 700),
            receipt("old", "", 99),
            receipt("a", "", 1000),
        ];
        let got = active_from_receipts(&rs, 1000, 900);
        assert_eq!(
            got,
            [
                ActiveWidget {
                    size: Some("small".into()),
                    ..aw("a", None, None, Some(1000))
                },
                ActiveWidget {
                    size: Some("small".into()),
                    ..aw("w", Some("1"), None, Some(800))
                },
                ActiveWidget {
                    size: Some("small".into()),
                    ..aw("w", Some("2"), None, Some(900))
                },
            ]
        );
        // Exactly at max age still counts; the default window is two days.
        assert_eq!(
            active_from_receipts(&[receipt("x", "", 100)], 1000, 900).len(),
            1
        );
        let h = 60 * 60 * 1000;
        let rs = [
            receipt("fresh", "", 100 * h - 47 * h),
            receipt("stale", "", 100 * h - 49 * h),
        ];
        let got = active_from_receipts(&rs, 100 * h, ACTIVE_RECEIPT_MAX_AGE_MS);
        assert_eq!(
            got.iter().map(|a| a.widget_id.as_str()).collect::<Vec<_>>(),
            ["fresh"]
        );

        // Live first; receipts fill gaps and add what is missing.
        let got = merge_active(
            vec![
                aw("w", Some("win"), None, None),
                aw("b", None, Some("large"), None),
            ],
            vec![
                aw("w", Some("win"), Some("small"), Some(5)),
                aw("a", None, None, Some(1)),
                aw("b", None, Some("small"), Some(2)),
            ],
        );
        assert_eq!(
            got,
            vec![
                aw("a", None, None, Some(1)),
                aw("b", None, Some("large"), Some(2)),
                aw("w", Some("win"), Some("small"), Some(5)),
            ]
        );
        // Android: exact list; receipts fill, never add.
        let got = merge_native_active(
            vec![aw("w", Some("7"), None, None)],
            vec![
                aw("w", Some("7"), Some("small"), Some(3)),
                aw("w", Some("8"), None, Some(4)),
            ],
            true,
        );
        assert_eq!(got, vec![aw("w", Some("7"), Some("small"), Some(3))]);
        // iOS: kinds from WidgetCenter, instances from receipts.
        let got = merge_native_active(
            vec![
                aw("w", None, Some("small"), None),
                aw("w", None, Some("medium"), None),
                aw("plain", None, Some("large"), None),
            ],
            vec![
                aw("w", Some("nyc"), Some("small"), Some(5)),
                aw("w", Some("ldn"), Some("medium"), Some(6)),
                aw("plain", None, Some("large"), Some(9)),
                aw("gone", Some("x"), None, Some(1)),
            ],
            false,
        );
        assert_eq!(
            got,
            vec![
                aw("plain", None, Some("large"), Some(9)),
                aw("w", Some("ldn"), Some("medium"), Some(6)),
                aw("w", Some("nyc"), Some("small"), Some(5)),
            ]
        );
        // Placed kind that never rendered: listed bare.
        assert_eq!(
            merge_native_active(vec![aw("n", None, None, None)], vec![], false),
            vec![aw("n", None, None, None)]
        );
        // Windows Widgets Board list.
        assert_eq!(
            parse_board_widgets(r#"[{"widgetId":"default","instance":"B1"}]"#),
            vec![aw("default", Some("B1"), Some("medium"), None)]
        );
        assert!(parse_board_widgets("nope").is_empty());
    }

    /// Several transports hold copies of the map: each write bumps the nonce and time, and
    /// the reader picks the freshest (config first, then nonce, then updatedAt).
    #[test]
    fn transport_freshness_scenario() {
        let m = |tag: &str, config: bool, nonce: u64, ts: u64| {
            let mut d = DataMap::new();
            d.insert("tag".into(), tag.into());
            if config {
                d.insert("config:w".into(), "{}".into());
            }
            d.insert(META_NONCE_KEY.into(), nonce.to_string());
            d.insert(META_UPDATED_AT_KEY.into(), ts.to_string());
            d
        };
        let pick = |maps: Vec<DataMap>| pick_freshest(maps)["tag"].clone();

        // A write: nonce + 1, updatedAt = now (Unix ms).
        let before = now_ms();
        assert!(before > 1_600_000_000_000, "now_ms is Unix ms");
        let mut live = m("live", true, 7, 0);
        touch_meta(&mut live);
        assert_eq!(map_nonce(&live), 8);
        assert!(map_updated_at(&live) >= before);
        assert_eq!(map_updated_at(&m("x", false, 0, 123)), 123);
        assert_eq!(
            (map_nonce(&DataMap::new()), map_updated_at(&DataMap::new())),
            (0, 0)
        );
        assert!(map_has_config(&live) && !map_has_config(&m("p", false, 0, 0)));

        // A probe-only sibling with a higher nonce never wipes the config, in either order.
        let probe = m("probe", false, 99, u64::MAX);
        assert_eq!(pick(vec![probe.clone(), live.clone()]), "live");
        assert_eq!(pick(vec![live.clone(), probe]), "live");
        // A stale sibling stuck at nonce 65: the next write goes above it and wins.
        let stale = m("stale", true, 65, 0);
        assert_eq!(pick(vec![live.clone(), stale.clone()]), "stale");
        touch_meta_above(&mut live, 65);
        assert_eq!(map_nonce(&live), 66);
        assert_eq!(pick(vec![stale, live]), "live");

        // Same config axis: higher nonce wins whatever the order and timestamps.
        assert_eq!(pick(vec![m("a", false, 1, 9), m("b", false, 2, 0)]), "b");
        assert_eq!(pick(vec![m("a", true, 1, 9), m("b", true, 2, 0)]), "b");
        assert_eq!(pick(vec![m("a", false, 2, 0), m("b", false, 1, 9)]), "a");
        // Equal nonce: newer updatedAt wins; a full tie keeps the first.
        assert_eq!(pick(vec![m("a", false, 2, 1), m("b", false, 2, 5)]), "b");
        assert_eq!(pick(vec![m("a", false, 2, 5), m("b", false, 2, 1)]), "a");
        assert_eq!(pick(vec![m("a", false, 2, 5), m("b", false, 2, 5)]), "a");
        assert!(pick_freshest(Vec::new()).is_empty());
    }

    /// Widget taps: held until JS listens (or the fallback), and a drain that keeps taps
    /// queued meanwhile. Delivery through `pollPendingActions`: tests/actions.rs.
    #[test]
    fn action_queue_scenario() {
        let g = ListenerGate::new(1_000, 500);
        assert!(!g.is_open(1_000));
        assert!(!g.is_open(1_499));
        assert!(g.is_open(1_500));
        assert!(!g.is_open(0));
        assert_eq!(g.remaining_ms(1_200), 300);
        assert_eq!(g.remaining_ms(1_500), 0);
        g.open();
        assert_eq!(g.remaining_ms(1_000), 0);
        assert!(g.is_open(1_000));

        let a = WidgetActionEnvelope::new("a", None, "w", "g");
        let b = WidgetActionEnvelope::new("b", Some("1".into()), "w", "g");
        let disk = encode_pending_actions(&[a.clone(), b.clone()]).unwrap();
        let left = pending_minus_drained(Some(&disk), &[a]).unwrap();
        assert_eq!(parse_pending_actions(Some(&left)), vec![b]);
        assert_eq!(pending_minus_drained(None, &[]).unwrap(), "[]");
        assert!(parse_pending_actions(Some("junk")).is_empty());
    }
}
