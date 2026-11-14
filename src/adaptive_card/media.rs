//! Media group: image / shape / canvas (plus the shared rasterize fallback).

use crate::models::{ClipShape, ContentMode, ImageElement, ShapeElement, WidgetElement};
use crate::receipt::SkippedElement;
use serde_json::{json, Value};

use super::style::{ac_color, approx_hex_semantic, img_size_bucket, px, size_bucket};

pub(super) fn image(i: &ImageElement) -> Value {
    let ImageElement {
        url,
        data,
        system_name,
        size,
        color,
        fallback_system_name,
        palette_colors,
        ..
    } = i;
    // AC draws one tint: palette rendering keeps its primary color.
    let color = color.as_ref().or(palette_colors.first());
    let url_str = url
        .clone()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| {
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
        // AC cannot detect a failed load, so `fallbackSystemName` only covers a missing source.
        // Prefer rasterized glyph Image when `rasterize` is on; else emoji TextBlock.
        if let Some(name) = system_name.as_deref().or(fallback_system_name.as_deref()) {
            #[cfg(feature = "rasterize")]
            {
                let sz = size.unwrap_or(24.0);
                let color_hex = super::style::color_hex(color).unwrap_or_else(|| "#FFFFFF".into());
                if let Ok(uri) = crate::icons::icon_png_data_uri(name, sz, &color_hex) {
                    return finish_image(
                        json!({
                            "type": "Image",
                            "url": uri,
                            "size": img_size_bucket(*size),
                        }),
                        i,
                    );
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
            if let Some(c) = ac_color(color).or_else(|| approx_hex_semantic(color)) {
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
        if let Some(c) = ac_color(color).or_else(|| approx_hex_semantic(color)) {
            obj["color"] = json!(c);
        }
        return obj;
    }
    finish_image(
        json!({
            "type": "Image",
            "url": url_str,
            "size": img_size_bucket(*size),
        }),
        i,
    )
}

/// `contentMode`, `clipShape` and `aspectRatio` on an AC `Image`.
fn finish_image(mut img: Value, i: &ImageElement) -> Value {
    let ImageElement {
        size,
        content_mode,
        style,
        ..
    } = i;
    match content_mode {
        // AC `Stretch` fills the width but keeps the aspect ratio.
        Some(ContentMode::Stretch) => img["size"] = json!("Stretch"),
        Some(ContentMode::Center) => {
            img["size"] = json!("Auto");
            img["horizontalAlignment"] = json!("Center");
        }
        _ => {}
    }
    match style.clip_shape {
        // Square AC images: an ellipse clip is a circle.
        Some(ClipShape::Circle | ClipShape::Ellipse) => img["style"] = json!("Person"),
        Some(ref shape @ (ClipShape::RoundedRectangle | ClipShape::Capsule)) => {
            // Only inline data can be re-drawn clipped; remote URLs stay square.
            let side = size.unwrap_or(48.0);
            let rx = match shape {
                ClipShape::Capsule => side / 2.0,
                _ => style.corner_radius.unwrap_or(side / 4.0),
            };
            if let Some(href) = img["url"].as_str().filter(|u| u.starts_with("data:")) {
                let svg = crate::rasterize::clipped_image_svg(href, side, side, rx);
                if let Ok(uri) = crate::rasterize::svg_to_data_uri(&svg) {
                    img["url"] = json!(uri);
                }
            }
        }
        _ => {}
    }
    if let (Some(s), Some(r)) = (size, style.aspect_ratio.filter(|r| *r > 0.0)) {
        img["width"] = json!(px(s * r));
        img["height"] = json!(px(*s));
    }
    img
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
                    // `size` is only a host bucket (WPF "Large" = 160 px); width / height win.
                    if let Some(s) = size.filter(|s| *s > 0.0) {
                        img["width"] = json!(super::style::px(s));
                        img["height"] = json!(super::style::px(s));
                    }
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
