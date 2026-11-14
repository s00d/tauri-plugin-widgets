//! Stories: the macOS App Group store shared with the WidgetKit extension.
#![cfg(target_os = "macos")]

mod common;
use common::*;

/// Given the host wrote a key and reloaded WidgetKit (its files hold the write), and the
/// widget extension then wrote `state:` after a `set` tap straight into the store files
/// (fan-out, higher nonce, like `applyStatePatch`),
/// When the host writes another key from its cached map,
/// Then the extension's state survives next to both host keys, and reads see it.
#[test]
fn host_write_keeps_extension_state() {
    let app = App::new();
    app.ok("set_items", json!({ "key": "story.before", "value": "1" }));
    // Host writes reach the files from a writer thread; a reload waits for them (WidgetKit
    // reads the files).
    app.ok("reload_all_timelines", json!({}));

    let files = store_files(sandbox());
    let nonce_of = |m: &serde_json::Map<String, Value>| {
        m.get("__meta_nonce__")
            .and_then(Value::as_str)
            .and_then(|n| n.parse::<u64>().ok())
            .unwrap_or(0)
    };
    let mut map = files
        .iter()
        .filter_map(|f| std::fs::read_to_string(f).ok())
        .filter_map(|s| serde_json::from_str::<serde_json::Map<String, Value>>(&s).ok())
        .max_by_key(|m| nonce_of(m))
        .expect("host wrote a store file");
    assert_eq!(map.get("story.before"), Some(&json!("1")));
    let nonce = nonce_of(&map);
    map.insert("state:ext".into(), json!(r#"{"done":true}"#));
    map.insert("__meta_nonce__".into(), json!((nonce + 100).to_string()));
    for f in &files {
        std::fs::write(f, serde_json::to_string(&map).unwrap()).unwrap();
    }

    app.ok("set_items", json!({ "key": "story.after", "value": "2" }));

    assert_eq!(
        app.ok("get_widget_state", json!({ "widgetId": "ext" })),
        json!({ "done": true })
    );
    assert_eq!(
        app.ok("get_items", json!({ "key": "story.before" })),
        json!("1")
    );
    assert_eq!(
        app.ok("get_items", json!({ "key": "story.after" })),
        json!("2")
    );
}
