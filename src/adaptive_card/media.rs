//! Media group: image / shape / canvas (plus the shared rasterize fallback).

use crate::models::{ImageElement, ShapeElement, WidgetElement};
use crate::receipt::SkippedElement;
use serde_json::{json, Value};

use super::style::{ac_color, approx_hex_semantic, img_size_bucket, size_bucket};

pub(super) fn image(i: &ImageElement) -> Value {
    let ImageElement {
        url,
        data,
        system_name,
        size,
        color,
        ..
    } = i;
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
        // Prefer rasterized glyph Image when `rasterize` is on; else emoji TextBlock.
        if let Some(name) = system_name.as_deref() {
            #[cfg(feature = "rasterize")]
            {
                let sz = size.unwrap_or(24.0);
                let color_hex =
                    super::style::color_hex(color.as_ref()).unwrap_or_else(|| "#FFFFFF".into());
                if let Ok(uri) = crate::icons::icon_png_data_uri(name, sz, &color_hex) {
                    return json!({
                        "type": "Image",
                        "url": uri,
                        "size": img_size_bucket(*size),
                    });
                }
            }
            let glyph = crate::icons::sf_symbol_emoji(name);
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
        let mut obj = json!({
            "type": "TextBlock",
            "text": "•",
            "wrap": true,
            "size": size_bucket(size.map(|s| s * 0.75)),
            "weight": "Bolder",
        });
        if let Some(c) = ac_color(color.as_ref()).or_else(|| approx_hex_semantic(color.as_ref())) {
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

pub(super) fn shape(e: &WidgetElement, skipped: &mut Vec<SkippedElement>) -> Value {
    rasterized_or_skip(e, skipped)
}

pub(super) fn canvas(e: &WidgetElement, skipped: &mut Vec<SkippedElement>) -> Value {
    rasterized_or_skip(e, skipped)
}

/// PNG data URI fallback for nodes Adaptive Cards cannot express natively.
pub(super) fn rasterized_or_skip(e: &WidgetElement, skipped: &mut Vec<SkippedElement>) -> Value {
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
                WidgetElement::Canvas(_) | WidgetElement::Chart(_) | WidgetElement::Gauge(_) => {
                    img["size"] = json!("Stretch");
                }
                WidgetElement::Shape(ShapeElement { size, .. }) => {
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
