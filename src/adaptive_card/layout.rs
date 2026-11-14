//! Layout group: vstack / hstack / zstack / grid / container.

use crate::models::{
    ContainerElement, GridElement, HStackElement, Justify, VStackElement, WidgetElement,
    ZStackElement,
};
use crate::receipt::SkippedElement;
use serde_json::{json, Value};

use super::el;
use super::style::{
    apply_container_style, apply_container_style_to_wrapper, apply_gap_id, flex_width, priority_of,
};

/// Insert `filler()` blocks around `items` for a `justify` distribution. AC has no
/// main-axis distribution; empty stretch blocks share the free space instead.
/// `spaceAround` is drawn like `spaceEvenly` (edge gaps not halved).
fn distribute(
    items: Vec<Value>,
    justify: Option<Justify>,
    filler: impl Fn() -> Value,
) -> Vec<Value> {
    let Some(j) = justify.filter(|j| *j != Justify::Start) else {
        return items;
    };
    let n = items.len();
    let mut out = Vec::with_capacity(n * 2 + 1);
    let edges = matches!(
        j,
        Justify::Center | Justify::End | Justify::SpaceAround | Justify::SpaceEvenly
    );
    if edges {
        out.push(filler());
    }
    for (i, it) in items.into_iter().enumerate() {
        if i > 0
            && matches!(
                j,
                Justify::SpaceBetween | Justify::SpaceAround | Justify::SpaceEvenly
            )
        {
            out.push(filler());
        }
        out.push(it);
    }
    if matches!(
        j,
        Justify::Center | Justify::SpaceAround | Justify::SpaceEvenly
    ) {
        out.push(filler());
    }
    out
}

pub(super) fn vstack(v: &VStackElement, skipped: &mut Vec<SkippedElement>) -> Value {
    let VStackElement {
        children,
        style,
        alignment,
        spacing,
        justify,
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
    let items = distribute(
        items,
        *justify,
        || json!({"type": "Container", "spacing": "None", "height": "stretch", "items": []}),
    );
    let mut obj = json!({
        "type": "Container",
        "spacing": "None",
        "items": items,
    });
    match justify {
        Some(Justify::Center) => obj["verticalContentAlignment"] = json!("Center"),
        Some(Justify::End) => obj["verticalContentAlignment"] = json!("Bottom"),
        _ => {}
    }
    if justify.is_some_and(|j| j != Justify::Start) {
        obj["height"] = json!("stretch");
    }
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
        children,
        spacing,
        justify,
        style,
        ..
    } = h;
    let n = children.len();
    // layoutPriority: the highest-priority children keep their natural size ("auto"),
    // the rest share what is left ("stretch").
    let top_priority = children
        .iter()
        .filter_map(priority_of)
        .fold(None, |m: Option<f64>, p| Some(m.map_or(p, |m| m.max(p))));
    let columns: Vec<Value> = children
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let (width, items) = match c {
                WidgetElement::Spacer(_) => (json!("stretch"), json!([])),
                // Column immediately before a spacer expands (title + Sync pattern).
                _ if i + 1 < n && matches!(children[i + 1], WidgetElement::Spacer(_)) => {
                    (json!("stretch"), json!([el(c, skipped)]))
                }
                _ => {
                    let mut w = flex_width(c);
                    if w == json!("auto") && super::style::fills_width(c) {
                        w = json!("stretch");
                    }
                    if let Some(top) = top_priority {
                        if w == json!("auto") && priority_of(c).unwrap_or(0.0) < top {
                            w = json!("stretch");
                        }
                    }
                    (w, json!([el(c, skipped)]))
                }
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
        .collect();
    let columns = distribute(
        columns,
        *justify,
        || json!({"type": "Column", "width": "stretch", "spacing": "None", "items": []}),
    );
    let set = json!({
        "type": "ColumnSet",
        "spacing": "None",
        "columns": columns,
    });
    // ColumnSet has no backgroundImage: a background (solid or gradient) needs a Container.
    if style.background.is_none() {
        return set;
    }
    let mut obj = json!({"type": "Container", "spacing": "None", "items": [set]});
    apply_container_style(&mut obj, style);
    obj
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
        // Drawn at its intrinsic size (badge, avatar): show it at that size, not stretched to
        // the column (a 20 pt badge filled the whole card).
        let mut img = json!({
            "type": "Image",
            "url": uri,
            "size": "Stretch",
            "horizontalAlignment": "Center",
        });
        if let Some((w, h)) = crate::rasterize::zstack_size(children, style) {
            img.as_object_mut().unwrap().remove("size");
            img["width"] = json!(super::style::px(w));
            img["height"] = json!(super::style::px(h));
        }
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
