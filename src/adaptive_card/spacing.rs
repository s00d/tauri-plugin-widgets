//! Spacing group: spacer / divider.

use crate::models::DividerElement;
use serde_json::{json, Value};

use super::style::color_hex;

pub(super) fn spacer() -> Value {
    json!({
        "type": "TextBlock",
        "text": " ",
        "spacing": "Medium",
    })
}

pub(super) fn divider(d: &DividerElement) -> Value {
    let DividerElement { color, .. } = d;
    let mut obj = json!({
        "type": "Container",
        "separator": true,
        "spacing": "Medium",
        "items": [],
    });
    if let Some(hex) = color_hex(color.as_ref()) {
        obj["id"] = json!(format!("rule:{hex}"));
    } else {
        obj["id"] = json!("rule:#334155");
    }
    obj
}
