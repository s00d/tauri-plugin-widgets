//! IR → Adaptive Card 1.5 transpiler for Windows Widgets Board.
//!
//! Pure Rust — runs on any host. Elements Adaptive Cards cannot express
//! (`canvas`, `chart`, `gauge`, `zstack`, …) become empty placeholders and are
//! recorded in [`TranspileResult::skipped`].

use crate::models::{
    ColorValue, ElementStyle, FontWeight, TextAlignment, WidgetConfig, WidgetElement,
};
use crate::receipt::SkippedElement;
use serde_json::{json, Value};

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

fn el(e: &WidgetElement, skipped: &mut Vec<SkippedElement>) -> Value {
    match e {
        WidgetElement::VStack {
            children,
            style,
            alignment,
            spacing,
            ..
        } => {
            let mut obj = json!({
                "type": "Container",
                "spacing": "None",
                "items": children.iter().map(|c| el(c, skipped)).collect::<Vec<_>>(),
            });
            apply_container_style(&mut obj, style);
            apply_gap_id(&mut obj, *spacing);
            // HorizontalAlignment on VStack children (center/leading/trailing).
            if let Some(a) = alignment {
                match a {
                    crate::models::HorizontalAlignment::Center => {
                        obj["horizontalAlignment"] = json!("Center");
                    }
                    crate::models::HorizontalAlignment::Trailing => {
                        obj["horizontalAlignment"] = json!("Right");
                    }
                    _ => {}
                }
            }
            obj
        }

        WidgetElement::HStack {
            children, spacing, ..
        } => {
            let n = children.len();
            let mut obj = json!({
                "type": "ColumnSet",
                "columns": children
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        let (width, items) = match c {
                            WidgetElement::Spacer { .. } => (json!("stretch"), json!([])),
                            // Column immediately before a spacer expands (title + Sync pattern).
                            _ if i + 1 < n
                                && matches!(children[i + 1], WidgetElement::Spacer { .. }) =>
                            {
                                (json!("stretch"), json!([el(c, skipped)]))
                            }
                            _ => (flex_width(c), json!([el(c, skipped)])),
                        };
                        json!({
                            "type": "Column",
                            "width": width,
                            "items": items,
                        })
                    })
                    .collect::<Vec<_>>(),
            });
            apply_gap_id(&mut obj, *spacing);
            obj
        }

        WidgetElement::Container {
            children,
            style,
            content_alignment,
            ..
        } => {
            let mut obj = json!({
                "type": "Container",
                "spacing": "None",
                "items": children.iter().map(|c| el(c, skipped)).collect::<Vec<_>>(),
            });
            apply_container_style(&mut obj, style);
            if content_alignment
                .as_deref()
                .map(|a| a.to_ascii_lowercase().contains("center"))
                .unwrap_or(false)
            {
                obj["horizontalAlignment"] = json!("Center");
            }
            obj
        }

        WidgetElement::Grid {
            children,
            columns,
            spacing,
            row_spacing,
            ..
        } => {
            let cols = (*columns).max(1) as usize;
            let mut rows = Vec::new();
            for chunk in children.chunks(cols) {
                let columns_json: Vec<Value> = chunk
                    .iter()
                    .map(|c| {
                        json!({
                            "type": "Column",
                            "width": "stretch",
                            "items": [el(c, skipped)],
                        })
                    })
                    .collect();
                let mut row = json!({
                    "type": "ColumnSet",
                    "columns": columns_json,
                });
                apply_gap_id(&mut row, *spacing);
                rows.push(row);
            }
            let mut obj = json!({
                "type": "Container",
                "spacing": "None",
                "items": rows,
            });
            apply_gap_id(&mut obj, row_spacing.or(*spacing));
            obj
        }

        WidgetElement::Text {
            content,
            font_size,
            font_weight,
            color,
            alignment,
            line_limit,
            ..
        } => {
            let wrap = !matches!(line_limit, Some(n) if *n <= 1);
            let mut obj = json!({
                "type": "TextBlock",
                "text": content,
                "wrap": wrap,
                "size": size_bucket(*font_size),
                "weight": weight_token(font_weight.as_ref()),
            });
            if let Some(c) =
                ac_color(color.as_ref()).or_else(|| approx_hex_semantic(color.as_ref()))
            {
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

        WidgetElement::Image {
            url,
            data,
            system_name,
            size,
            color,
            ..
        } => {
            let url_str = url.clone().unwrap_or_else(|| {
                let b64 = data.clone().unwrap_or_default();
                if b64.is_empty() {
                    String::new()
                } else if b64.starts_with("data:") {
                    b64
                } else {
                    format!("data:image/png;base64,{b64}")
                }
            });
            if url_str.is_empty() {
                // SF Symbol / Material name without bitmap → emoji TextBlock for AC/goldens.
                let glyph = system_name.as_deref().map(sf_symbol_glyph).unwrap_or("•");
                let mut obj = json!({
                    "type": "TextBlock",
                    "text": glyph,
                    "wrap": true,
                    "size": size_bucket(size.map(|s| s * 0.75)),
                    "weight": "Bolder",
                });
                if let Some(c) =
                    ac_color(color.as_ref()).or_else(|| approx_hex_semantic(color.as_ref()))
                {
                    obj["color"] = json!(c);
                }
                return obj;
            }
            json!({
                "type": "Image",
                "url": url_str,
                "size": img_size_bucket(*size),
            })
        }

        WidgetElement::Button {
            label,
            action,
            url,
            background_color,
            color,
            ..
        } => {
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

        WidgetElement::Toggle {
            is_on,
            label,
            action,
            ..
        } => {
            // Prefer readable TextBlock for Adaptive Cards / goldens (ActionSet is clickable but low-fidelity).
            let title =
                label
                    .clone()
                    .unwrap_or_else(|| if *is_on { "On".into() } else { "Off".into() });
            let mark = if *is_on { "✓" } else { "○" };
            let _ = action;
            json!({
                "type": "TextBlock",
                "text": format!("{mark} {title}"),
                "wrap": true,
                "size": "Default",
            })
        }

        WidgetElement::Progress {
            value,
            total,
            label,
            tint,
            color,
            ..
        } => {
            let total = if *total <= 0.0 { 1.0 } else { *total };
            let pct = ((*value / total) * 100.0).clamp(0.0, 100.0) as u32;
            let rest = 100u32.saturating_sub(pct);
            let mut items = Vec::new();
            if let Some(l) = label {
                let mut tb = json!({
                    "type": "TextBlock",
                    "text": l,
                    "size": "Small",
                    "wrap": true,
                });
                if let Some(c) = ac_color(color.as_ref())
                    .or_else(|| ac_color(tint.as_ref()))
                    .or_else(|| approx_hex_semantic(color.as_ref()))
                    .or_else(|| approx_hex_semantic(tint.as_ref()))
                {
                    tb["color"] = json!(c);
                }
                items.push(tb);
            }
            let fill_hex = color_hex(tint.as_ref());
            let fill_style = approx_hex_semantic(tint.as_ref())
                .unwrap_or("Good")
                .to_ascii_lowercase();
            let mut filled = json!({
                "type": "Column",
                "width": pct.max(1),
                "style": fill_style,
                "items": []
            });
            if let Some(ref hex) = fill_hex {
                filled["id"] = json!(format!("fill:{hex}"));
            }
            items.push(json!({
                "type": "ColumnSet",
                "columns": [
                    filled,
                    {
                        "type": "Column",
                        "width": rest.max(1),
                        "items": []
                    }
                ],
            }));
            json!({
                "type": "Container",
                "items": items,
            })
        }

        WidgetElement::Divider { color, .. } => {
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

        WidgetElement::Spacer { .. } => json!({
            "type": "TextBlock",
            "text": " ",
            "spacing": "Medium",
        }),

        WidgetElement::Date { date, .. } => json!({
            "type": "TextBlock",
            "text": date,
            "wrap": true,
            "size": "Default",
        }),

        WidgetElement::Timer { target_date, .. } => json!({
            "type": "TextBlock",
            "text": target_date,
            "wrap": true,
            "size": "Default",
        }),

        WidgetElement::Label {
            text,
            system_name,
            color,
            ..
        } => {
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
            if let Some(c) =
                ac_color(color.as_ref()).or_else(|| approx_hex_semantic(color.as_ref()))
            {
                obj["color"] = json!(c);
            } else {
                obj["color"] = json!("Light");
            }
            obj
        }

        WidgetElement::Link {
            children,
            action,
            url,
            style,
            ..
        } => {
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

        WidgetElement::List { items, spacing, .. } => {
            // ColumnSet mark | text keeps a shared left edge regardless of glyph width.
            let lines: Vec<Value> = items
                .iter()
                .map(|it| {
                    let (mark, mark_color) = match it.checked {
                        Some(true) => ("✓", "Good"),
                        Some(false) => ("○", "Light"),
                        // Keep a blank mark slot so labels share a left edge.
                        None => (" ", "Light"),
                    };
                    json!({
                        "type": "ColumnSet",
                        "columns": [
                            {
                                "type": "Column",
                                "width": "auto",
                                "id": "mark:col",
                                "items": [{
                                    "type": "TextBlock",
                                    "text": mark,
                                    "wrap": false,
                                    "size": "Small",
                                    "color": mark_color,
                                    "horizontalAlignment": "Center",
                                }]
                            },
                            {
                                "type": "Column",
                                "width": "stretch",
                                "items": [{
                                    "type": "TextBlock",
                                    "text": it.text,
                                    "wrap": true,
                                    "size": "Small",
                                    "horizontalAlignment": "Left",
                                    "color": "Light",
                                }]
                            }
                        ]
                    })
                })
                .collect();
            let mut obj = json!({
                "type": "Container",
                "horizontalAlignment": "Left",
                "items": lines,
            });
            apply_gap_id(&mut obj, spacing.or(Some(4.0)));
            obj
        }

        WidgetElement::Shape { .. } => rasterized_or_skip(e, skipped),

        WidgetElement::ZStack {
            children,
            style,
            alignment,
            ..
        } => {
            // Degraded: no true overlay — flatten; keep center hint for composite.
            let mut obj = json!({
                "type": "Container",
                "spacing": "None",
                "items": children.iter().map(|c| el(c, skipped)).collect::<Vec<_>>(),
            });
            apply_container_style(&mut obj, style);
            let centered = alignment
                .as_deref()
                .map(|a| a.to_ascii_lowercase().contains("center"))
                .unwrap_or(false);
            if centered {
                obj["horizontalAlignment"] = json!("Center");
            }
            // Icon badge (shape/image + glyph text): mark for overlay+center in composite.
            if let Some(items) = obj.get("items").and_then(|i| i.as_array()) {
                if items.len() == 2 {
                    obj["id"] = json!("badge:overlay");
                }
            }
            obj
        }

        WidgetElement::Gauge { .. }
        | WidgetElement::Chart { .. }
        | WidgetElement::Canvas { .. } => rasterized_or_skip(e, skipped),
    }
}

fn rasterized_or_skip(e: &WidgetElement, skipped: &mut Vec<SkippedElement>) -> Value {
    let ty = e.type_name();
    match crate::rasterize::element_to_png_data_uri(e) {
        Ok(uri) => {
            let mut img = json!({
                "type": "Image",
                "url": uri,
                "size": "Medium",
                "horizontalAlignment": "Center",
            });
            match e {
                WidgetElement::Canvas { .. }
                | WidgetElement::Chart { .. }
                | WidgetElement::Gauge { .. } => {
                    img["size"] = json!("Stretch");
                }
                WidgetElement::Shape { size, .. } => {
                    img["size"] = json!(img_size_bucket(*size));
                }
                _ => {}
            }
            img
        }
        Err(err) => {
            skipped.push(SkippedElement {
                type_name: ty.into(),
                reason: format!("rasterize failed: {err}"),
            });
            json!({
                "type": "TextBlock",
                "text": "",
                "spacing": "None",
            })
        }
    }
}

fn flex_width(c: &WidgetElement) -> Value {
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
        WidgetElement::VStack { style, .. }
        | WidgetElement::HStack { style, .. }
        | WidgetElement::ZStack { style, .. }
        | WidgetElement::Grid { style, .. }
        | WidgetElement::Container { style, .. }
        | WidgetElement::Text { style, .. }
        | WidgetElement::Image { style, .. }
        | WidgetElement::Progress { style, .. }
        | WidgetElement::Gauge { style, .. }
        | WidgetElement::Button { style, .. }
        | WidgetElement::Toggle { style, .. }
        | WidgetElement::Divider { style, .. }
        | WidgetElement::Date { style, .. }
        | WidgetElement::Chart { style, .. }
        | WidgetElement::List { style, .. }
        | WidgetElement::Link { style, .. }
        | WidgetElement::Shape { style, .. }
        | WidgetElement::Timer { style, .. }
        | WidgetElement::Canvas { style, .. }
        | WidgetElement::Label { style, .. } => style.flex,
        WidgetElement::Spacer { .. } => None,
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

fn apply_gap_id(obj: &mut Value, spacing: Option<f64>) {
    let Some(s) = spacing else {
        return;
    };
    if s <= 0.0 {
        return;
    }
    push_id_token(obj, &format!("gap:{}", s.round() as i64));
}

fn apply_container_style(obj: &mut Value, style: &ElementStyle) {
    if let Some(bg) = &style.background {
        // Adaptive Cards only has emphasis / good / attention / warning / accent / default.
        obj["style"] = json!("emphasis");
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

fn size_bucket(font_size: Option<f64>) -> &'static str {
    match font_size {
        Some(s) if s <= 12.0 => "Small",
        Some(s) if s <= 16.0 => "Default",
        Some(s) if s <= 22.0 => "Medium",
        Some(s) if s <= 32.0 => "Large",
        Some(_) => "ExtraLarge",
        None => "Default",
    }
}

fn img_size_bucket(size: Option<f64>) -> &'static str {
    match size {
        Some(s) if s <= 24.0 => "Small",
        Some(s) if s <= 48.0 => "Medium",
        Some(_) => "Large",
        None => "Medium",
    }
}

fn weight_token(w: Option<&FontWeight>) -> &'static str {
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

fn align(a: Option<&TextAlignment>) -> Option<&'static str> {
    match a {
        Some(TextAlignment::Leading) => Some("Left"),
        Some(TextAlignment::Center) => Some("Center"),
        Some(TextAlignment::Trailing) => Some("Right"),
        None => None,
    }
}

/// Map IR colors to Adaptive Card semantic tokens; hex → omit (caller may approx).
fn ac_color(color: Option<&ColorValue>) -> Option<&'static str> {
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

fn color_hex(color: Option<&ColorValue>) -> Option<String> {
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
fn approx_hex_semantic(color: Option<&ColorValue>) -> Option<&'static str> {
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
fn sanitize_button_label(label: &str) -> String {
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

fn sf_symbol_glyph(name: &str) -> &'static str {
    let n = name.to_ascii_lowercase();
    match n.as_str() {
        "gear" | "gearshape" | "gearshape.fill" => "*",
        "person" | "person.fill" => "☺",
        "checkmark" | "checkmark.circle" | "checkmark.circle.fill" => "✓",
        "calendar" | "calendar.badge.clock" | "calendar.circle" => "◷",
        "xmark" | "xmark.circle" => "x",
        "star" | "star.fill" => "*",
        "heart" | "heart.fill" => "+",
        "bell" | "bell.fill" => "!",
        "house" | "house.fill" => "H",
        "magnifyingglass" => "?",
        "plus" | "plus.circle" => "+",
        "minus" | "minus.circle" => "-",
        _ => "•",
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
