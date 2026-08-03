//! Layout group: vstack / hstack / zstack / grid / container.

use crate::models::{
    ContainerElement, GridElement, HStackElement, VStackElement, WidgetElement, ZStackElement,
};
use crate::receipt::SkippedElement;
use serde_json::{json, Value};

use super::el;
use super::style::{
    apply_container_style, apply_container_style_to_wrapper, apply_gap_id, flex_width,
};

pub(super) fn vstack(v: &VStackElement, skipped: &mut Vec<SkippedElement>) -> Value {
    let VStackElement {
        children,
        style,
        alignment,
        spacing,
        ..
    } = v;
    let items: Vec<Value> = children
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let mut item = el(c, skipped);
            if i > 0 {
                apply_gap_id(&mut item, *spacing);
            }
            item
        })
        .collect();
    let mut obj = json!({
        "type": "Container",
        "spacing": "None",
        "items": items,
    });
    apply_container_style(&mut obj, style);
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

pub(super) fn hstack(h: &HStackElement, skipped: &mut Vec<SkippedElement>) -> Value {
    let HStackElement {
        children, spacing, ..
    } = h;
    let n = children.len();
    json!({
        "type": "ColumnSet",
        "spacing": "None",
        "columns": children
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let (width, items) = match c {
                    WidgetElement::Spacer(_) => (json!("stretch"), json!([])),
                    // Column immediately before a spacer expands (title + Sync pattern).
                    _ if i + 1 < n
                        && matches!(children[i + 1], WidgetElement::Spacer(_)) =>
                    {
                        (json!("stretch"), json!([el(c, skipped)]))
                    }
                    _ => (flex_width(c), json!([el(c, skipped)])),
                };
                let mut col = json!({
                    "type": "Column",
                    "width": width,
                    "items": items,
                });
                // Inter-column gap (ColumnSet.spacing is preceding-element only).
                if i > 0 {
                    apply_gap_id(&mut col, *spacing);
                }
                col
            })
            .collect::<Vec<_>>(),
    })
}

pub(super) fn container(c: &ContainerElement, skipped: &mut Vec<SkippedElement>) -> Value {
    let ContainerElement {
        children,
        style,
        content_alignment,
        ..
    } = c;
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

pub(super) fn grid(g: &GridElement, skipped: &mut Vec<SkippedElement>) -> Value {
    let GridElement {
        children,
        columns,
        spacing,
        row_spacing,
        ..
    } = g;
    let cols = (*columns).max(1) as usize;
    let mut rows = Vec::new();
    for (ri, chunk) in children.chunks(cols).enumerate() {
        let columns_json: Vec<Value> = chunk
            .iter()
            .enumerate()
            .map(|(ci, c)| {
                let mut col = json!({
                    "type": "Column",
                    "width": "stretch",
                    "items": [el(c, skipped)],
                });
                // Horizontal gaps between columns within a row.
                if ci > 0 {
                    apply_gap_id(&mut col, *spacing);
                }
                col
            })
            .collect();
        let mut row = json!({
            "type": "ColumnSet",
            "spacing": "None",
            "columns": columns_json,
        });
        // Vertical gaps between rows.
        if ri > 0 {
            apply_gap_id(&mut row, row_spacing.or(*spacing));
        }
        rows.push(row);
    }
    json!({
        "type": "Container",
        "spacing": "None",
        "items": rows,
    })
}

/// `e` is the original node — rasterizing an overlay needs the whole element.
pub(super) fn zstack(
    e: &WidgetElement,
    z: &ZStackElement,
    skipped: &mut Vec<SkippedElement>,
) -> Value {
    let ZStackElement {
        children,
        style,
        alignment,
        ..
    } = z;
    // Prefer rasterized overlay approximation when `rasterize` works.
    if let Ok(uri) = crate::rasterize::element_to_png_data_uri(e) {
        let mut img = json!({
            "type": "Image",
            "url": uri,
            "size": "Stretch",
            "horizontalAlignment": "Center",
        });
        apply_container_style_to_wrapper(&mut img, style);
        return img;
    }
    // Fallback: no true overlay — flatten; keep center hint for composite.
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
    if let Some(items) = obj.get("items").and_then(|i| i.as_array()) {
        if items.len() == 2 {
            obj["id"] = json!("badge:overlay");
        }
    }
    obj
}
