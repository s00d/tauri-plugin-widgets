//! Text group: text / date / timer / label.

use crate::models::{DateElement, LabelElement, TextElement, TimerElement};
use serde_json::{json, Value};

use super::style::{
    ac_color, align, approx_hex_semantic, color_hex, sf_symbol_glyph, size_bucket, weight_token,
};

pub(super) fn text(t: &TextElement) -> Value {
    let TextElement {
        content,
        font_size,
        font_weight,
        color,
        alignment,
        line_limit,
        ..
    } = t;
    let wrap = !matches!(line_limit, Some(n) if *n <= 1);
    let mut obj = json!({
        "type": "TextBlock",
        "text": content,
        "wrap": wrap,
        "size": size_bucket(*font_size),
        "weight": weight_token(font_weight.as_ref()),
    });
    if let Some(c) = ac_color(color.as_ref()).or_else(|| approx_hex_semantic(color.as_ref())) {
        obj["color"] = json!(c);
    }
    // Preserve exact hex for composite (muted slates stay slate, not AC "Light").
    if let Some(hex) = color_hex(color.as_ref()) {
        obj["id"] = json!(format!("fg:{hex}"));
    }
    if let Some(a) = align(alignment.as_ref()) {
        obj["horizontalAlignment"] = json!(a);
    }
    obj
}

pub(super) fn date(d: &DateElement) -> Value {
    let DateElement { date, .. } = d;
    json!({
        "type": "TextBlock",
        "text": date,
        "wrap": true,
        "size": "Default",
    })
}

pub(super) fn timer(t: &TimerElement) -> Value {
    let TimerElement { target_date, .. } = t;
    json!({
        "type": "TextBlock",
        "text": target_date,
        "wrap": true,
        "size": "Default",
    })
}

pub(super) fn label(l: &LabelElement) -> Value {
    let LabelElement {
        text,
        system_name,
        color,
        ..
    } = l;
    let prefix = if system_name.is_empty() {
        String::new()
    } else {
        format!("{} ", sf_symbol_glyph(system_name))
    };
    let mut obj = json!({
        "type": "TextBlock",
        "text": format!("{prefix}{text}"),
        "wrap": true,
        "weight": "Bolder",
        "size": "Small",
    });
    if let Some(c) = ac_color(color.as_ref()).or_else(|| approx_hex_semantic(color.as_ref())) {
        obj["color"] = json!(c);
    } else {
        obj["color"] = json!("Light");
    }
    obj
}
