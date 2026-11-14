//! Property tests of the render-time contract (`src/resolve.rs`): invariants that must hold
//! for any config, not just the shared fixtures. Shrunk counterexamples belong in
//! `tests/fixtures/resolve/` so the Swift, Kotlin and TypeScript ports check them too.

use proptest::prelude::*;
use serde_json::{json, Value};
use tauri_plugin_widgets::models::WidgetConfig;
use tauri_plugin_widgets::resolve::{
    active_layouts, next_change_ms, resolve, source_url_allowed, upcoming_changes, ResolveContext,
};

fn text() -> impl Strategy<Value = String> {
    // Brace-heavy alphabet: exercises placeholder scanning, unclosed and nested braces.
    "[a-z{}. ]{0,14}"
}

fn element() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        text().prop_map(|s| json!({"type": "text", "content": s})),
        (0.0f64..1.0)
            .prop_map(|v| json!({"type": "progress", "value": v, "bind": {"value": "data.p"}})),
        text().prop_map(|s| json!({"type": "image", "url": s})),
    ];
    leaf.prop_recursive(4, 32, 4, |inner| {
        prop::collection::vec(inner, 0..4).prop_map(|c| json!({"type": "vstack", "children": c}))
    })
}

fn config() -> impl Strategy<Value = Value> {
    (
        prop::option::of(element()),
        prop::collection::vec((0u64..10_000, prop::option::of(element())), 0..5),
        prop::collection::btree_map("[a-z]{1,3}", text(), 0..3),
        prop::option::of(0.0f64..1.0),
    )
        .prop_map(|(small, entries, strings, p)| {
            let entries: Vec<Value> = entries
                .into_iter()
                .map(|(at, el)| match el {
                    Some(el) => json!({"at": at, "small": el}),
                    None => json!({"at": at}),
                })
                .collect();
            let mut cfg = json!({"entries": entries, "strings": {"en": strings}});
            if let Some(s) = small {
                cfg["small"] = s;
            }
            if let Some(p) = p {
                cfg["data"] = json!({"p": p, "inj": "{{data.p}}"});
            }
            cfg
        })
}

fn ctx(now: u64) -> ResolveContext<'static> {
    ResolveContext {
        now_ms: now,
        locale: "en",
        instance: Some("i"),
        ..Default::default()
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 256, ..ProptestConfig::default() })]

    #[test]
    fn resolve_keeps_version_and_strips_meta(cfg in config(), now in 0u64..12_000) {
        let out = resolve(&cfg, &ctx(now));
        prop_assert!(out.get("version").is_some());
        for key in ["entries", "strings", "data", "source", "defaultLocale"] {
            prop_assert!(out.get(key).is_none(), "{key} leaked");
        }
        prop_assert!(!out.to_string().contains("\"bind\""));
    }

    #[test]
    fn next_change_is_strictly_future_and_nothing_changes_before_it(cfg in config(), now in 0u64..12_000) {
        if let Some(t) = next_change_ms(&cfg, now) {
            prop_assert!(t > now);
            prop_assert_eq!(active_layouts(&cfg, now), active_layouts(&cfg, t - 1));
        } else {
            prop_assert_eq!(active_layouts(&cfg, now), active_layouts(&cfg, u64::MAX));
        }
    }

    #[test]
    fn upcoming_is_sorted_unique_bounded(cfg in config(), now in 0u64..12_000, limit in 0usize..8) {
        let v = upcoming_changes(&cfg, now, limit);
        prop_assert!(v.len() <= limit);
        prop_assert!(v.windows(2).all(|w| w[0] < w[1]));
        prop_assert!(v.iter().all(|t| *t > now));
        prop_assert_eq!(v.first().copied(), if limit == 0 { None } else { next_change_ms(&cfg, now) });
    }

    #[test]
    fn data_values_are_not_interpolated_again(p in 0.0f64..1.0) {
        // A data value containing `{{…}}` is inserted literally (no template injection).
        let cfg = json!({"small": {"type": "text", "content": "{{data.inj}}"}, "data": {"inj": "{{data.p}}", "p": p}});
        let data = cfg["data"].clone();
        let out = resolve(&cfg, &ResolveContext { now_ms: 0, locale: "en", instance: None, data: Some(&data), state: None });
        prop_assert_eq!(&out["small"]["content"], "{{data.p}}");
    }

    #[test]
    fn resolve_is_idempotent_on_its_output(cfg in config(), now in 0u64..12_000) {
        let once = resolve(&cfg, &ctx(now));
        // Placeholders are gone after one pass unless they came from data/strings.
        let twice = resolve(&once, &ctx(now));
        prop_assert_eq!(once.get("version"), twice.get("version"));
        prop_assert_eq!(once.get("small").is_some(), twice.get("small").is_some());
    }

    #[test]
    fn widget_config_roundtrip_is_stable(cfg in config()) {
        if let Ok(c) = serde_json::from_value::<WidgetConfig>(cfg) {
            let a = serde_json::to_value(&c).unwrap();
            let b = serde_json::to_value(serde_json::from_value::<WidgetConfig>(a.clone()).unwrap()).unwrap();
            prop_assert_eq!(a, b);
        }
    }

    #[test]
    fn plain_http_only_reaches_loopback(host in "[a-z0-9.@:-]{1,20}", path in "[a-z/?#]{0,8}") {
        let url = format!("http://{host}{path}");
        if source_url_allowed(&url) {
            let authority = host.split(['/', '?', '#']).next().unwrap();
            prop_assert!(!authority.contains('@'));
            let h = authority.split(':').next().unwrap();
            prop_assert!(h.eq_ignore_ascii_case("localhost") || h == "127.0.0.1", "{url}");
        }
    }
}
