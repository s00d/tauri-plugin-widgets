//! Untrusted JSON through every host-side path: resolve, typed parse, normalize for each
//! platform, Adaptive Cards transpile and SVG rasterization. Must never panic.
//! Run: `cargo +nightly fuzz run config -- -max_total_time=300` (corpus: tests/fixtures).
#![no_main]

use tauri_plugin_widgets::capabilities::WidgetPlatform;
use tauri_plugin_widgets::models::WidgetConfig;
use tauri_plugin_widgets::resolve::{next_change_ms, resolve, source_request, ResolveContext};

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(data) else {
        return;
    };
    let ctx = ResolveContext {
        now_ms: 1_000,
        locale: "en",
        instance: Some("i"),
        data: v.get("data"),
    };
    let _ = resolve(&v, &ctx);
    let _ = next_change_ms(&v, 1_000);
    let _ = source_request(&v, &ctx);
    if let Ok(cfg) = serde_json::from_value::<WidgetConfig>(v) {
        for p in WidgetPlatform::all() {
            let _ = tauri_plugin_widgets::normalize::normalize(&cfg, p);
            let _ = tauri_plugin_widgets::capabilities::validate_config(&cfg, p);
        }
        for size in ["small", "medium", "large"] {
            let _ = tauri_plugin_widgets::adaptive_card::to_adaptive_card_for_size(&cfg, size);
        }
        for el in [&cfg.small, &cfg.medium, &cfg.large].into_iter().flatten() {
            let _ = tauri_plugin_widgets::rasterize::element_to_svg(el);
        }
    }
});
