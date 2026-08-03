//! macOS multi-transport integration tests.
//!
//! Separate binary so `WIDGET_*` env overrides do not leak into other crates' tests.
//! Requires macOS (file transports + optional UserDefaults FFI).

#![cfg(target_os = "macos")]

use std::sync::{Mutex, OnceLock};
use tauri_plugin_widgets::macos_transport::{
    self as mt, enqueue_action_like_extension, get_config_freshest, map_with_nonce,
    poll_pending_actions_files, put_config_in_map, write_map_file, write_sandbox_map,
};
use tauri_plugin_widgets::models::WidgetConfig;

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn cfg_v1() -> WidgetConfig {
    serde_json::from_value(serde_json::json!({
        "version": 1,
        "small": { "type": "text", "content": "v1" }
    }))
    .unwrap()
}

fn cfg_v2() -> WidgetConfig {
    serde_json::from_value(serde_json::json!({
        "version": 1,
        "small": { "type": "text", "content": "v2" }
    }))
    .unwrap()
}

struct EnvGuard {
    keys: Vec<&'static str>,
}

impl EnvGuard {
    fn apply(pairs: &[(&'static str, String)]) -> Self {
        let keys: Vec<_> = pairs.iter().map(|(k, _)| *k).collect();
        for (k, v) in pairs {
            // SAFETY: tests hold `env_lock`; this binary is single-threaded for env mutation.
            unsafe { std::env::set_var(k, v) };
        }
        Self { keys }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for k in &self.keys {
            unsafe { std::env::remove_var(k) };
        }
    }
}

#[test]
fn freshest_pick_across_transports() {
    let _lock = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let app_group_file = tmp.path().join("appgroup/widget_data.json");
    let _env = EnvGuard::apply(&[
        (
            "WIDGET_CONTAINER_ROOT",
            tmp.path().to_string_lossy().into_owned(),
        ),
        ("WIDGET_EXTENSION_BUNDLE", "test.app.widgetkit".into()),
        (
            "WIDGET_APP_GROUP_DATA_FILE",
            app_group_file.to_string_lossy().into_owned(),
        ),
    ]);

    const GROUP: &str = "group.test.app";
    const WIDGET: &str = "probe";

    // Transport A (App Group file): older config
    let mut map_a = Default::default();
    put_config_in_map(&mut map_a, WIDGET, &cfg_v1()).unwrap();
    write_map_file(&app_group_file, &map_a).unwrap();

    // Transport B (sandbox): fresher nonce + cfg_v2
    let map_b = map_with_nonce(99, WIDGET, &cfg_v2()).unwrap();
    write_sandbox_map(GROUP, &map_b).unwrap();

    let got = get_config_freshest(GROUP, WIDGET, Some(&app_group_file), [])
        .unwrap()
        .expect("config present");
    assert_eq!(
        serde_json::to_string(&got).unwrap(),
        serde_json::to_string(&cfg_v2()).unwrap(),
        "pick_freshest did not choose the higher-nonce transport"
    );
}

#[test]
fn action_roundtrip() {
    let _lock = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let _env = EnvGuard::apply(&[
        (
            "WIDGET_CONTAINER_ROOT",
            tmp.path().to_string_lossy().into_owned(),
        ),
        ("WIDGET_EXTENSION_BUNDLE", "test.app.widgetkit".into()),
    ]);

    const GROUP: &str = "group.test.app";

    enqueue_action_like_extension("toggle", Some("true"), "probe", GROUP).unwrap();
    let got = poll_pending_actions_files(GROUP, None).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].action, "toggle");
    assert_eq!(got[0].payload.as_deref(), Some("true"));
    assert!(
        poll_pending_actions_files(GROUP, None).unwrap().is_empty(),
        "queue was not cleared"
    );
}

#[test]
fn clear_pending_everywhere_keeps_undrained_and_skips_meta_bump() {
    use tauri_plugin_widgets::store::{
        encode_pending_actions, map_nonce, parse_pending_actions, DataMap, PENDING_ACTIONS_KEY,
        WidgetActionEnvelope,
    };

    let _lock = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let app_group_file = tmp.path().join("appgroup/widget_data.json");
    let _env = EnvGuard::apply(&[
        (
            "WIDGET_CONTAINER_ROOT",
            tmp.path().to_string_lossy().into_owned(),
        ),
        ("WIDGET_EXTENSION_BUNDLE", "test.app.widgetkit".into()),
        (
            "WIDGET_APP_GROUP_DATA_FILE",
            app_group_file.to_string_lossy().into_owned(),
        ),
    ]);

    const GROUP: &str = "group.test.clear-pending";
    let drained = WidgetActionEnvelope::new("tap", Some("a".into()), "w", GROUP);
    let keep = WidgetActionEnvelope::new("tap", Some("b".into()), "w", GROUP);

    let mut map = DataMap::new();
    map.insert(
        PENDING_ACTIONS_KEY.into(),
        encode_pending_actions(&[drained.clone(), keep.clone()]).unwrap(),
    );
    map.insert("__meta_nonce__".into(), "7".into());
    write_map_file(&app_group_file, &map).unwrap();
    write_sandbox_map(GROUP, &map).unwrap();

    mt::clear_pending_actions_everywhere(GROUP, &[drained]);

    for m in mt::read_file_transports(GROUP, Some(&app_group_file)) {
        let left = parse_pending_actions(m.get(PENDING_ACTIONS_KEY).map(|s| s.as_str()));
        assert_eq!(left.len(), 1, "only undrained action remains");
        assert_eq!(left[0].payload.as_deref(), Some("b"));
        assert_eq!(map_nonce(&m), 7, "clear must not bump nonce");
    }
}

#[test]
fn merge_pending_then_clear_leftover_preserves_sibling_taps() {
    use tauri_plugin_widgets::store::{
        encode_pending_actions, map_has_config, parse_pending_actions, DataMap, PENDING_ACTIONS_KEY,
        WidgetActionEnvelope,
    };

    let _lock = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let app_group_file = tmp.path().join("appgroup/widget_data.json");
    let _env = EnvGuard::apply(&[
        (
            "WIDGET_CONTAINER_ROOT",
            tmp.path().to_string_lossy().into_owned(),
        ),
        ("WIDGET_EXTENSION_BUNDLE", "test.app.widgetkit".into()),
        (
            "WIDGET_APP_GROUP_DATA_FILE",
            app_group_file.to_string_lossy().into_owned(),
        ),
    ]);

    const GROUP: &str = "group.test.clear-leftover";
    const WIDGET: &str = "probe";

    // Stale config + pending tap on App Group file.
    let mut stale = DataMap::new();
    put_config_in_map(&mut stale, WIDGET, &cfg_v1()).unwrap();
    stale.insert(
        PENDING_ACTIONS_KEY.into(),
        encode_pending_actions(&[WidgetActionEnvelope::new(
            "tap",
            Some("1".into()),
            WIDGET,
            GROUP,
        )])
        .unwrap(),
    );
    write_map_file(&app_group_file, &stale).unwrap();

    // Host write path: merge taps → wipe leftovers → write live map to sandbox.
    let mut live = map_with_nonce(50, WIDGET, &cfg_v2()).unwrap();
    mt::merge_pending_into_map(&mut live, GROUP);
    mt::clear_leftover_transports(GROUP);
    write_sandbox_map(GROUP, &live).unwrap();

    let pending = parse_pending_actions(live.get(PENDING_ACTIONS_KEY).map(|s| s.as_str()));
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].payload.as_deref(), Some("1"));

    // App Group leftover must be empty (no stale config).
    let wiped = std::fs::read_to_string(&app_group_file).unwrap_or_default();
    let wiped_map: DataMap = serde_json::from_str(&wiped).unwrap_or_default();
    assert!(
        !map_has_config(&wiped_map),
        "clear_leftover must wipe sibling config"
    );
}

#[test]
fn container_root_override_not_home() {
    let _lock = env_lock().lock().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let _env = EnvGuard::apply(&[
        (
            "WIDGET_CONTAINER_ROOT",
            tmp.path().to_string_lossy().into_owned(),
        ),
        ("WIDGET_EXTENSION_BUNDLE", "custom.bundle.widgetkit".into()),
    ]);
    let path = mt::sandbox_widget_data_path("group.test.app");
    assert!(
        path.starts_with(tmp.path()),
        "sandbox path must use WIDGET_CONTAINER_ROOT, got {}",
        path.display()
    );
    assert!(
        path.to_string_lossy().contains("custom.bundle.widgetkit"),
        "must use WIDGET_EXTENSION_BUNDLE"
    );
}
