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
            "justify?: Justify",
        ],
    },
    ElementSpec {
        wire: "hstack",
        ts_name: "HStackElement",
        fields: &[
            "children: WidgetElement[]",
            "spacing?: number",
            "alignment?: VerticalAlignment",
            "justify?: Justify",
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
            "truncationMode?: TruncationMode",
            "minimumScaleFactor?: number",
            "lineHeight?: number",
            "letterSpacing?: number",
            "underline?: boolean",
            "strikethrough?: boolean",
            "italic?: boolean",
            "monospacedDigit?: boolean",
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
            "fallbackSystemName?: string",
            "symbolRendering?: SymbolRendering",
            "symbolWeight?: FontWeight",
            "paletteColors?: ColorValue[]",
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
            "startAt?: number",
            "endAt?: number",
            "trackColor?: ColorValue",
            "thickness?: number",
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
            "startAt?: number",
            "endAt?: number",
            "minLabel?: string",
            "maxLabel?: string",
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
            "variant?: ButtonVariant",
            "systemName?: string",
            "disabled?: boolean",
            "set?: StatePatch",
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
            "variant?: ToggleVariant",
            "uncheckedTint?: ColorValue",
            "set?: StatePatch",
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
            "showXAxis?: boolean",
            "showYAxis?: boolean",
            "showLegend?: boolean",
            "yMin?: number",
            "yMax?: number",
            "interpolation?: ChartInterpolation",
            "innerRadius?: number",
        ],
    },
    ElementSpec {
        wire: "list",
        ts_name: "ListElement",
        fields: &[
            "items: ListItem[]",
            "children?: WidgetElement[]",
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
            "set?: StatePatch",
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
            "startDate?: string",
            "showHours?: boolean",
            "completedText?: string",
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
    ElementSpec {
        wire: "repeat",
        ts_name: "RepeatElement",
        fields: &[
            "each: string",
            "as?: string",
            "limit?: number",
            "template: WidgetElement",
        ],
    },
    ElementSpec {
        wire: "use",
        ts_name: "UseElement",
        fields: &["template: string", "args?: Record<string, unknown>"],
    },
];

/// Wire type strings + field hints for Swift (flat Codable stays in Models.swift).
pub fn emit_wire_catalog_swift() -> String {
    let mut out = String::from(
        "// AUTO-GENERATED by `cargo run --bin gen-native --features codegen`\n\
         // Do not edit. Source: `src/codegen.rs` IR_ELEMENTS.\n\
         // Decoding uses the generated `WidgetElement` (Generated/WidgetElement.swift).\n\n\
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
        out.push_str(&format!("    const val {const_name} = \"{}\"\n", spec.wire));
    }
    out.push_str("    const val SPACER = \"spacer\"\n\n");
    out.push_str("    val ALL: Set<String> = setOf(\n");
    for spec in IR_ELEMENTS {
        out.push_str(&format!("        {},\n", kotlin_const_name(spec.wire)));
    }
    out.push_str("        SPACER,\n    )\n}\n");
    out
}

// ─── Native decoders from the Rust IR (schemars schema of `WidgetConfig`) ────────────────

/// Element fields resolved away before any native renderer sees them.
#[cfg(feature = "schema")]
const HOST_ONLY_FIELDS: &[&str] = &["bind", "when"];

/// Element types resolved away before any native renderer sees them.
#[cfg(feature = "schema")]
const HOST_ONLY_TYPES: &[&str] = &["repeat", "use"];

#[cfg(feature = "schema")]
fn variant_wire(v: &serde_json::Value) -> &str {
    v["properties"]["type"]["enum"][0]
        .as_str()
        .unwrap_or_default()
}

/// Numeric fields Swift keeps as `Double` (values, ratios); other numbers are `CGFloat`.
#[cfg(feature = "schema")]
const SWIFT_DOUBLE_FIELDS: &[&str] = &[
    "value",
    "total",
    "min",
    "max",
    "opacity",
    "minimumScaleFactor",
    "yMin",
    "yMax",
    "innerRadius",
    "aspectRatio",
    "layoutPriority",
    "scale",
    "rotation",
];

/// Union of every element's properties (style included) → wire type, from the schema
/// the host validates with. Panics on a name used with two different types.
#[cfg(feature = "schema")]
pub fn element_fields() -> std::collections::BTreeMap<String, WireType> {
    use serde_json::Value;
    let schema = serde_json::to_value(schemars::schema_for!(crate::models::WidgetConfig))
        .expect("schema serializes");
    let defs = schema["definitions"].as_object().expect("definitions");
    let variants = defs["WidgetElement"]["oneOf"].as_array().expect("oneOf");
    let mut out = std::collections::BTreeMap::new();
    for v in variants
        .iter()
        .filter(|v| !HOST_ONLY_TYPES.contains(&variant_wire(v)))
    {
        for (name, prop) in v["properties"].as_object().expect("properties") {
            if name == "type" || HOST_ONLY_FIELDS.contains(&name.as_str()) {
                continue;
            }
            // `set` (contract v3) is free-form JSON; Swift decodes it with the generated
            // `StatePatch` (end of WidgetElement.swift).
            let ty = if name == "set" {
                WireType::Named("StatePatch".into())
            } else {
                wire_type(prop, defs)
            };
            if let Some(prev) = out.insert(name.clone(), ty.clone()) {
                assert_eq!(prev, ty, "field `{name}` has two wire types");
            }
        }
    }
    let _: Option<Value> = None;
    out
}

/// Wire type of one schema property (nullability dropped: every field is optional).
#[cfg(feature = "schema")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireType {
    /// JSON number.
    Number,
    /// JSON integer.
    Integer,
    /// JSON boolean.
    Bool,
    /// JSON string, string enum included.
    String,
    /// A named object / union type shared with the hand-written support code.
    Named(String),
    /// Array of a wire type.
    Array(Box<WireType>),
}

#[cfg(feature = "schema")]
fn wire_type(
    prop: &serde_json::Value,
    defs: &serde_json::Map<String, serde_json::Value>,
) -> WireType {
    use serde_json::Value;
    if let Some(r) = prop.get("$ref").and_then(Value::as_str) {
        let name = r.rsplit('/').next().unwrap_or(r);
        return if is_string_enum(&defs[name]) {
            WireType::String
        } else {
            WireType::Named(name.to_string())
        };
    }
    if let Some(all) = prop.get("allOf").and_then(Value::as_array) {
        assert_eq!(all.len(), 1, "unsupported allOf {prop}");
        return wire_type(&all[0], defs);
    }
    if let Some(any) = prop.get("anyOf").and_then(Value::as_array) {
        let non_null: Vec<&Value> = any.iter().filter(|a| a["type"] != "null").collect();
        assert_eq!(non_null.len(), 1, "unsupported union {prop}");
        return wire_type(non_null[0], defs);
    }
    let types: Vec<&str> = match &prop["type"] {
        Value::String(t) => vec![t.as_str()],
        Value::Array(a) => a
            .iter()
            .filter_map(Value::as_str)
            .filter(|t| *t != "null")
            .collect(),
        _ => panic!("no type in {prop}"),
    };
    match types.as_slice() {
        ["number"] => WireType::Number,
        ["integer"] => WireType::Integer,
        ["boolean"] => WireType::Bool,
        ["string"] => WireType::String,
        ["array"] => WireType::Array(Box::new(wire_type(&prop["items"], defs))),
        other => panic!("unsupported type {other:?} in {prop}"),
    }
}

#[cfg(feature = "schema")]
fn is_string_enum(def: &serde_json::Value) -> bool {
    if def.get("enum").is_some() {
        return true;
    }
    def.get("oneOf")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|items| items.iter().all(|i| i["type"] == "string"))
}

#[cfg(feature = "schema")]
fn swift_type(name: &str, ty: &WireType) -> String {
    match ty {
        WireType::Number if SWIFT_DOUBLE_FIELDS.contains(&name) => "Double".into(),
        WireType::Number => "CGFloat".into(),
        WireType::Integer => "Int".into(),
        WireType::Bool => "Bool".into(),
        WireType::String => "String".into(),
        WireType::Named(n) => n.clone(),
        WireType::Array(inner) => format!("[{}]", swift_type(name, inner)),
    }
}

/// `swift/Sources/TauriWidgets/Generated/WidgetElement.swift`: the flat `Codable` element
/// every SwiftUI renderer decodes, generated from the Rust IR so no field can drift.
#[cfg(feature = "schema")]
pub fn emit_widget_element_swift() -> String {
    let mut out = String::from(
        "// AUTO-GENERATED by `cargo run --bin gen-native --features codegen`\n\
         // Do not edit. Source: the Rust IR (`src/models`), via its JSON Schema.\n\
         // Supporting types (ColorValue, PaddingValue, …) are hand-written in Models.swift.\n\n\
         import CoreGraphics\n\
         import Foundation\n\n\
         /// One widget element: the union of every element's fields, all optional but `type`.\n\
         public struct WidgetElement: Codable {\n\
         \x20   public let type: String\n",
    );
    let fields = element_fields();
    for (name, ty) in &fields {
        out.push_str(&format!(
            "    public let {name}: {}?\n",
            swift_type(name, ty)
        ));
    }
    // Lossy: a field of the wrong type reads as absent and a broken child is dropped, so
    // one bad value never blanks the whole widget.
    out.push_str(
        "\n    public init(from decoder: Decoder) throws {\n\
         \x20       let c = try decoder.container(keyedBy: CodingKeys.self)\n\
         \x20       type = try c.decode(String.self, forKey: .type)\n",
    );
    for (name, ty) in &fields {
        let t = swift_type(name, ty);
        if t == "[WidgetElement]" {
            out.push_str(&format!(
                "        {name} = (try? c.decodeIfPresent([LossyWidgetElement].self, forKey: .{name}))?.compactMap(\\.value)\n"
            ));
        } else {
            out.push_str(&format!(
                "        {name} = try? c.decodeIfPresent({t}.self, forKey: .{name})\n"
            ));
        }
    }
    out.push_str(
        "    }\n}\n\n\
         /// Decodes to `nil` instead of failing the enclosing array.\n\
         struct LossyWidgetElement: Decodable {\n\
         \x20   let value: WidgetElement?\n\
         \x20   init(from decoder: Decoder) throws { value = try? WidgetElement(from: decoder) }\n\
         }\n",
    );
    out.push_str(SWIFT_STATE_PATCH);
    out
}

/// Swift support type for the free-form `set` object (contract v3).
#[cfg(feature = "schema")]
const SWIFT_STATE_PATCH: &str = r#"
/// `set` on a button / toggle / link (contract v3): key → JSON value, as
/// `JSONSerialization` objects (`NSNull` for `null`), ready for `WidgetStateIntent`.
public struct StatePatch: Codable {
    public let values: [String: Any]

    public init(_ values: [String: Any]) { self.values = values }

    public init(from decoder: Decoder) throws {
        values = try decoder.singleValueContainer().decode([String: StatePatchJSON].self).mapValues(\.value)
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.singleValueContainer()
        try c.encode(values.mapValues(StatePatchJSON.init))
    }
}

/// Any JSON value (the values of a `StatePatch`).
struct StatePatchJSON: Codable {
    let value: Any

    init(_ value: Any) { self.value = value }

    init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if c.decodeNil() { value = NSNull() }
        else if let b = try? c.decode(Bool.self) { value = b }
        else if let i = try? c.decode(Int.self) { value = i }
        else if let d = try? c.decode(Double.self) { value = d }
        else if let s = try? c.decode(String.self) { value = s }
        else if let a = try? c.decode([StatePatchJSON].self) { value = a.map(\.value) }
        else { value = try c.decode([String: StatePatchJSON].self).mapValues(\.value) }
    }

    func encode(to encoder: Encoder) throws {
        var c = encoder.singleValueContainer()
        switch value {
        case let b as Bool: try c.encode(b)
        case let i as Int: try c.encode(i)
        case let d as Double: try c.encode(d)
        case let s as String: try c.encode(s)
        case let a as [Any]: try c.encode(a.map(StatePatchJSON.init))
        case let o as [String: Any]: try c.encode(o.mapValues(StatePatchJSON.init))
        default: try c.encodeNil()
        }
    }
}
"#;

/// `android/src/main/java/render/WireFields.kt`: every field name as a constant, plus the
/// fields each element type accepts (renderers read JSONObject keys through these).
#[cfg(feature = "schema")]
pub fn emit_wire_fields_kotlin() -> String {
    use serde_json::Value;
    let schema = serde_json::to_value(schemars::schema_for!(crate::models::WidgetConfig))
        .expect("schema serializes");
    let mut out = String::from(
        "// AUTO-GENERATED by `cargo run --bin gen-native --features codegen`\n\
         // Do not edit. Source: the Rust IR (`src/models`), via its JSON Schema.\n\n\
         package git.s00d.widgets\n\n\
         /** JSON keys of widget elements (`El.str(WireField.CONTENT)`). */\n\
         object WireField {\n",
    );
    for name in element_fields().keys() {
        out.push_str(&format!(
            "    const val {} = \"{name}\"\n",
            kotlin_field_const(name)
        ));
    }
    out.push_str("}\n\n/** Fields each element `type` accepts (style fields included). */\nval ELEMENT_FIELDS: Map<String, Set<String>> = mapOf(\n");
    let variants = schema["definitions"]["WidgetElement"]["oneOf"]
        .as_array()
        .expect("oneOf");
    let mut rows: Vec<(String, Vec<String>)> = variants
        .iter()
        .filter(|v| !HOST_ONLY_TYPES.contains(&variant_wire(v)))
        .map(|v| {
            let wire = v["properties"]["type"]["enum"][0]
                .as_str()
                .expect("type enum")
                .to_string();
            let mut fields: Vec<String> = v["properties"]
                .as_object()
                .expect("properties")
                .keys()
                .filter(|k| *k != "type" && !HOST_ONLY_FIELDS.contains(&k.as_str()))
                .map(|k| format!("WireField.{}", kotlin_field_const(k)))
                .collect();
            fields.sort();
            (wire, fields)
        })
        .collect();
    rows.sort();
    for (wire, fields) in rows {
        out.push_str(&format!(
            "    \"{wire}\" to setOf({}),\n",
            fields.join(", ")
        ));
    }
    let _: Option<Value> = None;
    out.push_str(")\n");
    out
}

/// `minimumScaleFactor` → `MINIMUM_SCALE_FACTOR`.
fn kotlin_field_const(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_uppercase());
    }
    out
}

fn swift_case_name(wire: &str) -> String {
    // Wire names are already lower camelCase; Swift keywords need backticks.
    match wire {
        "repeat" | "switch" | "case" | "default" | "in" | "is" => format!("`{wire}`"),
        _ => wire.to_string(),
    }
}

fn kotlin_const_name(wire: &str) -> String {
    wire.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capabilities::ELEMENT_TYPES;

    #[cfg(feature = "schema")]
    #[test]
    fn native_decoders_are_generated_and_current() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let swift = std::fs::read_to_string(
            root.join("swift/Sources/TauriWidgets/Generated/WidgetElement.swift"),
        )
        .unwrap_or_default();
        assert_eq!(swift, emit_widget_element_swift(), "run `pnpm codegen`");
        let kt = std::fs::read_to_string(root.join("android/src/main/java/render/WireFields.kt"))
            .unwrap_or_default();
        assert_eq!(kt, emit_wire_fields_kotlin(), "run `pnpm codegen`");
    }

    /// Every IR element type reaches every emitter: IR_ELEMENTS (+ spacer) → generated TS
    /// (and, with `schema`, every schema field is in the TS spec) → Swift and Kotlin wire
    /// catalogs → Kotlin field constant naming → `type_name`.
    #[test]
    fn every_element_type_reaches_every_emitter() {
        // 1. The IR spec lists exactly the element types.
        let wires: Vec<&str> = IR_ELEMENTS
            .iter()
            .map(|e| e.wire)
            .chain(["spacer"])
            .collect();
        for ty in ELEMENT_TYPES {
            assert!(wires.contains(ty), "IR_ELEMENTS missing wire type `{ty}`");
        }
        assert_eq!(wires.len(), ELEMENT_TYPES.len());

        // 2. Generated TS: one discriminated member per type, emitted from IR_SPEC.
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
        assert!(ts.contains("Emitter: `src/codegen.rs` IR_SPEC"));
        // The hand-written TS spec (IR_ELEMENTS + ElementStyle in supporting.ts) must name
        // every field the Rust IR accepts.
        #[cfg(feature = "schema")]
        {
            let missing: Vec<String> = element_fields()
                .keys()
                .filter(|f| !ts.contains(&format!("  {f}?:")) && !ts.contains(&format!("  {f}:")))
                .cloned()
                .collect();
            assert!(
                missing.is_empty(),
                "add to IR_ELEMENTS / supporting.ts: {missing:?}"
            );
        }

        // 3. Native wire catalogs.
        let swift = emit_wire_catalog_swift();
        let kt = emit_wire_catalog_kotlin();
        for ty in ELEMENT_TYPES {
            let needle = format!("= \"{ty}\"");
            assert!(swift.contains(&needle), "Swift catalog missing `{ty}`");
            assert!(kt.contains(&needle), "Kotlin catalog missing `{ty}`");
        }
        assert_eq!(
            kotlin_field_const("minimumScaleFactor"),
            "MINIMUM_SCALE_FACTOR"
        );
        assert_eq!(kotlin_field_const("x"), "X");

        // 4. The Rust side names the same type.
        use crate::models::{SpacerElement, WidgetElement};
        assert_eq!(
            WidgetElement::Spacer(SpacerElement { min_length: None }).type_name(),
            "spacer"
        );
    }
}
