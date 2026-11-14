//! Stories: live activities on the desktop island, where timing matters (delayed
//! dismissal, hover collapse) or the island window must already exist (events).
//! The command semantics (validation, templates, stale `timestamp`, final content)
//! are in the model test (tests/offline_model.rs).
#![cfg(not(any(target_os = "android", target_os = "ios")))]
// Linux pins the window through `WebviewWindow::gtk_window`, which MockRuntime does not
// implement (`unimplemented!()`).
#![cfg(not(target_os = "linux"))]

mod common;
use common::*;

fn start(app: &App, id: &str, title: &str) -> Value {
    app.ok(
        "start_activity",
        json!({ "content": { "id": id, "title": title } }),
    )
}

fn island_ids(view: &Value) -> Vec<&str> {
    view["activities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["id"].as_str().unwrap())
        .collect()
}

/// Given a running activity on the island,
/// When a second one starts, the first is updated, then both end,
/// Then the page gets `activity-island` with the newest in the pill and the update in
/// place, and the island empties with the last end.
#[test]
fn island_lifecycle_with_events() {
    let app = App::new();
    assert_eq!(start(&app, "life-a", "A")["surface"], json!("desktop"));
    start(&app, "life-b", "B");
    let view = app
        .wait_event("activity-island", Duration::from_secs(5), |v| {
            island_ids(v) == ["life-b", "life-a"]
        })
        .expect("activity-island with both activities");
    assert_eq!(view["activities"][0]["widgetId"], json!("activity.life-b"));

    let updated = json!({ "content": { "id": "life-a", "title": "A2" } });
    assert_eq!(app.ok("update_activity", updated), json!(true));
    app.wait_event("activity-island", Duration::from_secs(5), |v| {
        island_ids(v) == ["life-b", "life-a"]
    })
    .expect("update keeps its place");

    assert_eq!(
        app.ok("end_activity", json!({ "id": "life-a" })),
        json!(true)
    );
    assert_eq!(
        app.ok("end_activity", json!({ "id": "life-b" })),
        json!(true)
    );
    assert_eq!(app.ok("list_activities", json!({})), json!([]));
    assert_eq!(app.ok("activity_island", json!({})), Value::Null);
}

/// Given a running activity,
/// When it ends with `dismissAfterMs`,
/// Then it is no longer listed at once, the island keeps showing its final state, and
/// the island drops it after the delay.
#[test]
fn delayed_dismissal_keeps_the_final_state_until_the_delay() {
    let app = App::new();
    start(&app, "late", "Late");
    let end = json!({ "id": "late", "dismissAfterMs": 300 });
    assert_eq!(app.ok("end_activity", end), json!(true));
    assert_eq!(app.ok("list_activities", json!({})), json!([]));
    let shown = app.ok("activity_island", json!({}));
    assert_eq!(island_ids(&shown), ["late"], "{shown}");
    let gone = poll_until(Duration::from_secs(5), || {
        app.ok("activity_island", json!({})).is_null().then_some(())
    });
    assert!(gone.is_some(), "island still shows the dismissed activity");
}

/// Given an expanded island and the cursor away from it,
/// When the host's hover poller samples the cursor for a while,
/// Then the island collapses by itself.
#[test]
fn expanded_island_collapses_when_the_cursor_is_away() {
    let app = App::new();
    start(&app, "hover", "Hover");
    let view = app.ok("activity_island", json!({ "expanded": true }));
    assert_eq!(view["expanded"], json!(true));
    let collapsed = poll_until(Duration::from_secs(5), || {
        (app.ok("activity_island", json!({}))["expanded"] == json!(false)).then_some(())
    });
    assert!(collapsed.is_some(), "island stayed expanded");
    app.ok("end_activity", json!({ "id": "hover" }));
}
