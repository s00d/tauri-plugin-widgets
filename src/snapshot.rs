//! Canonical layout dump for core WidgetConfig snapshot tests.

use crate::models::{WidgetConfig, WidgetElement, VStackElement, HStackElement, ZStackElement, GridElement, ContainerElement, TextElement, ImageElement, ProgressElement, ButtonElement, DividerElement, SpacerElement, LinkElement, ShapeElement};
use serde_json::{json, Value};

/// Core elements with a strict cross-platform snapshot contract.
pub const CORE_ELEMENTS: &[&str] = &[
    "vstack",
    "hstack",
    "zstack",
    "container",
    "grid",
    "text",
    "image",
    "spacer",
    "divider",
    "progress",
    "button",
    "link",
    "shape",
];

/// Extended / best-effort elements (platform-dependent).
pub const EXTENDED_ELEMENTS: &[&str] = &[
    "gauge", "toggle", "date", "chart", "list", "timer", "canvas", "label",
];

pub fn is_core_element(ty: &str) -> bool {
    CORE_ELEMENTS.contains(&ty)
}

/// Dump a config to a stable JSON tree (nulls omitted, keys sorted via serde_json Map order
/// of insertion — we insert in a fixed order).
pub fn dump_config(config: &WidgetConfig) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("version".into(), json!(config.version));
    if let Some(el) = &config.small {
        obj.insert("small".into(), dump_element(el, ParentAxis::Vertical));
    }
    if let Some(el) = &config.medium {
        obj.insert("medium".into(), dump_element(el, ParentAxis::Vertical));
    }
    if let Some(el) = &config.large {
        obj.insert("large".into(), dump_element(el, ParentAxis::Vertical));
    }
    Value::Object(obj)
}

#[derive(Clone, Copy)]
enum ParentAxis {
    Vertical,
    Horizontal,
    Overlay,
}

fn dump_element(el: &WidgetElement, parent: ParentAxis) -> Value {
    let ty = el.type_name();
    let mut obj = serde_json::Map::new();
    obj.insert("type".into(), json!(ty));
    obj.insert(
        "tier".into(),
        json!(if is_core_element(ty) {
            "core"
        } else {
            "extended"
        }),
    );

    match el {
        WidgetElement::VStack(VStackElement {
            children,
            spacing,
            alignment,
            style,
        }) => {
            put_opt_f64(&mut obj, "spacing", *spacing);
            if let Some(a) = alignment {
                obj.insert("alignment".into(), json!(format!("{a:?}").to_lowercase()));
            }
            dump_style(&mut obj, style);
            obj.insert(
                "children".into(),
                Value::Array(
                    children
                        .iter()
                        .map(|c| dump_element(c, ParentAxis::Vertical))
                        .collect(),
                ),
            );
        }
        WidgetElement::HStack(HStackElement {
            children,
            spacing,
            alignment,
            style,
        }) => {
            put_opt_f64(&mut obj, "spacing", *spacing);
            if let Some(a) = alignment {
                obj.insert("alignment".into(), json!(format!("{a:?}").to_lowercase()));
            }
            dump_style(&mut obj, style);
            obj.insert(
                "children".into(),
                Value::Array(
                    children
                        .iter()
                        .map(|c| dump_element(c, ParentAxis::Horizontal))
                        .collect(),
                ),
            );
        }
        WidgetElement::ZStack(ZStackElement {
            children,
            alignment,
            style,
        }) => {
            if let Some(a) = alignment {
                obj.insert("alignment".into(), json!(a));
            } else {
                obj.insert("alignment".into(), json!("center"));
            }
            dump_style(&mut obj, style);
            obj.insert(
                "children".into(),
                Value::Array(
                    children
                        .iter()
                        .map(|c| dump_element(c, ParentAxis::Overlay))
                        .collect(),
                ),
            );
        }
        WidgetElement::Grid(GridElement {
            children,
            columns,
            spacing,
            row_spacing,
            style,
        }) => {
            obj.insert("columns".into(), json!(columns));
            put_opt_f64(&mut obj, "spacing", *spacing);
            put_opt_f64(&mut obj, "rowSpacing", *row_spacing);
            dump_style(&mut obj, style);
            obj.insert(
                "children".into(),
                Value::Array(
                    children
                        .iter()
                        .map(|c| dump_element(c, ParentAxis::Vertical))
                        .collect(),
                ),
            );
        }
        WidgetElement::Container(ContainerElement {
            children,
            content_alignment,
            style,
        }) => {
            if let Some(a) = content_alignment {
                obj.insert("contentAlignment".into(), json!(a));
            }
            dump_style(&mut obj, style);
            obj.insert(
                "children".into(),
                Value::Array(
                    children
                        .iter()
                        .map(|c| dump_element(c, ParentAxis::Overlay))
                        .collect(),
                ),
            );
        }
        WidgetElement::Text(TextElement {
            content,
            font_size,
            font_weight,
            text_style,
            color,
            alignment,
            line_limit,
            style,
            ..
        }) => {
            obj.insert("content".into(), json!(content));
            put_opt_f64(&mut obj, "fontSize", *font_size);
            if let Some(w) = font_weight {
                obj.insert("fontWeight".into(), json!(format!("{w:?}").to_lowercase()));
            }
            if let Some(ts) = text_style {
                obj.insert("textStyle".into(), json!(format!("{ts:?}")));
            }
            if let Some(c) = color {
                obj.insert("color".into(), color_json(c));
            }
            if let Some(a) = alignment {
                obj.insert("alignment".into(), json!(format!("{a:?}").to_lowercase()));
            }
            if let Some(n) = line_limit {
                obj.insert("lineLimit".into(), json!(n));
            }
            dump_style(&mut obj, style);
        }
        WidgetElement::Image(ImageElement {
            system_name,
            url,
            size,
            color,
            content_mode,
            style,
            ..
        }) => {
            if let Some(s) = system_name {
                obj.insert("systemName".into(), json!(s));
            }
            if let Some(u) = url {
                obj.insert("url".into(), json!(u));
            }
            put_opt_f64(&mut obj, "size", *size);
            if let Some(c) = color {
                obj.insert("color".into(), color_json(c));
            }
            if let Some(m) = content_mode {
                obj.insert("contentMode".into(), json!(format!("{m:?}").to_lowercase()));
            }
            dump_style(&mut obj, style);
        }
        WidgetElement::Spacer(SpacerElement { min_length }) => {
            put_opt_f64(&mut obj, "minLength", *min_length);
            obj.insert("flex".into(), json!(true));
        }
        WidgetElement::Divider(DividerElement {
            color,
            thickness,
            style,
        }) => {
            let axis = match parent {
                ParentAxis::Horizontal => "vertical",
                _ => "horizontal",
            };
            obj.insert("axis".into(), json!(axis));
            put_opt_f64(&mut obj, "thickness", *thickness);
            if let Some(c) = color {
                obj.insert("color".into(), color_json(c));
            }
            dump_style(&mut obj, style);
        }
        WidgetElement::Progress(ProgressElement {
            value,
            total,
            label,
            tint,
            bar_style,
            style,
            ..
        }) => {
            obj.insert("value".into(), json!(value));
            obj.insert("total".into(), json!(total));
            if let Some(l) = label {
                obj.insert("label".into(), json!(l));
            }
            if let Some(t) = tint {
                obj.insert("tint".into(), color_json(t));
            }
            if let Some(s) = bar_style {
                obj.insert("barStyle".into(), json!(format!("{s:?}").to_lowercase()));
            }
            dump_style(&mut obj, style);
        }
        WidgetElement::Button(ButtonElement {
            label,
            url,
            action,
            color,
            background_color,
            font_size,
            style,
            ..
        }) => {
            obj.insert("label".into(), json!(label));
            if let Some(u) = url {
                obj.insert("url".into(), json!(u));
            }
            if let Some(a) = action {
                obj.insert("action".into(), json!(a));
            }
            if let Some(c) = color {
                obj.insert("color".into(), color_json(c));
            }
            if let Some(c) = background_color {
                obj.insert("backgroundColor".into(), color_json(c));
            }
            put_opt_f64(&mut obj, "fontSize", *font_size);
            dump_style(&mut obj, style);
        }
        WidgetElement::Link(LinkElement {
            children,
            url,
            action,
            style,
        }) => {
            if let Some(u) = url {
                obj.insert("url".into(), json!(u));
            }
            if let Some(a) = action {
                obj.insert("action".into(), json!(a));
            }
            dump_style(&mut obj, style);
            obj.insert(
                "children".into(),
                Value::Array(children.iter().map(|c| dump_element(c, parent)).collect()),
            );
        }
        WidgetElement::Shape(ShapeElement {
            shape_type,
            fill,
            stroke,
            stroke_width,
            size,
            style,
        }) => {
            obj.insert(
                "shapeType".into(),
                json!(format!("{shape_type:?}").to_lowercase()),
            );
            // Capsule contract: width = 2*size, height = size when size set.
            if let Some(s) = size {
                obj.insert("size".into(), json!(s));
                if matches!(shape_type, crate::models::ShapeType::Capsule) {
                    obj.insert("width".into(), json!(s * 2.0));
                    obj.insert("height".into(), json!(s));
                } else {
                    obj.insert("width".into(), json!(s));
                    obj.insert("height".into(), json!(s));
                }
            }
            if let Some(f) = fill {
                obj.insert("fill".into(), color_json(f));
            }
            if let Some(s) = stroke {
                obj.insert("stroke".into(), color_json(s));
            }
            put_opt_f64(&mut obj, "strokeWidth", *stroke_width);
            dump_style(&mut obj, style);
        }
        // Extended — minimal dump
        other => {
            obj.insert("note".into(), json!("extended best-effort"));
            let _ = other;
        }
    }

    Value::Object(obj)
}

fn put_opt_f64(obj: &mut serde_json::Map<String, Value>, key: &str, v: Option<f64>) {
    if let Some(n) = v {
        obj.insert(key.into(), json!(n));
    }
}

fn color_json(c: &crate::models::ColorValue) -> Value {
    match c {
        crate::models::ColorValue::Solid(s) => json!(s),
        crate::models::ColorValue::Adaptive { light, dark } => {
            json!({ "light": light, "dark": dark })
        }
    }
}

fn dump_style(obj: &mut serde_json::Map<String, Value>, style: &crate::models::ElementStyle) {
    if let Some(p) = &style.padding {
        obj.insert(
            "padding".into(),
            serde_json::to_value(p).unwrap_or(Value::Null),
        );
    }
    if let Some(b) = &style.background {
        obj.insert(
            "background".into(),
            serde_json::to_value(b).unwrap_or(Value::Null),
        );
    }
    put_opt_f64(obj, "cornerRadius", style.corner_radius);
    put_opt_f64(obj, "opacity", style.opacity);
    put_opt_f64(obj, "flex", style.flex);
    if let Some(f) = &style.frame {
        obj.insert(
            "frame".into(),
            serde_json::to_value(f).unwrap_or(Value::Null),
        );
    }
    if let Some(b) = &style.border {
        obj.insert(
            "border".into(),
            serde_json::to_value(b).unwrap_or(Value::Null),
        );
    }
    if let Some(s) = &style.shadow {
        obj.insert(
            "shadow".into(),
            serde_json::to_value(s).unwrap_or(Value::Null),
        );
    }
    if let Some(c) = &style.clip_shape {
        obj.insert(
            "clipShape".into(),
            serde_json::to_value(c).unwrap_or(Value::Null),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn snap_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/core")
    }

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/core")
    }

    #[test]
    fn core_and_extended_partition_element_types() {
        use crate::capabilities::ELEMENT_TYPES;
        let mut all: Vec<&str> = CORE_ELEMENTS
            .iter()
            .chain(EXTENDED_ELEMENTS.iter())
            .copied()
            .collect();
        all.sort();
        let mut expected: Vec<&str> = ELEMENT_TYPES.to_vec();
        expected.sort();
        assert_eq!(all, expected);
    }

    #[test]
    fn core_layout_snapshot() {
        let path = fixture_dir().join("layout.json");
        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            let sample = r##"{
  "version": 1,
  "small": {
    "type": "vstack",
    "spacing": 8,
    "padding": 12,
    "background": "#1a1a2e",
    "children": [
      { "type": "text", "content": "Hello", "fontSize": 18, "fontWeight": "bold", "color": "#fff", "alignment": "center" },
      { "type": "hstack", "spacing": 4, "children": [
        { "type": "shape", "shapeType": "capsule", "size": 8, "fill": "#4CAF50" },
        { "type": "divider" },
        { "type": "spacer" },
        { "type": "progress", "value": 0.5, "tint": "#4CAF50", "label": "OK" }
      ]},
      { "type": "button", "label": "Go", "action": "go", "backgroundColor": "#2196F3", "color": "#fff" }
    ]
  }
}"##;
            fs::write(&path, sample).unwrap();
        }
        let cfg: WidgetConfig = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let dump = dump_config(&cfg);
        let pretty = serde_json::to_string_pretty(&dump).unwrap() + "\n";

        let snap = snap_dir().join("layout.snap.json");
        fs::create_dir_all(snap.parent().unwrap()).unwrap();
        if !snap.exists() {
            fs::write(&snap, &pretty).unwrap();
        }
        let expected = fs::read_to_string(&snap).unwrap();
        assert_eq!(pretty, expected, "core layout snapshot drifted");

        // Divider inside hstack must be vertical axis in dump.
        let hstack = &dump["small"]["children"][1];
        assert_eq!(hstack["type"], "hstack");
        assert_eq!(hstack["children"][1]["type"], "divider");
        assert_eq!(hstack["children"][1]["axis"], "vertical");
        // Capsule size contract
        assert_eq!(hstack["children"][0]["width"], 16.0);
        assert_eq!(hstack["children"][0]["height"], 8.0);
    }
}
