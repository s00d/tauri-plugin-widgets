//! Data group: progress / gauge / chart / list.

use crate::models::{ListElement, ProgressElement, ProgressStyle, WidgetElement};
use crate::receipt::SkippedElement;
use serde_json::{json, Value};

use super::media::rasterized_or_skip;
use super::style::{ac_color, approx_hex_semantic, color_hex};

pub(super) fn progress(
    e: &WidgetElement,
    p: &ProgressElement,
    skipped: &mut Vec<SkippedElement>,
) -> Value {
    // startAt/endAt: value at transpile time (the host rebuilds the card on its timers).
    if let Some(f) = super::time_fraction(p.start_at, p.end_at) {
        let mut q = p.clone();
        q.total = if q.total <= 0.0 { 1.0 } else { q.total };
        q.value = f * q.total;
        (q.start_at, q.end_at) = (None, None);
        return progress(&WidgetElement::Progress(q.clone()), &q, skipped);
    }
    // Adaptive Cards has no circular progress — rasterize to preserve shape.
    if matches!(p.bar_style, Some(ProgressStyle::Circular)) {
        return rasterized_or_skip(e, skipped);
    }
    // trackColor / thickness: a column bar cannot draw them — rasterized bar.
    if p.track_color.is_some() || p.thickness.is_some() {
        if let Ok(uri) = crate::rasterize::element_to_png_data_uri(e) {
            let mut items = Vec::new();
            if let Some(l) = &p.label {
                items.push(progress_label(l, p));
            }
            items.push(json!({"type": "Image", "url": uri, "size": "Stretch"}));
            return json!({"type": "Container", "items": items});
        }
    }
    let ProgressElement {
        value,
        total,
        label,
        tint,
        ..
    } = p;
    let total = if *total <= 0.0 { 1.0 } else { *total };
    let pct = ((*value / total) * 100.0).clamp(0.0, 100.0) as u32;
    let rest = 100u32.saturating_sub(pct);
    let mut items = Vec::new();
    if let Some(l) = label {
        items.push(progress_label(l, p));
    }
    let fill_hex = color_hex(tint.as_ref());
    let fill_style = approx_hex_semantic(tint.as_ref())
        .unwrap_or("Good")
        .to_ascii_lowercase();
    // Empty columns have no height of their own (renderers drew a pale block or nothing):
    // `minHeight` makes the bar, a repeated 1x1 PNG gives the exact fill / track colour.
    let fill = fill_hex.clone().unwrap_or_else(|| "#4caf50".into());
    let solid = |hex: &str| {
        crate::rasterize::solid_png_data_uri(hex).map(|u| json!({ "url": u, "fillMode": "repeat" }))
    };
    let mut filled = json!({
        "type": "Column",
        "width": pct.max(1),
        "minHeight": "6px",
        "items": []
    });
    match solid(&fill) {
        Some(bg) => filled["backgroundImage"] = bg,
        // A style paints its own padded background around the bar: only without the image.
        None => filled["style"] = json!(fill_style),
    }
    if let Some(ref hex) = fill_hex {
        filled["id"] = json!(format!("fill:{hex}"));
    }
    let mut track = json!({
        "type": "Column",
        "width": rest.max(1),
        "spacing": "None",
        "minHeight": "6px",
        "items": []
    });
    // Track: the fill colour at 25 % (as the desktop / Apple renderers draw it).
    if let Some(bg) = solid(&format!("{}40", &fill[..7.min(fill.len())])) {
        track["backgroundImage"] = bg;
    }
    items.push(json!({
        "type": "ColumnSet",
        "spacing": "Small",
        "columns": [filled, track],
    }));
    json!({
        "type": "Container",
        "items": items,
    })
}

fn progress_label(l: &str, p: &ProgressElement) -> Value {
    let ProgressElement { tint, color, .. } = p;
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
    tb
}

pub(super) fn gauge(e: &WidgetElement, skipped: &mut Vec<SkippedElement>) -> Value {
    if let WidgetElement::Gauge(g) = e {
        if let Some(f) = super::time_fraction(g.start_at, g.end_at) {
            let mut g = g.clone();
            let (lo, hi) = (g.min.unwrap_or(0.0), g.max.unwrap_or(1.0));
            g.value = lo + f * (hi - lo);
            return rasterized_or_skip(&WidgetElement::Gauge(g), skipped);
        }
    }
    rasterized_or_skip(e, skipped)
}

pub(super) fn chart(e: &WidgetElement, skipped: &mut Vec<SkippedElement>) -> Value {
    rasterized_or_skip(e, skipped)
}

pub(super) fn list(l: &ListElement) -> Value {
    let ListElement { items, .. } = l;
    let rows: Vec<Value> = items
        .iter()
        .map(|it| {
            let (mark, mark_color) = match it.checked {
                Some(true) => ("✓", "Good"),
                Some(false) => ("○", "Light"),
                None => (" ", "Light"),
            };
            let mut mark_cell = json!({
                "type": "TableCell",
                "verticalContentAlignment": "Center",
                "items": [{
                    "type": "TextBlock",
                    "text": mark,
                    "wrap": false,
                    "size": "Small",
                    "color": mark_color,
                    "horizontalAlignment": "Center",
                }]
            });
            let mut text_cell = json!({
                "type": "TableCell",
                "verticalContentAlignment": "Center",
                "items": [{
                    "type": "TextBlock",
                    "text": it.text,
                    "wrap": true,
                    "size": "Small",
                    "horizontalAlignment": "Left",
                    "color": "Light",
                }]
            });
            if let Some(ref a) = it.action {
                let action = json!({
                    "type": "Action.Execute",
                    "verb": a,
                    "data": { "payload": it.payload },
                });
                text_cell["selectAction"] = action.clone();
                mark_cell["selectAction"] = action;
            }
            json!({
                "type": "TableRow",
                "cells": [mark_cell, text_cell],
            })
        })
        .collect();
    json!({
        "type": "Table",
        "firstRowAsHeaders": false,
        "showGridLines": false,
        "columns": [
            { "width": 1 },
            { "width": 5 }
        ],
        "rows": rows,
    })
}
