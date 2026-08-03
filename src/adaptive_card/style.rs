//! Shared token helpers: container style, size/weight/color buckets, flex widths.

use crate::models::{
    ButtonElement, CanvasElement, ChartElement, ColorValue, ContainerElement, DateElement,
    DividerElement, ElementStyle, FontWeight, GaugeElement, GridElement, HStackElement,
    ImageElement, LabelElement, LinkElement, ListElement, ProgressElement, ShapeElement,
    TextAlignment, TextElement, TimerElement, ToggleElement, VStackElement, WidgetElement,
    ZStackElement,
};
use serde_json::{json, Value};

pub(super) fn flex_width(c: &WidgetElement) -> Value {
    flex_of(c)
        .map(|f| {
            if f <= 0.0 {
                json!("auto")
            } else {
                json!((f * 100.0).round() as u64)
            }
        })
        .unwrap_or(json!("auto"))
}

fn flex_of(e: &WidgetElement) -> Option<f64> {
    match e {
        WidgetElement::VStack(VStackElement { style, .. })
        | WidgetElement::HStack(HStackElement { style, .. })
        | WidgetElement::ZStack(ZStackElement { style, .. })
        | WidgetElement::Grid(GridElement { style, .. })
        | WidgetElement::Container(ContainerElement { style, .. })
        | WidgetElement::Text(TextElement { style, .. })
        | WidgetElement::Image(ImageElement { style, .. })
        | WidgetElement::Progress(ProgressElement { style, .. })
        | WidgetElement::Gauge(GaugeElement { style, .. })
        | WidgetElement::Button(ButtonElement { style, .. })
        | WidgetElement::Toggle(ToggleElement { style, .. })
        | WidgetElement::Divider(DividerElement { style, .. })
        | WidgetElement::Date(DateElement { style, .. })
        | WidgetElement::Chart(ChartElement { style, .. })
        | WidgetElement::List(ListElement { style, .. })
        | WidgetElement::Link(LinkElement { style, .. })
        | WidgetElement::Shape(ShapeElement { style, .. })
        | WidgetElement::Timer(TimerElement { style, .. })
        | WidgetElement::Canvas(CanvasElement { style, .. })
        | WidgetElement::Label(LabelElement { style, .. }) => style.flex,
        WidgetElement::Spacer(_) => None,
    }
}

fn push_id_token(obj: &mut Value, token: &str) {
    match obj.get("id").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => {
            if id.split(';').any(|p| p == token) {
                return;
            }
            obj["id"] = json!(format!("{id};{token}"));
        }
        _ => {
            obj["id"] = json!(token);
        }
    }
}

pub(super) fn apply_gap_id(obj: &mut Value, spacing: Option<f64>) {
    let Some(s) = spacing else {
        return;
    };
    if s <= 0.0 {
        return;
    }
    // Adaptive Cards layout spacing (pt buckets) — keep gap id for debug tooling.
    let ac = if s < 4.0 {
        "Small"
    } else if s < 10.0 {
        "Default"
    } else if s < 16.0 {
        "Medium"
    } else {
        "Large"
    };
    obj["spacing"] = json!(ac);
    push_id_token(obj, &format!("gap:{}", s.round() as i64));
}

pub(super) fn apply_container_style(obj: &mut Value, style: &ElementStyle) {
    if let Some(bg) = &style.background {
        match bg {
            crate::models::BackgroundValue::Gradient(g) => {
                if let Ok(uri) = crate::rasterize::gradient_to_png_data_uri(g) {
                    obj["backgroundImage"] = json!({
                        "url": uri,
                        "fillMode": "cover",
                    });
                } else {
                    // Fallback when rasterize is off: emphasis + first stop in id.
                    obj["style"] = json!("emphasis");
                    if let Some(hex) = background_hex(bg) {
                        push_id_token(obj, &format!("card:{hex}"));
                    }
                }
            }
            _ => {
                // Adaptive Cards only has emphasis / good / attention / warning / accent / default.
                obj["style"] = json!("emphasis");
                if let Some(hex) = background_hex(bg) {
                    push_id_token(obj, &format!("card:{hex}"));
                }
            }
        }
    }
}

pub(super) fn apply_container_style_to_wrapper(obj: &mut Value, style: &ElementStyle) {
    // Rasterized zstack Image cannot take Container style; still stamp color id for composites.
    if let Some(bg) = &style.background {
        if let Some(hex) = background_hex(bg) {
            push_id_token(obj, &format!("card:{hex}"));
        }
    }
}

fn background_hex(bg: &crate::models::BackgroundValue) -> Option<String> {
    match bg {
        crate::models::BackgroundValue::Solid(s) => {
            let s = s.trim();
            if s.starts_with('#') {
                Some(s.to_ascii_lowercase())
            } else {
                None
            }
        }
        crate::models::BackgroundValue::Adaptive { dark, .. } => {
            let s = dark.trim();
            if s.starts_with('#') {
                Some(s.to_ascii_lowercase())
            } else {
                None
            }
        }
        crate::models::BackgroundValue::Gradient(g) => g
            .colors
            .first()
            .map(|c| c.trim().to_ascii_lowercase())
            .filter(|c| c.starts_with('#')),
    }
}

pub(super) fn size_bucket(font_size: Option<f64>) -> &'static str {
    match font_size {
        Some(s) if s <= 12.0 => "Small",
        Some(s) if s <= 16.0 => "Default",
        Some(s) if s <= 22.0 => "Medium",
        Some(s) if s <= 32.0 => "Large",
        Some(_) => "ExtraLarge",
        None => "Default",
    }
}

pub(super) fn img_size_bucket(size: Option<f64>) -> &'static str {
    match size {
        Some(s) if s <= 24.0 => "Small",
        Some(s) if s <= 48.0 => "Medium",
        Some(_) => "Large",
        None => "Medium",
    }
}

pub(super) fn weight_token(w: Option<&FontWeight>) -> &'static str {
    match w {
        Some(FontWeight::Bold)
        | Some(FontWeight::Semibold)
        | Some(FontWeight::Heavy)
        | Some(FontWeight::Black) => "Bolder",
        Some(FontWeight::Light) | Some(FontWeight::Thin) | Some(FontWeight::Ultralight) => {
            "Lighter"
        }
        _ => "Default",
    }
}

pub(super) fn align(a: Option<&TextAlignment>) -> Option<&'static str> {
    match a {
        Some(TextAlignment::Leading) => Some("Left"),
        Some(TextAlignment::Center) => Some("Center"),
        Some(TextAlignment::Trailing) => Some("Right"),
        None => None,
    }
}

/// Map IR colors to Adaptive Card semantic tokens; hex → omit (caller may approx).
pub(super) fn ac_color(color: Option<&ColorValue>) -> Option<&'static str> {
    let raw = match color? {
        ColorValue::Solid(s) => s.as_str(),
        ColorValue::Adaptive { light, .. } => light.as_str(),
    };
    let lower = raw.to_ascii_lowercase();
    match lower.as_str() {
        "accent" => Some("Accent"),
        "good" | "success" => Some("Good"),
        "warning" => Some("Warning"),
        "attention" | "error" | "danger" => Some("Attention"),
        // "Dark" text on dark Widgets Board / goldens is invisible — use Default.
        "label" | "dark" => Some("Default"),
        "secondarylabel" | "light" => Some("Light"),
        "default" => Some("Default"),
        s if s.starts_with('#') => None,
        _ => None,
    }
}

pub(super) fn color_hex(color: Option<&ColorValue>) -> Option<String> {
    let raw = match color? {
        ColorValue::Solid(s) => s.as_str(),
        ColorValue::Adaptive { light, .. } => light.as_str(),
    };
    let s = raw.trim();
    if s.starts_with('#') && (s.len() == 7 || s.len() == 4) {
        Some(s.to_ascii_lowercase())
    } else {
        None
    }
}

/// Approximate hex → AC semantic (for labels / progress style hints).
pub(super) fn approx_hex_semantic(color: Option<&ColorValue>) -> Option<&'static str> {
    let hex = color_hex(color)?;
    let (r, g, b) = parse_hex_rgb(&hex)?;
    let max = r.max(g).max(b) as f32;
    let min = r.min(g).min(b) as f32;
    let luma = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    let sat = max - min;
    // Near-black → light text on dark goldens.
    if luma < 60.0 {
        return Some("Light");
    }
    // Low-saturation slate/gray (e.g. #94a3b8) → Light, not Accent.
    if sat < 50.0 {
        return Some("Light");
    }
    if luma > 200.0 && sat < 30.0 {
        return Some("Light");
    }
    let (rf, gf, bf) = (r as f32, g as f32, b as f32);
    if rf > gf + 30.0 && rf > bf + 30.0 {
        return Some("Attention");
    }
    if gf > rf + 20.0 && gf > bf + 20.0 {
        return Some("Good");
    }
    if bf > rf + 20.0 && bf > gf + 10.0 {
        return Some("Accent");
    }
    if rf > 180.0 && gf > 120.0 && bf < 100.0 {
        return Some("Warning");
    }
    Some("Default")
}

fn parse_hex_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let h = hex.trim_start_matches('#');
    if h.len() == 3 {
        let r = u8::from_str_radix(&h[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&h[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&h[2..3].repeat(2), 16).ok()?;
        return Some((r, g, b));
    }
    if h.len() == 6 {
        let r = u8::from_str_radix(&h[0..2], 16).ok()?;
        let g = u8::from_str_radix(&h[2..4], 16).ok()?;
        let b = u8::from_str_radix(&h[4..6], 16).ok()?;
        return Some((r, g, b));
    }
    None
}

/// Media / SF-ish glyphs often missing in SVG fonts → ASCII stand-ins.
pub(super) fn sanitize_button_label(label: &str) -> String {
    let t = label.trim();
    match t {
        "⏮" | "⏮️" => "Prev".into(),
        "⏭" | "⏭️" => "Next".into(),
        "⏸" | "⏸️" => "Pause".into(),
        "▶" | "▶️" | "⏯" => "Play".into(),
        "⏹" | "⏹️" => "Stop".into(),
        "⌫" => "BS".into(),
        "±" => "+/-".into(),
        "÷" => "/".into(),
        "×" => "x".into(),
        "−" => "-".into(),
        _ => {
            let mut s = t.to_string();
            for (from, to) in [
                ("⏮", "Prev"),
                ("⏭", "Next"),
                ("⏸", "Pause"),
                ("▶", "Play"),
                ("⌫", "BS"),
                ("±", "+/-"),
                ("÷", "/"),
                ("×", "x"),
                ("−", "-"),
            ] {
                if s.contains(from) {
                    s = s.replace(from, to);
                }
            }
            s
        }
    }
}

pub(super) fn sf_symbol_glyph(name: &str) -> &'static str {
    crate::icons::sf_symbol_emoji(name)
}

/// Shared SF Symbol → emoji/glyph map for Adaptive Cards and rasterized zstacks.
pub(crate) fn sf_symbol_glyph_public(name: &str) -> &'static str {
    crate::icons::sf_symbol_emoji(name)
}
