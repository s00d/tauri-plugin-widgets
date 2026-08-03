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
    // Adaptive Cards has no circular progress — rasterize to preserve shape.
    if matches!(p.bar_style, Some(ProgressStyle::Circular)) {
        return rasterized_or_skip(e, skipped);
    }
    let ProgressElement {
        value,
        total,
        label,
        tint,
        color,
        ..
    } = p;
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

pub(super) fn gauge(e: &WidgetElement, skipped: &mut Vec<SkippedElement>) -> Value {
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
