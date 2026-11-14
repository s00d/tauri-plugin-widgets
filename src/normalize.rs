//! Host-side IR normalization before store write.
//!
//! Renders must not guess defaults: `textStyle` becomes concrete `fontSize`,
//! semantic colors become adaptive hex pairs, stack `spacing` is explicit.
//! Layout (flex, Dynamic Type reflow) stays on the native renderer.

use crate::capabilities::{support_for, Support, WidgetPlatform};
use crate::models::{ColorValue, FontWeight, TextElement, TextStyle, WidgetConfig, WidgetElement};

/// Result of [`normalize`]: config with resolved defaults + recorded degradations.
#[derive(Debug, Clone)]
pub struct NormalizedConfig {
    /// Config ready to serialize into the store.
    pub config: WidgetConfig,
    /// Capability degradations applied / noted (`canvas.path`, `image.url`, …).
    pub degraded: Vec<String>,
}

/// Expand semantic typography / colors / spacing so renderers do not guess.
pub fn normalize(cfg: &WidgetConfig, target: WidgetPlatform) -> NormalizedConfig {
    let mut degraded = Vec::new();
    let mut config = cfg.clone();
    for el in config.layouts_mut() {
        walk_element(el, target, &mut degraded);
    }
    degraded.sort();
    degraded.dedup();
    NormalizedConfig { config, degraded }
}

fn walk_element(el: &mut WidgetElement, target: WidgetPlatform, degraded: &mut Vec<String>) {
    let type_name = el.type_name();
    match support_for(type_name, target).support {
        Support::Unsupported => {
            degraded.push(format!("{type_name}:unsupported"));
        }
        Support::Degraded => {
            degraded.push(format!("{type_name}:degraded"));
        }
        Support::Full => {}
    }

    match el {
        WidgetElement::VStack(v) => {
            if v.spacing.is_none() {
                v.spacing = Some(0.0);
            }
            for c in &mut v.children {
                walk_element(c, target, degraded);
            }
        }
        WidgetElement::HStack(v) => {
            if v.spacing.is_none() {
                v.spacing = Some(0.0);
            }
            for c in &mut v.children {
                walk_element(c, target, degraded);
            }
        }
        WidgetElement::ZStack(v) => {
            for c in &mut v.children {
                walk_element(c, target, degraded);
            }
        }
        WidgetElement::Grid(v) => {
            if v.spacing.is_none() {
                v.spacing = Some(0.0);
            }
            for c in &mut v.children {
                walk_element(c, target, degraded);
            }
        }
        WidgetElement::Container(v) => {
            for c in &mut v.children {
                walk_element(c, target, degraded);
            }
        }
        WidgetElement::Link(v) => {
            for c in &mut v.children {
                walk_element(c, target, degraded);
            }
        }
        WidgetElement::Text(t) => normalize_text(t),
        WidgetElement::Date(d) => {
            if let Some(ref mut c) = d.color {
                *c = expand_semantic_color(c);
            }
        }
        WidgetElement::Timer(t) => {
            if let Some(ref mut c) = t.color {
                *c = expand_semantic_color(c);
            }
        }
        WidgetElement::Repeat(r) => walk_element(&mut r.template, target, degraded),
        WidgetElement::Use(_) => {}
        WidgetElement::Label(l) => {
            if let Some(ref mut c) = l.color {
                *c = expand_semantic_color(c);
            }
            if let Some(ref mut c) = l.icon_color {
                *c = expand_semantic_color(c);
            }
        }
        WidgetElement::Image(i) => {
            if let Some(ref mut c) = i.color {
                *c = expand_semantic_color(c);
            }
            if i.url.is_some() && matches!(target, WidgetPlatform::Ios | WidgetPlatform::Macos) {
                // URL images are prefetched host-side; note residual risk if data empty.
                if i.data.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    degraded.push("image.url:prefetch-required".into());
                }
            }
        }
        WidgetElement::Button(b) => {
            if let Some(ref mut c) = b.color {
                *c = expand_semantic_color(c);
            }
            if let Some(ref mut c) = b.background_color {
                *c = expand_semantic_color(c);
            }
        }
        WidgetElement::Toggle(t) => {
            if let Some(ref mut c) = t.tint {
                *c = expand_semantic_color(c);
            }
        }
        WidgetElement::Progress(p) => {
            if let Some(ref mut c) = p.color {
                *c = expand_semantic_color(c);
            }
            if let Some(ref mut c) = p.tint {
                *c = expand_semantic_color(c);
            }
        }
        WidgetElement::Gauge(g) => {
            if let Some(ref mut c) = g.color {
                *c = expand_semantic_color(c);
            }
            if let Some(ref mut c) = g.tint {
                *c = expand_semantic_color(c);
            }
        }
        WidgetElement::Divider(d) => {
            if let Some(ref mut c) = d.color {
                *c = expand_semantic_color(c);
            }
        }
        WidgetElement::List(l) => {
            if let Some(ref mut c) = l.color {
                *c = expand_semantic_color(c);
            }
            if l.spacing.is_none() {
                l.spacing = Some(4.0);
            }
        }
        WidgetElement::Chart(c) => {
            if let Some(ref mut t) = c.tint {
                *t = expand_semantic_color(t);
            }
        }
        WidgetElement::Shape(s) => {
            if let Some(ref mut c) = s.fill {
                *c = expand_semantic_color(c);
            }
            if let Some(ref mut c) = s.stroke {
                *c = expand_semantic_color(c);
            }
        }
        WidgetElement::Canvas(_) | WidgetElement::Spacer(_) => {}
    }

    // Shared ElementStyle colors on every variant that flattens style.
    expand_style_colors(el);
}

fn expand_style_colors(el: &mut WidgetElement) {
    let style = match el {
        WidgetElement::VStack(v) => Some(&mut v.style),
        WidgetElement::HStack(v) => Some(&mut v.style),
        WidgetElement::ZStack(v) => Some(&mut v.style),
        WidgetElement::Grid(v) => Some(&mut v.style),
        WidgetElement::Container(v) => Some(&mut v.style),
        WidgetElement::Text(v) => Some(&mut v.style),
        WidgetElement::Image(v) => Some(&mut v.style),
        WidgetElement::Progress(v) => Some(&mut v.style),
        WidgetElement::Gauge(v) => Some(&mut v.style),
        WidgetElement::Button(v) => Some(&mut v.style),
        WidgetElement::Toggle(v) => Some(&mut v.style),
        WidgetElement::Divider(v) => Some(&mut v.style),
        WidgetElement::Spacer(_) => None,
        WidgetElement::Date(v) => Some(&mut v.style),
        WidgetElement::Chart(v) => Some(&mut v.style),
        WidgetElement::List(v) => Some(&mut v.style),
        WidgetElement::Link(v) => Some(&mut v.style),
        WidgetElement::Shape(v) => Some(&mut v.style),
        WidgetElement::Timer(v) => Some(&mut v.style),
        WidgetElement::Label(v) => Some(&mut v.style),
        WidgetElement::Repeat(v) => Some(&mut v.style),
        WidgetElement::Use(v) => Some(&mut v.style),
        WidgetElement::Canvas(v) => Some(&mut v.style),
    };
    if let Some(style) = style {
        if let Some(ref mut bg) = style.background {
            if let crate::models::BackgroundValue::Solid(s) = bg {
                if let Some(pair) = semantic_pair(s) {
                    *bg = crate::models::BackgroundValue::Adaptive {
                        light: pair.0.into(),
                        dark: pair.1.into(),
                    };
                }
            }
        }
        if let Some(ref mut border) = style.border {
            if let Some((light, dark)) = semantic_pair(&border.color) {
                // Borders are single-color on wire; pick dark-friendly default hex.
                let _ = light;
                border.color = dark.to_string();
            }
        }
    }
}

fn normalize_text(t: &mut TextElement) {
    if let Some(ts) = t.text_style.take() {
        let (size, weight) = text_style_metrics(&ts);
        if t.font_size.is_none() {
            t.font_size = Some(size);
        }
        if t.font_weight.is_none() {
            t.font_weight = weight;
        }
    }
    if let Some(ref mut c) = t.color {
        *c = expand_semantic_color(c);
    }
}

/// Approximate Dynamic Type → points (body = 17), matching Apple defaults.
fn text_style_metrics(ts: &TextStyle) -> (f64, Option<FontWeight>) {
    match ts {
        TextStyle::LargeTitle => (34.0, Some(FontWeight::Regular)),
        TextStyle::Title => (28.0, Some(FontWeight::Regular)),
        TextStyle::Title2 => (22.0, Some(FontWeight::Regular)),
        TextStyle::Title3 => (20.0, Some(FontWeight::Regular)),
        TextStyle::Headline => (17.0, Some(FontWeight::Semibold)),
        TextStyle::Body => (17.0, Some(FontWeight::Regular)),
        TextStyle::Callout => (16.0, Some(FontWeight::Regular)),
        TextStyle::Subheadline => (15.0, Some(FontWeight::Regular)),
        TextStyle::Footnote => (13.0, Some(FontWeight::Regular)),
        TextStyle::Caption => (12.0, Some(FontWeight::Regular)),
        TextStyle::Caption2 => (11.0, Some(FontWeight::Regular)),
    }
}

fn expand_semantic_color(c: &ColorValue) -> ColorValue {
    match c {
        ColorValue::Solid(s) => {
            if let Some((light, dark)) = semantic_pair(s) {
                ColorValue::Adaptive {
                    light: light.to_string(),
                    dark: dark.to_string(),
                }
            } else {
                c.clone()
            }
        }
        ColorValue::Adaptive { .. } => c.clone(),
    }
}

fn semantic_pair(name: &str) -> Option<(&'static str, &'static str)> {
    match name {
        "label" => Some(("#000000", "#FFFFFF")),
        "secondaryLabel" => Some(("#3C3C43", "#EBEBF5")),
        "systemBackground" => Some(("#FFFFFF", "#000000")),
        "secondarySystemBackground" => Some(("#F2F2F7", "#1C1C1E")),
        "accent" => Some(("#007AFF", "#0A84FF")),
        "separator" => Some(("#C6C6C8", "#545458")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{text, vstack, BackgroundValue};
    use std::fs;
    use std::path::PathBuf;

    fn cfg(v: serde_json::Value) -> WidgetConfig {
        serde_json::from_value(v).unwrap()
    }

    fn adaptive(c: Option<ColorValue>) -> (String, String) {
        match c {
            Some(ColorValue::Adaptive { light, dark }) => (light, dark),
            other => panic!("expected adaptive, got {other:?}"),
        }
    }

    /// Semantic colour names: the lookup table → a text colour → style background and
    /// border → layouts inside timeline `entries`; hex values are left alone.
    #[test]
    fn semantic_colors_expand_through_every_layout() {
        // 1. The table (light, dark) per name; a hex is not a semantic name.
        for (name, light, dark) in [
            ("label", "#000000", "#FFFFFF"),
            ("secondaryLabel", "#3C3C43", "#EBEBF5"),
            ("systemBackground", "#FFFFFF", "#000000"),
            ("secondarySystemBackground", "#F2F2F7", "#1C1C1E"),
            ("accent", "#007AFF", "#0A84FF"),
            ("separator", "#C6C6C8", "#545458"),
        ] {
            assert_eq!(semantic_pair(name), Some((light, dark)), "{name}");
        }
        assert_eq!(semantic_pair("#123456"), None);

        // 2. A text colour becomes an adaptive pair.
        let mut el = text("Hi");
        el.color = Some(ColorValue::Solid("label".into()));
        let out = normalize(&WidgetConfig::small(el), WidgetPlatform::Desktop);
        let WidgetElement::Text(t) = out.config.small.unwrap() else {
            panic!("expected text")
        };
        assert_eq!(
            adaptive(t.color),
            ("#000000".to_string(), "#FFFFFF".to_string())
        );

        // 3. Style background → adaptive; a border colour has one slot, so it takes the dark one.
        let out = normalize(
            &cfg(
                serde_json::json!({"small": {"type": "vstack", "background": "systemBackground",
                "border": {"color": "separator", "width": 1}}}),
            ),
            WidgetPlatform::Desktop,
        );
        let WidgetElement::VStack(v) = out.config.small.unwrap() else {
            panic!("expected vstack")
        };
        match v.style.background {
            Some(BackgroundValue::Adaptive { light, dark }) => {
                assert_eq!((light.as_str(), dark.as_str()), ("#FFFFFF", "#000000"))
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(v.style.border.unwrap().color, "#545458");

        // 4. Timeline entries are layouts too.
        let out = normalize(
            &cfg(
                serde_json::json!({"entries": [{"at": 1, "small": {"type": "text", "content": "x", "color": "label"}}]}),
            ),
            WidgetPlatform::Desktop,
        );
        let WidgetElement::Text(t) = out.config.entries[0].small.clone().unwrap() else {
            panic!("expected text")
        };
        assert!(matches!(t.color, Some(ColorValue::Adaptive { .. })));
    }

    /// Typography and stack defaults become explicit, so renderers never guess them.
    #[test]
    fn text_style_and_stack_spacing_become_explicit() {
        // 1. `textStyle: title` → fontSize 28 + regular weight, the style itself is dropped.
        let mut el = text("Hi");
        el.text_style = Some(TextStyle::Title);
        el.font_size = None;
        let out = normalize(&WidgetConfig::small(el), WidgetPlatform::Desktop);
        let WidgetElement::Text(t) = out.config.small.unwrap() else {
            panic!("expected text")
        };
        assert!(t.text_style.is_none());
        assert_eq!(t.font_size, Some(28.0));
        assert!(matches!(t.font_weight, Some(FontWeight::Regular)));

        // 2. Stack spacing defaults to 0 instead of the platform's own default.
        let cfg = WidgetConfig::small(vstack(vec![text("a").into(), text("b").into()]));
        let WidgetElement::VStack(v) = normalize(&cfg, WidgetPlatform::Ios).config.small.unwrap()
        else {
            panic!("expected vstack")
        };
        assert_eq!(v.spacing, Some(0.0));
    }

    /// `image.url` needs a host prefetch only where the widget cannot fetch (Apple), and
    /// only until `data` is filled.
    #[test]
    fn image_url_prefetch_degradation_by_platform_and_data() {
        let url_only =
            cfg(serde_json::json!({"small": {"type": "image", "url": "https://x/a.png"}}));
        assert_eq!(
            normalize(&url_only, WidgetPlatform::Ios).degraded,
            ["image.url:prefetch-required"]
        );
        assert!(normalize(&url_only, WidgetPlatform::Android)
            .degraded
            .is_empty());
        let with_data = cfg(
            serde_json::json!({"small": {"type": "image", "url": "https://x/a.png", "data": "QQ=="}}),
        );
        assert!(normalize(&with_data, WidgetPlatform::Ios)
            .degraded
            .is_empty());
        let no_url = cfg(serde_json::json!({"small": {"type": "image", "systemName": "star"}}));
        assert!(!normalize(&no_url, WidgetPlatform::Ios)
            .degraded
            .iter()
            .any(|d| d.contains("prefetch")));
    }

    /// One config through the host pipeline: parse → normalize per platform → capability
    /// warnings → Adaptive Card transpile of the normalized result.
    #[test]
    fn config_pipeline_parse_normalize_validate_transpile() {
        use crate::adaptive_card::{to_adaptive_card_for_size, validate_card_structure};
        use crate::capabilities::validate_config;

        // 1. Parse.
        let raw = cfg(serde_json::json!({
            "small": {"type": "vstack", "children": [
                {"type": "text", "content": "Now", "textStyle": "title", "color": "label"},
                {"type": "image", "url": "https://x/a.png"},
                {"type": "timer", "targetDate": "2030-01-01T00:00:00Z"}
            ]}
        }));
        // 2. Normalize: Apple needs the image prefetched; Windows degrades the live timer.
        let ios = normalize(&raw, WidgetPlatform::Ios);
        assert!(ios
            .degraded
            .contains(&"image.url:prefetch-required".to_string()));
        let win = normalize(&raw, WidgetPlatform::Windows);
        assert!(
            win.degraded.contains(&"timer:degraded".to_string()),
            "{:?}",
            win.degraded
        );
        assert!(!win.degraded.iter().any(|d| d.contains("prefetch")));
        let WidgetElement::VStack(v) = win.config.small.clone().unwrap() else {
            panic!("expected vstack")
        };
        assert_eq!(v.spacing, Some(0.0));
        // 3. Capability warnings agree with the degradations above.
        let warns = validate_config(&win.config, WidgetPlatform::Windows);
        assert!(warns.iter().any(|w| w.element == "timer.live"), "{warns:?}");
        // 4. The normalized config still transpiles to a valid AC 1.5 card with the
        //    concrete font size / colour.
        let card = to_adaptive_card_for_size(&win.config, "small")
            .unwrap()
            .card;
        validate_card_structure(&card).unwrap();
        assert!(card.to_string().contains("Now"), "{card}");
    }

    /// Every fixture normalizes on the platforms without a native renderer (no panics).
    #[test]
    fn fixtures_normalize_without_panic() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let mut n = 0;
        for entry in fs::read_dir(&root).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                for f in fs::read_dir(entry.path()).unwrap() {
                    let f = f.unwrap().path();
                    if f.extension().and_then(|e| e.to_str()) != Some("json") {
                        continue;
                    }
                    let raw = fs::read_to_string(&f).unwrap();
                    let cfg: WidgetConfig = match serde_json::from_str(&raw) {
                        Ok(c) => c,
                        Err(_) => continue, // event before/after blobs etc.
                    };
                    let _ = normalize(&cfg, WidgetPlatform::Desktop);
                    let _ = normalize(&cfg, WidgetPlatform::Windows);
                    n += 1;
                }
            }
        }
        assert!(n > 10, "expected to normalize many fixtures, got {n}");
    }
}
