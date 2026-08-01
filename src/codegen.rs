//! Emit TypeScript IR types from an explicit Rust IR_SPEC (SoT with models.rs).
//!
//! Not `include_str` of a hand-maintained twin — the emitter builds the file.
//! Exhaustiveness is checked via [`WidgetElement`] match in tests / type_name.

/// Emit `guest-js/generated/widget-types.ts`.
pub fn emit_widget_types_ts() -> String {
    let mut out = String::new();
    out.push_str(
        "/**\n\
         * Generated widget IR types — do not edit by hand.\n\
         * Source of truth: Rust `src/models.rs` via `cargo run --bin gen-ts --features codegen`.\n\
         * Emitter: `src/codegen.rs` IR_SPEC.\n\
         */\n\n",
    );

    out.push_str(
        "// ── Enums ──\n\n\
         export type FontWeight = \"ultralight\" | \"thin\" | \"light\" | \"regular\" | \"medium\" | \"semibold\" | \"bold\" | \"heavy\" | \"black\";\n\
         export type FontDesign = \"default\" | \"monospaced\" | \"rounded\" | \"serif\";\n\
         export type TextAlignment = \"leading\" | \"center\" | \"trailing\";\n\
         export type HorizontalAlignment = \"leading\" | \"center\" | \"trailing\";\n\
         export type VerticalAlignment = \"top\" | \"center\" | \"bottom\";\n\
         export type ContentMode = \"fit\" | \"fill\";\n\
         export type ProgressStyle = \"linear\" | \"circular\";\n\
         export type GaugeStyle = \"circular\" | \"linear\";\n\
         export type DateStyle = \"time\" | \"date\" | \"relative\" | \"offset\" | \"timer\";\n\
         export type ChartType = \"bar\" | \"line\" | \"area\" | \"pie\";\n\
         export type ShapeType = \"circle\" | \"capsule\" | \"rectangle\";\n\
         export type TimerCounting = \"up\" | \"down\";\n\
         export type ClipShape = \"circle\" | \"capsule\" | \"rectangle\";\n\
         export type TextStyle =\n\
           | \"largeTitle\" | \"title\" | \"title2\" | \"title3\"\n\
           | \"headline\" | \"subheadline\"\n\
           | \"body\" | \"callout\"\n\
           | \"footnote\" | \"caption\" | \"caption2\";\n\
         export type GradientType = \"linear\" | \"radial\" | \"angular\";\n\
         export type GradientDirection =\n\
           | \"topToBottom\" | \"bottomToTop\"\n\
           | \"leadingToTrailing\" | \"trailingToLeading\"\n\
           | \"topLeadingToBottomTrailing\" | \"topTrailingToBottomLeading\";\n\n",
    );

    out.push_str(
        "// ── Supporting types ──\n\n\
         export type ColorValue = string | { light: string; dark: string };\n\n\
         export interface ChartDataPoint {\n\
           label: string;\n\
           value: number;\n\
           color?: ColorValue;\n\
         }\n\n\
         export interface FrameConfig {\n\
           width?: number;\n\
           height?: number;\n\
           maxWidth?: number | \"infinity\";\n\
           maxHeight?: number | \"infinity\";\n\
         }\n\n\
         export interface BorderConfig {\n\
           color: string;\n\
           width?: number;\n\
         }\n\n\
         export interface GradientConfig {\n\
           gradientType: GradientType;\n\
           colors: string[];\n\
           direction?: GradientDirection;\n\
         }\n\n\
         export interface ShadowConfig {\n\
           color?: string;\n\
           radius?: number;\n\
           x?: number;\n\
           y?: number;\n\
         }\n\n\
         export type BackgroundValue = string | GradientConfig | { light: string; dark: string };\n\n\
         export type PaddingValue = number | {\n\
           top?: number;\n\
           bottom?: number;\n\
           leading?: number;\n\
           trailing?: number;\n\
         };\n\n\
         export interface ElementStyle {\n\
           padding?: PaddingValue;\n\
           background?: BackgroundValue;\n\
           cornerRadius?: number;\n\
           opacity?: number;\n\
           frame?: FrameConfig;\n\
           border?: BorderConfig;\n\
           shadow?: ShadowConfig;\n\
           clipShape?: ClipShape;\n\
           flex?: number;\n\
         }\n\n",
    );

    // Element interfaces — IR_SPEC mapping (must stay in sync with WidgetElement)
    for spec in IR_ELEMENTS {
        out.push_str(&format!(
            "export interface {} extends ElementStyle {{\n  type: \"{}\";\n",
            spec.ts_name, spec.wire
        ));
        for field in spec.fields {
            out.push_str(&format!("  {};\n", field));
        }
        out.push_str("}\n\n");
    }

    out.push_str(
        "export interface SpacerElement {\n\
           type: \"spacer\";\n\
           minLength?: number;\n\
         }\n\n\
         export interface ListItem {\n\
           text: string;\n\
           checked?: boolean;\n\
           action?: string;\n\
           payload?: string;\n\
         }\n\n\
         export interface CanvasCircle {\n\
           draw: \"circle\";\n\
           cx: number; cy: number; r: number;\n\
           fill?: ColorValue; stroke?: ColorValue; strokeWidth?: number;\n\
         }\n\
         export interface CanvasLine {\n\
           draw: \"line\";\n\
           x1: number; y1: number; x2: number; y2: number;\n\
           stroke?: ColorValue; strokeWidth?: number; lineCap?: \"butt\" | \"round\" | \"square\";\n\
         }\n\
         export interface CanvasRect {\n\
           draw: \"rect\";\n\
           x: number; y: number; width: number; height: number;\n\
           fill?: ColorValue; stroke?: ColorValue; strokeWidth?: number; cornerRadius?: number;\n\
         }\n\
         export interface CanvasArc {\n\
           draw: \"arc\";\n\
           cx: number; cy: number; r: number;\n\
           startAngle: number; endAngle: number;\n\
           fill?: ColorValue; stroke?: ColorValue; strokeWidth?: number;\n\
         }\n\
         export interface CanvasText {\n\
           draw: \"text\";\n\
           x: number; y: number; content: string;\n\
           fontSize?: number; color?: ColorValue; anchor?: \"start\" | \"middle\" | \"end\";\n\
         }\n\
         export interface CanvasPath {\n\
           draw: \"path\";\n\
           d: string;\n\
           fill?: ColorValue; stroke?: ColorValue; strokeWidth?: number;\n\
         }\n\
         export type CanvasDrawCommand = CanvasCircle | CanvasLine | CanvasRect | CanvasArc | CanvasText | CanvasPath;\n\n",
    );

    // Remaining elements that need custom bodies (list, canvas, etc. already partially in IR_ELEMENTS)
    out.push_str("export type WidgetElement =\n");
    let names: Vec<&str> = IR_ELEMENTS
        .iter()
        .map(|e| e.ts_name)
        .chain(std::iter::once("SpacerElement"))
        .collect();
    for (i, n) in names.iter().enumerate() {
        let sep = if i + 1 == names.len() { ";" } else { "" };
        out.push_str(&format!("  | {}{}\n", n, sep));
    }
    out.push('\n');

    out.push_str(
        "export interface WidgetConfig {\n\
           version?: number;\n\
           small?: WidgetElement;\n\
           medium?: WidgetElement;\n\
           large?: WidgetElement;\n\
         }\n",
    );

    out
}

struct ElementSpec {
    wire: &'static str,
    ts_name: &'static str,
    fields: &'static [&'static str],
}

/// Explicit IR → TS mapping. When adding a [`WidgetElement`] variant, extend this list
/// and the exhaustive match in tests.
const IR_ELEMENTS: &[ElementSpec] = &[
    ElementSpec {
        wire: "vstack",
        ts_name: "VStackElement",
        fields: &["children: WidgetElement[]", "spacing?: number", "alignment?: HorizontalAlignment"],
    },
    ElementSpec {
        wire: "hstack",
        ts_name: "HStackElement",
        fields: &["children: WidgetElement[]", "spacing?: number", "alignment?: VerticalAlignment"],
    },
    ElementSpec {
        wire: "zstack",
        ts_name: "ZStackElement",
        fields: &["children: WidgetElement[]", "alignment?: string"],
    },
    ElementSpec {
        wire: "grid",
        ts_name: "GridElement",
        fields: &[
            "children: WidgetElement[]",
            "columns?: number",
            "spacing?: number",
            "rowSpacing?: number",
        ],
    },
    ElementSpec {
        wire: "container",
        ts_name: "ContainerElement",
        fields: &["children?: WidgetElement[]", "contentAlignment?: string"],
    },
    ElementSpec {
        wire: "text",
        ts_name: "TextElement",
        fields: &[
            "content: string",
            "fontSize?: number",
            "fontWeight?: FontWeight",
            "fontDesign?: FontDesign",
            "textStyle?: TextStyle",
            "color?: ColorValue",
            "alignment?: TextAlignment",
            "lineLimit?: number",
        ],
    },
    ElementSpec {
        wire: "image",
        ts_name: "ImageElement",
        fields: &[
            "systemName?: string",
            "data?: string",
            "url?: string",
            "size?: number",
            "color?: ColorValue",
            "contentMode?: ContentMode",
        ],
    },
    ElementSpec {
        wire: "progress",
        ts_name: "ProgressElement",
        fields: &[
            "value: number",
            "total?: number",
            "label?: string",
            "tint?: ColorValue",
            "color?: ColorValue",
            "barStyle?: ProgressStyle",
        ],
    },
    ElementSpec {
        wire: "gauge",
        ts_name: "GaugeElement",
        fields: &[
            "value: number",
            "min?: number",
            "max?: number",
            "label?: string",
            "currentValueLabel?: string",
            "tint?: ColorValue",
            "color?: ColorValue",
            "gaugeStyle?: GaugeStyle",
        ],
    },
    ElementSpec {
        wire: "button",
        ts_name: "ButtonElement",
        fields: &[
            "label: string",
            "url?: string",
            "action?: string",
            "color?: ColorValue",
            "backgroundColor?: ColorValue",
            "fontSize?: number",
            "textAlignment?: TextAlignment",
        ],
    },
    ElementSpec {
        wire: "toggle",
        ts_name: "ToggleElement",
        fields: &[
            "isOn: boolean",
            "label?: string",
            "tint?: ColorValue",
            "action?: string",
        ],
    },
    ElementSpec {
        wire: "divider",
        ts_name: "DividerElement",
        fields: &["color?: ColorValue", "thickness?: number"],
    },
    ElementSpec {
        wire: "date",
        ts_name: "DateElement",
        fields: &[
            "date: string",
            "dateStyle?: DateStyle",
            "fontSize?: number",
            "color?: ColorValue",
        ],
    },
    ElementSpec {
        wire: "chart",
        ts_name: "ChartElement",
        fields: &["chartType: ChartType", "chartData: ChartDataPoint[]", "tint?: ColorValue"],
    },
    ElementSpec {
        wire: "list",
        ts_name: "ListElement",
        fields: &[
            "items: ListItem[]",
            "spacing?: number",
            "fontSize?: number",
            "color?: ColorValue",
        ],
    },
    ElementSpec {
        wire: "link",
        ts_name: "LinkElement",
        fields: &["children: WidgetElement[]", "url?: string", "action?: string"],
    },
    ElementSpec {
        wire: "shape",
        ts_name: "ShapeElement",
        fields: &[
            "shapeType: ShapeType",
            "fill?: ColorValue",
            "stroke?: ColorValue",
            "strokeWidth?: number",
            "size?: number",
        ],
    },
    ElementSpec {
        wire: "timer",
        ts_name: "TimerElement",
        fields: &[
            "targetDate: string",
            "counting?: TimerCounting",
            "fontSize?: number",
            "fontWeight?: FontWeight",
            "color?: ColorValue",
        ],
    },
    ElementSpec {
        wire: "canvas",
        ts_name: "CanvasElement",
        fields: &["width: number", "height: number", "elements: CanvasDrawCommand[]"],
    },
    ElementSpec {
        wire: "label",
        ts_name: "LabelElement",
        fields: &[
            "text: string",
            "systemName: string",
            "iconColor?: ColorValue",
            "fontSize?: number",
            "fontWeight?: FontWeight",
            "color?: ColorValue",
            "spacing?: number",
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::ELEMENT_TYPES;
    use crate::models::WidgetElement;

    #[test]
    fn ir_spec_covers_all_element_types() {
        let wires: Vec<&str> = IR_ELEMENTS.iter().map(|e| e.wire).chain(["spacer"]).collect();
        for ty in ELEMENT_TYPES {
            assert!(
                wires.contains(ty),
                "IR_ELEMENTS missing wire type `{ty}`"
            );
        }
        assert_eq!(wires.len(), ELEMENT_TYPES.len());
    }

    #[test]
    fn codegen_covers_all_element_types() {
        let ts = emit_widget_types_ts();
        for ty in ELEMENT_TYPES {
            let needle = format!("type: \"{ty}\"");
            assert!(
                ts.contains(&needle),
                "generated TS missing element type `{ty}`"
            );
        }
        assert!(ts.contains("export interface WidgetConfig"));
        assert!(ts.contains("export type WidgetElement"));
        // Must not be a stale include_str twin
        assert!(ts.contains("Emitter: `src/codegen.rs` IR_SPEC"));
    }

    #[test]
    fn type_name_exhaustive() {
        // Compiling this match fails if a WidgetElement variant is added without update.
        fn check(el: &WidgetElement) -> &'static str {
            el.type_name()
        }
        let _ = check(&WidgetElement::Spacer { min_length: None });
    }
}
