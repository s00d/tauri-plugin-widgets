//! Interactive group: button / toggle / link.

use crate::models::{ButtonElement, ButtonVariant, LinkElement, ToggleElement, ToggleVariant};
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
        variant,
        system_name,
        disabled,
        ..
    } = b;
    let mut title = sanitize_button_label(label);
    // Icon: rasterized glyph as `iconUrl`; without `rasterize`, a glyph before the title.
    let icon = system_name.as_deref().filter(|n| !n.is_empty());
    let icon_url = icon.and_then(|_n| {
        #[cfg(feature = "rasterize")]
        {
            let fg = color_hex(color.as_ref()).unwrap_or_else(|| "#FFFFFF".into());
            crate::icons::icon_png_data_uri(_n, 16.0, &fg).ok()
        }
        #[cfg(not(feature = "rasterize"))]
        None::<String>
    });
    if let (Some(n), None) = (icon, &icon_url) {
        title = format!("{} {title}", crate::icons::sf_symbol_emoji(n));
    }
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
    // AC 1.5 action styles: positive (accent fill) / default; bordered, plain, outline → default.
    if *variant == Some(ButtonVariant::Prominent) {
        action_json["style"] = json!("positive");
    }
    if let Some(uri) = icon_url {
        action_json["iconUrl"] = json!(uri);
    }
    if *disabled == Some(true) {
        action_json["isEnabled"] = json!(false);
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
        variant,
        ..
    } = t;
    // Prefer readable TextBlock for Adaptive Cards / goldens (ActionSet is clickable but low-fidelity).
    let title = label
        .clone()
        .unwrap_or_else(|| if *is_on { "On".into() } else { "Off".into() });
    let mark = match (variant, is_on) {
        (Some(ToggleVariant::Checkbox), true) => "☑",
        (Some(ToggleVariant::Checkbox), false) => "☐",
        (_, true) => "✓",
        (_, false) => "○",
    };
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
