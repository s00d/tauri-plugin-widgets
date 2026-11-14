//! Stories: the macOS file transports the host and the WidgetKit extension share
//! (App Group file, sandbox container), driven through `macos_transport` directly: the
//! mock app cannot point its single driver at several transports at once.
//!
//! Separate binary so `WIDGET_*` env overrides do not leak into other tests.
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

fn env_for(tmp: &std::path::Path, app_group_file: &std::path::Path) -> EnvGuard {
    EnvGuard::apply(&[
        ("WIDGET_CONTAINER_ROOT", tmp.to_string_lossy().into_owned()),
        ("WIDGET_EXTENSION_BUNDLE", "test.app.widgetkit".into()),
        (
            "WIDGET_APP_GROUP_DATA_FILE",
            app_group_file.to_string_lossy().into_owned(),
        ),
    ])
}

fn content(cfg: &WidgetConfig) -> String {
    serde_json::to_string(cfg).unwrap()
}

/// Given the App Group file and the sandbox container (paths from the test overrides),
/// When the host writes a config, a fresher copy lands in the sandbox, writers race, and
/// the sandbox file gets corrupted,
/// Then reads return the written config, then the freshest copy, then a whole freshest
/// map, then fall back to the App Group copy, and the next host write recovers.
#[test]
fn config_delivery_write_read_concurrent_recovery() {
    let _lock = env_lock().lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let app_group_file = tmp.path().join("appgroup/widget_data.json");
    const GROUP: &str = "group.test.app";
    const WIDGET: &str = "probe";

    // 1. Overrides: sandbox path under WIDGET_CONTAINER_ROOT with WIDGET_EXTENSION_BUNDLE.
    {
        let _env = EnvGuard::apply(&[
            (
                "WIDGET_CONTAINER_ROOT",
                tmp.path().to_string_lossy().into_owned(),
            ),
            ("WIDGET_EXTENSION_BUNDLE", "custom.bundle.widgetkit".into()),
        ]);
        let path = mt::sandbox_widget_data_path(GROUP);
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
    let _env = env_for(tmp.path(), &app_group_file);
    assert!(
        get_config_freshest(GROUP, WIDGET, Some(&app_group_file), [])
            .unwrap()
            .is_none()
    );

    // 2. Write v1 on the App Group file → read back.
    let mut map_a = Default::default();
    put_config_in_map(&mut map_a, WIDGET, &cfg_v1()).unwrap();
    write_map_file(&app_group_file, &map_a).unwrap();
    let got = get_config_freshest(GROUP, WIDGET, Some(&app_group_file), [])
        .unwrap()
        .unwrap();
    assert_eq!(content(&got), content(&cfg_v1()));

    // 3. Sandbox gets v2 with a higher nonce → it wins.
    let map_b = map_with_nonce(99, WIDGET, &cfg_v2()).unwrap();
    write_sandbox_map(GROUP, &map_b).unwrap();
    let got = get_config_freshest(GROUP, WIDGET, Some(&app_group_file), [])
        .unwrap()
        .unwrap();
    assert_eq!(
        content(&got),
        content(&cfg_v2()),
        "pick_freshest did not choose the higher-nonce transport"
    );

    // 4. Concurrent writers on the sandbox file: never a torn file, the result is one of
    //    the written maps, and the freshest pick still sees a config.
    let handles: Vec<_> = (0..8u64)
        .map(|t| {
            std::thread::spawn(move || {
                for k in 0..10 {
                    let m = map_with_nonce(100 + t * 10 + k, WIDGET, &cfg_v2()).unwrap();
                    write_sandbox_map(GROUP, &m).unwrap();
                }
            })
        })
        .collect();
    handles.into_iter().for_each(|h| h.join().unwrap());
    let sandbox = mt::sandbox_widget_data_path(GROUP);
    let last = mt::read_map_file(&sandbox).expect("whole map after concurrent writes");
    let nonce = tauri_plugin_widgets::store::map_nonce(&last);
    assert!((100..180).contains(&nonce), "{nonce}");
    let got = get_config_freshest(GROUP, WIDGET, Some(&app_group_file), [])
        .unwrap()
        .unwrap();
    assert_eq!(content(&got), content(&cfg_v2()));

    // 5. Corrupt sandbox file → the App Group copy is used → a host rewrite recovers.
    std::fs::write(&sandbox, "{\"__meta_nonce__\": ").unwrap();
    let got = get_config_freshest(GROUP, WIDGET, Some(&app_group_file), [])
        .unwrap()
        .unwrap();
    assert_eq!(content(&got), content(&cfg_v1()));
    write_sandbox_map(GROUP, &map_b).unwrap();
    let got = get_config_freshest(GROUP, WIDGET, Some(&app_group_file), [])
        .unwrap()
        .unwrap();
    assert_eq!(content(&got), content(&cfg_v2()));
}

/// Given the extension enqueued taps on the transports,
/// When the host polls, clears after a partial drain, and writes a config,
/// Then each tap is polled once (queue cleared), the undrained tap survives the clear
/// everywhere without a nonce bump, and the config write keeps a sibling's pending tap
/// while wiping the sibling's stale config.
#[test]
fn action_queue_enqueue_poll_clear_merge() {
    use tauri_plugin_widgets::store::{
        encode_pending_actions, map_has_config, map_nonce, parse_pending_actions, DataMap,
        WidgetActionEnvelope, PENDING_ACTIONS_KEY,
    };

    let _lock = env_lock().lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();

    // 1. Enqueue → poll → empty on the second poll.
    {
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
    // Each step below gets its own container root: the bundle override makes the sandbox
    // path the same file for every group.

    // 2. Partial drain: only the drained tap is removed, on every file, nonce unchanged.
    {
        const GROUP: &str = "group.test.clear-pending";
        let root = tmp.path().join("clear");
        let app_group_file = root.join("appgroup/widget_data.json");
        let _env = env_for(&root, &app_group_file);
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

        let maps = mt::read_file_transports(GROUP, Some(&app_group_file));
        assert_eq!(maps.len(), 2);
        for m in maps {
            let left = parse_pending_actions(m.get(PENDING_ACTIONS_KEY).map(|s| s.as_str()));
            assert_eq!(left.len(), 1, "only undrained action remains");
            assert_eq!(left[0].payload.as_deref(), Some("b"));
            assert_eq!(map_nonce(&m), 7, "clear must not bump nonce");
        }
    }

    // 3. Host write: merge taps from siblings → wipe sibling leftovers → write live map.
    {
        const GROUP: &str = "group.test.clear-leftover";
        let root = tmp.path().join("leftover");
        let app_group_file = root.join("appgroup/widget_data.json");
        let _env = env_for(&root, &app_group_file);
        const WIDGET: &str = "probe";
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

        let mut live = map_with_nonce(50, WIDGET, &cfg_v2()).unwrap();
        mt::merge_pending_into_map(&mut live, GROUP);
        mt::clear_sibling_transports(GROUP, "container", &mut live);
        write_sandbox_map(GROUP, &live).unwrap();

        let pending = parse_pending_actions(live.get(PENDING_ACTIONS_KEY).map(|s| s.as_str()));
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].payload.as_deref(), Some("1"));
        // App Group leftover must be empty (no stale config). Primary was not wiped.
        let wiped = std::fs::read_to_string(&app_group_file).unwrap_or_default();
        let wiped_map: DataMap = serde_json::from_str(&wiped).unwrap_or_default();
        assert!(
            !map_has_config(&wiped_map),
            "clear_sibling must wipe non-primary config"
        );
        assert!(map_has_config(
            &mt::read_map_file(&mt::sandbox_widget_data_path(GROUP)).unwrap()
        ));
    }
}
