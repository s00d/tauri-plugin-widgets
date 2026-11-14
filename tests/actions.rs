//! Stories: taps queued by a native widget reach the app.
#![cfg(not(any(target_os = "android", target_os = "ios")))]

mod common;
use common::*;

fn envelope(action: &str, payload: Option<&str>) -> Value {
    json!({"action": action, "payload": payload, "ts": 1, "widgetId": "tapper", "group": DEFAULT_GROUP})
}

/// Given a widget queued a tap and a `set` tap (`$state`) in the shared store,
/// When the app polls,
/// Then the tap arrives exactly once (in the poll result, or as `widget-action` when the
/// background poller took it first), the `set` tap arrives as `widget-state-changed`
/// only, and the queue is empty afterwards.
#[test]
fn queued_taps_reach_the_app_once() {
    let app = App::new();
    let queue = json!([
        envelope("open", Some("p1")),
        envelope(
            "$state",
            Some(r#"{"widgetId":"tapper","instance":"i1","state":{"on":true}}"#)
        ),
    ]);
    app.ok(
        "set_items",
        json!({"key": "pending_actions", "value": queue.to_string()}),
    );
    let polled = app.ok("poll_pending_actions", json!({}));
    let mut taps: Vec<Value> = polled.as_array().unwrap().clone();
    let mut state = None;
    let delivered = poll_until(Duration::from_secs(5), || {
        for e in app.drain_events() {
            match e.name {
                "widget-action" => taps.push(e.payload),
                "widget-state-changed" => state = Some(e.payload),
                _ => {}
            }
        }
        (state.is_some() && !taps.is_empty()).then_some(())
    });
    assert!(delivered.is_some(), "taps {taps:?}, state {state:?}");
    assert_eq!(taps.len(), 1, "{taps:?}");
    assert_eq!(
        (&taps[0]["action"], &taps[0]["payload"]),
        (&json!("open"), &json!("p1"))
    );
    let state = state.unwrap();
    assert_eq!(
        (&state["widgetId"], &state["instance"], &state["state"]),
        (&json!("tapper"), &json!("i1"), &json!({"on": true}))
    );
    assert_eq!(app.ok("poll_pending_actions", json!({})), json!([]));
}
