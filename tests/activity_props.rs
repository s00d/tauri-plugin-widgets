//! Properties of Live Activity requests (`src/models/activity.rs`): a template merged with
//! the request's own fields, and the 4 KiB budget measured on the payload that is sent.

use proptest::prelude::*;
use serde_json::json;
use tauri_plugin_widgets::models::{resolve_activity, ActivityContent, MAX_ACTIVITY_BYTES};

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: std::env::var("PROPTEST_CASES")
            .ok()
            .and_then(|c| c.parse().ok())
            .unwrap_or(64),
        failure_persistence: None,
        ..ProptestConfig::default()
    }
}

proptest! {
    #![proptest_config(config())]

    /// Own fields win over the template; an empty / blank own title takes the template's;
    /// fields the request leaves out come from the template (with `{{data.…}}` filled).
    #[test]
    fn own_fields_override_template(
        own_title in prop_oneof!["[ ]{0,2}", "[A-Za-z]{1,8}"],
        own_body in prop::option::of("[a-z ]{0,8}"),
        name in "[a-z]{1,6}",
    ) {
        let tpl = json!({"title": "T {{data.name}}", "body": "tpl body", "icon": "bolt"}).to_string();
        let c = ActivityContent {
            id: "a".into(),
            title: own_title.clone(),
            body: own_body.clone(),
            template: Some("t".into()),
            data: Some(json!({"name": name})),
            ..Default::default()
        };
        let r = resolve_activity(c, |k| (k == "activity-template:t").then(|| tpl.clone())).unwrap();
        let want_title = if own_title.trim().is_empty() { format!("T {name}") } else { own_title };
        prop_assert_eq!(r.title, want_title);
        prop_assert_eq!(r.body, Some(own_body.unwrap_or_else(|| "tpl body".into())));
        prop_assert_eq!(r.icon.as_deref(), Some("bolt"));
        prop_assert!(r.template.is_none() && r.data.is_none());
    }

    /// `validate` accepts exactly the requests whose serialized payload fits the budget:
    /// fields left out (`None`) cost nothing, the body costs what it serializes to. The body
    /// is sized so the payload lands on the limit ± 2 bytes, so the edge is hit every run.
    #[test]
    fn size_check_counts_sent_payload(
        delta in -2i64..=2,
        quote in any::<bool>(),
        icon in prop::option::of("[a-z]{1,8}"),
    ) {
        let target = (MAX_ACTIVITY_BYTES as i64 + delta) as usize;
        let content = |body: String| ActivityContent {
            id: "a".into(), title: "T".into(), body: Some(body), icon: icon.clone(), ..Default::default()
        };
        let size = |c: &ActivityContent| serde_json::to_vec(c).unwrap().len();
        // Some `"` (two bytes each) up front, then plain `x` (one byte each) up to the target.
        let mut body: String = if quote { "\"".repeat(10) } else { String::new() };
        let missing = target.saturating_sub(size(&content(body.clone())));
        body.push_str(&"x".repeat(missing));
        let c = content(body);
        let sent = size(&c);
        prop_assert_eq!(sent, target, "body sized to the target");
        prop_assert_eq!(c.validate().is_ok(), sent <= MAX_ACTIVITY_BYTES, "sent {} bytes", sent);
    }
}
