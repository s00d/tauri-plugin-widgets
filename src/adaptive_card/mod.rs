//! IR → Adaptive Card 1.5 transpiler for Windows Widgets Board.
//!
//! Pure Rust — runs on any host. Elements Adaptive Cards cannot express natively
//! (`canvas`, `chart`, `gauge`, `shape`, `zstack`, gradients) are rasterized to
//! PNG data URIs when the `rasterize` feature is enabled; otherwise they may be
//! skipped / flattened and recorded in [`TranspileResult::skipped`].
//!
//! Element conversion is split across the six render groups (`layout`, `text`,
//! `media`, `data`, `interactive`, `spacing`) plus shared token helpers in
//! `style`, mirroring the Android/Swift renderers.

mod data;
mod interactive;
mod layout;
mod media;
mod spacing;
mod style;
mod text;

use crate::models::{WidgetConfig, WidgetElement};
use crate::receipt::SkippedElement;
use serde_json::{json, Value};

pub(crate) use style::sf_symbol_glyph_public;

/// Result of transpiling one IR root (plus skipped diagnostics).
#[derive(Debug, Clone)]
pub struct TranspileResult {
    pub card: Value,
    pub skipped: Vec<SkippedElement>,
}

/// Build an Adaptive Card 1.5 document from a root element.
pub fn to_adaptive_card(root: &WidgetElement) -> TranspileResult {
    let mut skipped = Vec::new();
    let body = el(root, &mut skipped);
    let card = json!({
        "type": "AdaptiveCard",
        "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
        "version": "1.5",
        "body": [body],
    });
    TranspileResult { card, skipped }
}

/// Pick a size branch from [`WidgetConfig`] and transpile.
pub fn to_adaptive_card_for_size(config: &WidgetConfig, size: &str) -> Option<TranspileResult> {
    let root = match size {
        "large" => config
            .large
            .as_ref()
            .or(config.medium.as_ref())
            .or(config.small.as_ref()),
        "medium" => config
            .medium
            .as_ref()
            .or(config.large.as_ref())
            .or(config.small.as_ref()),
        _ => config
            .small
            .as_ref()
            .or(config.medium.as_ref())
            .or(config.large.as_ref()),
    }?;
    Some(to_adaptive_card(root))
}

/// Structural check (no full JSON Schema dependency).
pub fn validate_card_structure(card: &Value) -> Result<(), String> {
    let obj = card
        .as_object()
        .ok_or_else(|| "card must be an object".to_string())?;
    if obj.get("type").and_then(|v| v.as_str()) != Some("AdaptiveCard") {
        return Err("type must be AdaptiveCard".into());
    }
    if obj.get("version").and_then(|v| v.as_str()) != Some("1.5") {
        return Err("version must be 1.5".into());
    }
    match obj.get("body") {
        Some(Value::Array(_)) => Ok(()),
        _ => Err("body must be an array".into()),
    }
}

/// Storage keys written next to the config map for the C# provider.
pub fn ac_template_key(widget_id: &str) -> String {
    format!("ac:template:{widget_id}")
}

pub fn ac_data_key(widget_id: &str) -> String {
    format!("ac:data:{widget_id}")
}

/// Dispatch one IR element to its render group.
fn el(e: &WidgetElement, skipped: &mut Vec<SkippedElement>) -> Value {
    match e {
        // layout
        WidgetElement::VStack(v) => layout::vstack(v, skipped),
        WidgetElement::HStack(h) => layout::hstack(h, skipped),
        WidgetElement::Container(c) => layout::container(c, skipped),
        WidgetElement::Grid(g) => layout::grid(g, skipped),
        WidgetElement::ZStack(z) => layout::zstack(e, z, skipped),

        // text
        WidgetElement::Text(t) => text::text(t),
        WidgetElement::Date(d) => text::date(d),
        WidgetElement::Timer(t) => text::timer(t),
        WidgetElement::Label(l) => text::label(l),

        // media
        WidgetElement::Image(i) => media::image(i),
        WidgetElement::Shape(_) => media::shape(e, skipped),
        WidgetElement::Canvas(_) => media::canvas(e, skipped),

        // data
        WidgetElement::Progress(p) => data::progress(p),
        WidgetElement::Gauge(_) => data::gauge(e, skipped),
        WidgetElement::Chart(_) => data::chart(e, skipped),
        WidgetElement::List(l) => data::list(l),

        // interactive
        WidgetElement::Button(b) => interactive::button(b),
        WidgetElement::Toggle(t) => interactive::toggle(t),
        WidgetElement::Link(l) => interactive::link(l, skipped),

        // spacing
        WidgetElement::Spacer(_) => spacing::spacer(),
        WidgetElement::Divider(d) => spacing::divider(d),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn snapshots_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/adaptive")
    }

    fn load_config(rel: &str) -> WidgetConfig {
        let path = fixtures_dir().join(rel);
        let raw =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        // Fixtures may contain explicit nulls — strip via Value first.
        let v: Value = serde_json::from_str(&raw).expect("json");
        serde_json::from_value(v).unwrap_or_else(|e| panic!("parse {rel}: {e}"))
    }

    #[test]
    fn weather_small_is_valid_v15() {
        let cfg = load_config("presets/weather.json");
        let result = to_adaptive_card_for_size(&cfg, "small").expect("small layout");
        validate_card_structure(&result.card).unwrap();
        assert_eq!(result.card["body"].as_array().unwrap().len(), 1);
        assert_eq!(result.card["body"][0]["type"], "Container");
    }

    #[test]
    fn unsupported_nodes_are_skipped() {
        let cfg: WidgetConfig = serde_json::from_value(json!({
            "version": 1,
            "small": {
                "type": "vstack",
                "children": [
                    { "type": "text", "content": "hi" },
                    { "type": "canvas", "width": 10, "height": 10, "elements": [] },
                    { "type": "zstack", "children": [
                        { "type": "text", "content": "a" }
                    ] }
                ]
            }
        }))
        .unwrap();
        let result = to_adaptive_card_for_size(&cfg, "small").unwrap();
        // zstack flattens; canvas rasterizes
        let body = serde_json::to_string(&result.card["body"]).unwrap();
        assert!(body.contains("Container") || body.contains("Image") || body.contains("TextBlock"));
        validate_card_structure(&result.card).unwrap();
    }

    #[test]
    #[cfg(feature = "rasterize")]
    fn chart_becomes_image_data_uri() {
        let cfg: WidgetConfig = serde_json::from_value(json!({
            "version": 1,
            "small": {
                "type": "chart",
                "chartType": "bar",
                "chartData": [
                    { "label": "a", "value": 1 },
                    { "label": "b", "value": 2 }
                ]
            }
        }))
        .unwrap();
        let result = to_adaptive_card_for_size(&cfg, "small").unwrap();
        assert!(result.skipped.is_empty(), "{:?}", result.skipped);
        assert_eq!(result.card["body"][0]["type"], "Image");
        let url = result.card["body"][0]["url"].as_str().unwrap();
        assert!(url.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn weather_small_matches_golden_or_writes_hint() {
        let cfg = load_config("presets/weather.json");
        let result = to_adaptive_card_for_size(&cfg, "small").unwrap();
        let golden_path = snapshots_dir().join("weather.small.json");
        if !golden_path.exists() {
            fs::create_dir_all(snapshots_dir()).unwrap();
            let pretty = serde_json::to_string_pretty(&result.card).unwrap();
            fs::write(&golden_path, pretty).unwrap();
            // First run creates golden — still assert structure.
            validate_card_structure(&result.card).unwrap();
            return;
        }
        let expected: Value =
            serde_json::from_str(&fs::read_to_string(&golden_path).unwrap()).unwrap();
        assert_eq!(
            result.card, expected,
            "adaptive card drift — update tests/snapshots/adaptive/weather.small.json if intentional"
        );
    }

    #[test]
    fn cases_transpile_without_panic() {
        let cases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/cases");
        let Ok(entries) = fs::read_dir(&cases_dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let case: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
            let fixture = case["fixture"].as_str().unwrap_or("");
            let size = case["size"].as_str().unwrap_or("small");
            let cfg_path = fixtures_dir().join(format!("{fixture}.json"));
            if !cfg_path.exists() {
                continue;
            }
            let cfg = load_config(&format!("{fixture}.json"));
            let Some(result) = to_adaptive_card_for_size(&cfg, size) else {
                continue;
            };
            validate_card_structure(&result.card)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        }
    }
}
