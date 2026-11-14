//! Stories: `setWidgetConfigFromPath` (a config JSON file on disk instead of an object).
#![cfg(not(any(target_os = "android", target_os = "ios")))]

mod common;
use common::*;

fn preset(name: &str) -> String {
    format!("{ROOT}/tests/fixtures/presets/{name}")
}

/// The number of children of the first `list` element in a layout.
fn list_children(el: &Value) -> Option<usize> {
    if el["type"] == "list" {
        return el["children"].as_array().map(Vec::len);
    }
    el["children"].as_array()?.iter().find_map(list_children)
}

/// Given preset config files on disk,
/// When the app applies them by path,
/// Then the stored config is the file's (a deeply nested preset keeps its rich list
/// children).
#[test]
fn preset_files_are_applied() {
    let app = App::new();
    let set = |file: &str, id: &str| {
        app.ok(
            "set_widget_config_from_path",
            json!({ "path": preset(file), "widgetId": id, "skipReload": true }),
        )
    };
    assert_eq!(
        set("weather.json", "from-path-weather")["written"],
        json!(true)
    );
    let cfg = app.ok(
        "get_widget_config",
        json!({ "widgetId": "from-path-weather" }),
    );
    assert!(cfg["small"].is_object(), "{cfg}");

    set("stress-nest.json", "from-path-nest");
    let cfg = app.ok("get_widget_config", json!({ "widgetId": "from-path-nest" }));
    assert_eq!(list_children(&cfg["large"]), Some(3), "{cfg}");
}

/// Given paths that are blank, a device, a directory or missing,
/// When the app applies them,
/// Then each is rejected and nothing is stored.
#[test]
fn bad_paths_are_rejected() {
    let app = App::new();
    for path in [
        "",
        "   ",
        "/dev/zero",
        ROOT,
        "/no/such/widget-config-xyz.json",
    ] {
        let r = app.call(
            "set_widget_config_from_path",
            json!({ "path": path, "widgetId": "from-path-bad", "skipReload": true }),
        );
        assert!(r.is_err(), "accepted {path:?}");
    }
    assert_eq!(
        app.ok("get_widget_config", json!({ "widgetId": "from-path-bad" })),
        Value::Null
    );
}
