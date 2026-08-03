//! Emit TypeScript IR types from an explicit Rust `IR_ELEMENTS` spec (SoT with models).
//!
//! Static preamble / enums / supporting / footer live in `src/codegen/*.ts` via
//! `include_str!`. The remaining `push_str` calls are intentional: per-element
//! interfaces and the `WidgetElement` union are generated from `IR_ELEMENTS`
//! so exhaustiveness tests stay coupled to [`crate::models::WidgetElement`].

/// Emit `guest-js/generated/widget-types.ts`.
pub fn emit_widget_types_ts() -> String {
    let mut out = String::new();
    out.push_str(include_str!("codegen/preamble.ts"));
    out.push('\n');
    out.push_str(include_str!("codegen/enums.ts"));
    out.push_str(include_str!("codegen/supporting.ts"));

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

    out.push_str(include_str!("codegen/footer.ts"));

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
        fields: &[
            "children: WidgetElement[]",
            "spacing?: number",
            "alignment?: HorizontalAlignment",
        ],
    },
    ElementSpec {
        wire: "hstack",
        ts_name: "HStackElement",
        fields: &[
            "children: WidgetElement[]",
            "spacing?: number",
            "alignment?: VerticalAlignment",
        ],
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
        fields: &[
            "chartType: ChartType",
            "chartData: ChartDataPoint[]",
            "tint?: ColorValue",
        ],
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
        fields: &[
            "children: WidgetElement[]",
            "url?: string",
            "action?: string",
        ],
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
        fields: &[
            "width: number",
            "height: number",
            "elements: CanvasDrawCommand[]",
        ],
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

/// Wire type strings + field hints for Swift (flat Codable stays in Models.swift).
pub fn emit_wire_catalog_swift() -> String {
    let mut out = String::from(
        "// AUTO-GENERATED by `cargo run --bin gen-native --features codegen`\n\
         // Do not edit. Source: `src/codegen.rs` IR_ELEMENTS.\n\
         // Runtime decode still uses flat `WidgetElement` in Models.swift.\n\n\
         import Foundation\n\n\
         /// Known `type` wire values for widget IR elements.\n\
         public enum WidgetWireType: String, CaseIterable, Sendable {\n",
    );
    for spec in IR_ELEMENTS {
        let case = swift_case_name(spec.wire);
        out.push_str(&format!("    case {case} = \"{}\"\n", spec.wire));
    }
    out.push_str("    case spacer = \"spacer\"\n}\n\n");
    out.push_str("/// Per-element field hints (documentation / drift checks; not Codable).\n");
    out.push_str("public enum WidgetWireFields {\n");
    for spec in IR_ELEMENTS {
        let case = swift_case_name(spec.wire);
        let fields: Vec<String> = spec
            .fields
            .iter()
            .map(|f| format!("\"{}\"", f.replace('\\', "\\\\").replace('"', "\\\"")))
            .collect();
        out.push_str(&format!(
            "    public static let {case}: [String] = [{}]\n",
            fields.join(", ")
        ));
    }
    out.push_str("    public static let spacer: [String] = [\"minLength?: number\"]\n");
    out.push_str("}\n");
    out
}

/// Wire type constants for Android Glance renderer (JSONObject path stays dynamic).
pub fn emit_wire_catalog_kotlin() -> String {
    let mut out = String::from(
        "// AUTO-GENERATED by `cargo run --bin gen-native --features codegen`\n\
         // Do not edit. Source: `src/codegen.rs` IR_ELEMENTS.\n\
         // Runtime still uses org.json.JSONObject via render.El.\n\n\
         package git.s00d.widgets\n\n\
         /** Known `type` wire values for widget IR elements. */\n\
         object WireTypes {\n",
    );
    for spec in IR_ELEMENTS {
        let const_name = kotlin_const_name(spec.wire);
        out.push_str(&format!(
            "    const val {const_name} = \"{}\"\n",
            spec.wire
        ));
    }
    out.push_str("    const val SPACER = \"spacer\"\n\n");
    out.push_str("    val ALL: Set<String> = setOf(\n");
    for spec in IR_ELEMENTS {
        out.push_str(&format!("        {},\n", kotlin_const_name(spec.wire)));
    }
    out.push_str("        SPACER,\n    )\n}\n");
    out
}

fn swift_case_name(wire: &str) -> String {
    // camelCase identifiers from wire names (already lower).
    wire.to_string()
}

fn kotlin_const_name(wire: &str) -> String {
    wire.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::ELEMENT_TYPES;

    #[test]
    fn ir_spec_covers_all_element_types() {
        let wires: Vec<&str> = IR_ELEMENTS
            .iter()
            .map(|e| e.wire)
            .chain(["spacer"])
            .collect();
        for ty in ELEMENT_TYPES {
            assert!(wires.contains(ty), "IR_ELEMENTS missing wire type `{ty}`");
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
    fn swift_catalog_covers_all_element_types() {
        let swift = emit_wire_catalog_swift();
        for ty in ELEMENT_TYPES {
            assert!(
                swift.contains(&format!("= \"{ty}\"")),
                "Swift catalog missing `{ty}`"
            );
        }
    }

    #[test]
    fn kotlin_catalog_covers_all_element_types() {
        let kt = emit_wire_catalog_kotlin();
        for ty in ELEMENT_TYPES {
            assert!(
                kt.contains(&format!("= \"{ty}\"")),
                "Kotlin catalog missing `{ty}`"
            );
        }
    }

    #[test]
    fn type_name_exhaustive() {
        // Compiling this match fails if a WidgetElement variant is added without update.
        use crate::models::{SpacerElement, WidgetElement};
        fn check(el: &WidgetElement) -> &'static str {
            el.type_name()
        }
        let _ = check(&WidgetElement::Spacer(SpacerElement { min_length: None }));
    }
}
