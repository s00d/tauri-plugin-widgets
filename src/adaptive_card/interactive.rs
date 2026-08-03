//! Interactive group: button / toggle / link.

use crate::models::{ButtonElement, LinkElement, ToggleElement};
use crate::receipt::SkippedElement;
use serde_json::{json, Value};

use super::el;
use super::style::{apply_container_style, color_hex, sanitize_button_label};

pub(super) fn button(b: &ButtonElement) -> Value {
    let ButtonElement {
        label,
        action,
        url,
        background_color,
        color,
        ..
    } = b;
    let title = sanitize_button_label(label);
    let mut action_json = match action {
        Some(a) => json!({
            "type": "Action.Execute",
            "title": title,
            "verb": a,
        }),
        None => json!({
            "type": "Action.OpenUrl",
            "title": title,
            "url": url.clone().unwrap_or_default(),
        }),
    };
    // Composite / PreviewHost: pass fill+fg via id (AC has no arbitrary action colors).
    let mut id_parts = Vec::new();
    if let Some(hex) = color_hex(background_color.as_ref()) {
        id_parts.push(format!("bg:{hex}"));
    }
    if let Some(hex) = color_hex(color.as_ref()) {
        id_parts.push(format!("fg:{hex}"));
    }
    if !id_parts.is_empty() {
        action_json["id"] = json!(id_parts.join(";"));
    }
    json!({
        "type": "ActionSet",
        "actions": [action_json],
    })
}

pub(super) fn toggle(t: &ToggleElement) -> Value {
    let ToggleElement {
        is_on,
        label,
        action,
        ..
    } = t;
    // Prefer readable TextBlock for Adaptive Cards / goldens (ActionSet is clickable but low-fidelity).
    let title = label
        .clone()
        .unwrap_or_else(|| if *is_on { "On".into() } else { "Off".into() });
    let mark = if *is_on { "✓" } else { "○" };
    let mut block = json!({
        "type": "TextBlock",
        "text": format!("{mark} {title}"),
        "wrap": true,
        "size": "Default",
    });
    if let Some(a) = action {
        // Keep toggle tappable on Widgets Board.
        // Payload is the scalar next-state string (matches iOS/Android/desktop).
        return json!({
            "type": "Container",
            "items": [block],
            "selectAction": {
                "type": "Action.Execute",
                "verb": a,
                "data": { "payload": (!is_on).to_string() },
            },
        });
    }
    let _ = &mut block;
    block
}

pub(super) fn link(l: &LinkElement, skipped: &mut Vec<SkippedElement>) -> Value {
    let LinkElement {
        children,
        action,
        url,
        style,
        ..
    } = l;
    let inner: Vec<Value> = children.iter().map(|c| el(c, skipped)).collect();
    let mut container = json!({
        "type": "Container",
        "items": inner,
        "selectAction": match action {
            Some(a) => json!({ "type": "Action.Execute", "verb": a }),
            None => json!({
                "type": "Action.OpenUrl",
                "url": url.clone().unwrap_or_default(),
            }),
        },
    });
    apply_container_style(&mut container, style);
    container
}
