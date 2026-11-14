//! Shared style values: colors, backgrounds, gradients, shadows, padding,
//! frame/border config, and the small enums used across element fields.

use serde::{Deserialize, Serialize};

#[cfg(feature = "schema")]
use schemars::JsonSchema;

/// Shared visual style applied to any element (padding, background, frame, …).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ElementStyle {
    /// Inset padding (number or per-edge object).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<PaddingValue>,
    /// Solid, adaptive, or gradient background.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<BackgroundValue>,
    /// Corner radius in points.
    #[serde(
        rename = "cornerRadius",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub corner_radius: Option<f64>,
    /// Opacity from `0` (invisible) to `1` (opaque).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    /// Explicit width / height / max constraints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<FrameConfig>,
    /// Border color and width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<BorderConfig>,
    /// Drop shadow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow: Option<ShadowConfig>,
    /// Clip content to a shape (e.g. circle avatar from square image).
    #[serde(rename = "clipShape", default, skip_serializing_if = "Option::is_none")]
    pub clip_shape: Option<ClipShape>,
    /// Layout weight for flexible sizing inside stacks (like Android `layout_weight`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flex: Option<f64>,
    /// Hide this content where the OS redacts private data (iOS / macOS locked screen,
    /// StandBy). Ignored where no redaction exists.
    #[serde(
        rename = "privacySensitive",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub privacy_sensitive: Option<bool>,
    /// Text read by screen readers instead of the visible content.
    #[serde(
        rename = "accessibilityLabel",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub accessibility_label: Option<String>,
    /// Set fields from data with their JSON type: `{"value": "data.progress"}`.
    /// Use for numbers / booleans / arrays (`{{…}}` only fills strings).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bind: Option<std::collections::BTreeMap<String, String>>,
    /// Outer spacing (number or per-edge object), outside background and border.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin: Option<PaddingValue>,
    /// Keep width / height at this ratio (`1` = square).
    #[serde(
        rename = "aspectRatio",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub aspect_ratio: Option<f64>,
    /// Who keeps its size in a crowded stack: higher first. Default `0`.
    #[serde(
        rename = "layoutPriority",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub layout_priority: Option<f64>,
    /// Move right by points after layout (does not affect siblings).
    #[serde(rename = "offsetX", default, skip_serializing_if = "Option::is_none")]
    pub offset_x: Option<f64>,
    /// Move down by points after layout (does not affect siblings).
    #[serde(rename = "offsetY", default, skip_serializing_if = "Option::is_none")]
    pub offset_y: Option<f64>,
    /// Rotate by degrees (clockwise).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f64>,
    /// Scale factor (`1` = unchanged).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    /// Apple tinted / accented Home Screen: draw this part in the accent color
    /// (`widgetAccentable`). Ignored elsewhere.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accentable: Option<bool>,
    /// Show only when the condition holds (`{"path": "data.n", "gt": 0}`; see
    /// [`crate::resolve`]). Removed before rendering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<serde_json::Map<String, serde_json::Value>>")
    )]
    pub when: Option<serde_json::Value>,
    /// How this element changes between timeline entries / redraws (a hint):
    /// `numeric` animates digits, `opacity` cross-fades, `slide` pushes from the bottom.
    /// Apple and desktop animate; Android and Adaptive Cards ignore it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition: Option<Transition>,
}

/// Color value — hex string, semantic name, or adaptive `{ light, dark }` pair.
///
/// Semantic names: `"label"`, `"secondaryLabel"`, `"systemBackground"`,
/// `"secondarySystemBackground"`, `"accent"`, `"separator"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum ColorValue {
    /// Hex string or semantic color name.
    Solid(String),
    /// Distinct colors for light and dark appearance.
    Adaptive {
        /// Color used in light appearance.
        light: String,
        /// Color used in dark appearance.
        dark: String,
    },
}

impl From<&str> for ColorValue {
    fn from(value: &str) -> Self {
        ColorValue::Solid(value.to_string())
    }
}

impl From<String> for ColorValue {
    fn from(value: String) -> Self {
        ColorValue::Solid(value)
    }
}

/// Clip shape for content masking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ClipShape {
    /// `roundedRectangle` (uses `cornerRadius`).
    RoundedRectangle,
    /// `ellipse`.
    Ellipse,
    /// `circle`.
    Circle,
    /// `capsule`.
    Capsule,
    /// `rectangle`.
    Rectangle,
}

/// Semantic text style — respects Dynamic Type / accessibility settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum TextStyle {
    /// `large title`.
    LargeTitle,
    /// `title`.
    Title,
    /// `title2`.
    Title2,
    /// `title3`.
    Title3,
    /// `headline`.
    Headline,
    /// `subheadline`.
    Subheadline,
    /// `body`.
    Body,
    /// `callout`.
    Callout,
    /// `footnote`.
    Footnote,
    /// `caption`.
    Caption,
    /// `caption2`.
    Caption2,
}

/// Background: solid color string, adaptive pair, gradient, or material blur.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum BackgroundValue {
    /// Hex string or semantic color name.
    Solid(String),
    /// Linear, radial, or angular gradient.
    Gradient(GradientConfig),
    /// Distinct colors for light and dark appearance.
    Adaptive {
        /// Color used in light appearance.
        light: String,
        /// Color used in dark appearance.
        dark: String,
    },
}

impl From<&str> for BackgroundValue {
    fn from(value: &str) -> Self {
        BackgroundValue::Solid(value.to_string())
    }
}

impl From<String> for BackgroundValue {
    fn from(value: String) -> Self {
        BackgroundValue::Solid(value)
    }
}

/// Linear, radial, or angular gradient background.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GradientConfig {
    /// `"linear"`, `"radial"`, or `"angular"`
    #[serde(rename = "gradientType")]
    pub gradient_type: GradientType,
    /// Stop colors, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub colors: Vec<String>,
    /// Direction for linear gradients
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<GradientDirection>,
    /// Colors at explicit positions (`location` 0…1); replaces `colors` when set.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stops: Vec<GradientStop>,
    /// Start point in unit space (`{x:0,y:0}` = top leading); overrides `direction`.
    #[serde(
        rename = "startPoint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub start_point: Option<UnitPoint>,
    /// End point in unit space (`{x:1,y:1}` = bottom trailing).
    #[serde(rename = "endPoint", default, skip_serializing_if = "Option::is_none")]
    pub end_point: Option<UnitPoint>,
}

/// A gradient color at a position.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GradientStop {
    /// Hex color.
    pub color: String,
    /// Position `0…1` along the gradient.
    pub location: f64,
}

/// A point in unit coordinates (`0…1` on both axes, origin top leading).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct UnitPoint {
    /// Horizontal position, `0` = leading.
    pub x: f64,
    /// Vertical position, `0` = top.
    pub y: f64,
}

/// Gradient shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum GradientType {
    /// `linear`.
    Linear,
    /// `radial`.
    Radial,
    /// `angular`.
    Angular,
}

/// Direction for linear gradients.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum GradientDirection {
    /// `top to bottom`.
    TopToBottom,
    /// `bottom to top`.
    BottomToTop,
    /// `leading to trailing`.
    LeadingToTrailing,
    /// `trailing to leading`.
    TrailingToLeading,
    /// `top leading to bottom trailing`.
    TopLeadingToBottomTrailing,
    /// `top trailing to bottom leading`.
    TopTrailingToBottomLeading,
}

/// Drop shadow configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ShadowConfig {
    /// Shadow color (hex string or semantic name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Blur radius in points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,
    /// Horizontal offset in points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Vertical offset in points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
}

/// Inset padding — a single uniform value or per-edge overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum PaddingValue {
    /// Same padding on every edge.
    Uniform(f64),
    /// Independent padding per edge.
    Edges {
        /// Top edge padding.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        top: Option<f64>,
        /// Bottom edge padding.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bottom: Option<f64>,
        /// Leading (left in LTR) edge padding.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        leading: Option<f64>,
        /// Trailing (right in LTR) edge padding.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trailing: Option<f64>,
    },
}

impl From<f64> for PaddingValue {
    fn from(value: f64) -> Self {
        PaddingValue::Uniform(value)
    }
}

impl From<f32> for PaddingValue {
    fn from(value: f32) -> Self {
        PaddingValue::Uniform(f64::from(value))
    }
}

impl From<i32> for PaddingValue {
    fn from(value: i32) -> Self {
        PaddingValue::Uniform(f64::from(value))
    }
}

/// Explicit width / height / max-size constraints for an element.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FrameConfig {
    /// Fixed width in points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    /// Fixed height in points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    /// Maximum width — a fixed point value or `"infinity"`.
    #[serde(rename = "maxWidth", default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<FrameDimension>,
    /// Maximum height — a fixed point value or `"infinity"`.
    #[serde(rename = "maxHeight", default, skip_serializing_if = "Option::is_none")]
    pub max_height: Option<FrameDimension>,
    /// Minimum width in points.
    #[serde(rename = "minWidth", default, skip_serializing_if = "Option::is_none")]
    pub min_width: Option<f64>,
    /// Minimum height in points.
    #[serde(rename = "minHeight", default, skip_serializing_if = "Option::is_none")]
    pub min_height: Option<f64>,
}

/// A frame dimension — either a fixed point value or `"infinity"`.
#[derive(Debug, Clone)]
pub enum FrameDimension {
    /// Fixed point value.
    Fixed(f64),
    /// `"infinity"`.
    Infinity,
}

#[cfg(feature = "schema")]
impl JsonSchema for FrameDimension {
    fn schema_name() -> String {
        "FrameDimension".into()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::{
            InstanceType, Metadata, Schema, SchemaObject, SingleOrVec, SubschemaValidation,
        };

        Schema::Object(SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "A frame dimension — either a fixed point value or `\"infinity\"`.".into(),
                ),
                ..Default::default()
            })),
            subschemas: Some(Box::new(SubschemaValidation {
                any_of: Some(vec![
                    SchemaObject {
                        metadata: Some(Box::new(Metadata {
                            description: Some("Fixed point value.".into()),
                            ..Default::default()
                        })),
                        instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Number))),
                        format: Some("double".into()),
                        ..Default::default()
                    }
                    .into(),
                    SchemaObject {
                        metadata: Some(Box::new(Metadata {
                            description: Some("Keyword `\"infinity\"`.".into()),
                            ..Default::default()
                        })),
                        instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
                        enum_values: Some(vec!["infinity".into()]),
                        ..Default::default()
                    }
                    .into(),
                ]),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}

impl Serialize for FrameDimension {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Fixed(v) => serializer.serialize_f64(*v),
            Self::Infinity => serializer.serialize_str("infinity"),
        }
    }
}

impl<'de> Deserialize<'de> for FrameDimension {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Helper {
            Num(f64),
            Str(String),
        }
        match Helper::deserialize(deserializer)? {
            Helper::Num(v) => Ok(Self::Fixed(v)),
            Helper::Str(s) if s == "infinity" => Ok(Self::Infinity),
            Helper::Str(s) => Err(serde::de::Error::custom(format!(
                "unknown frame dimension keyword: {s}"
            ))),
        }
    }
}

/// Border color and width.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BorderConfig {
    /// Border color (hex string or semantic name).
    pub color: String,
    /// Border width in points. Default `1.0`.
    #[serde(default = "default_border_width")]
    pub width: f64,
}

fn default_border_width() -> f64 {
    1.0
}

/// Font weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum FontWeight {
    /// `ultralight`.
    Ultralight,
    /// `thin`.
    Thin,
    /// `light`.
    Light,
    /// `regular`.
    Regular,
    /// `medium`.
    Medium,
    /// `semibold`.
    Semibold,
    /// `bold`.
    Bold,
    /// `heavy`.
    Heavy,
    /// `black`.
    Black,
}

/// Font design (default, monospaced, rounded, serif).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum FontDesign {
    /// `default`.
    Default,
    /// `monospaced`.
    Monospaced,
    /// `rounded`.
    Rounded,
    /// `serif`.
    Serif,
}

/// Text alignment within the line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum TextAlignment {
    /// `leading`.
    Leading,
    /// `center`.
    Center,
    /// `trailing`.
    Trailing,
}

/// Horizontal alignment of children within a stack.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum HorizontalAlignment {
    /// `leading`.
    Leading,
    /// `center`.
    Center,
    /// `trailing`.
    Trailing,
}

/// Vertical alignment of children within a stack.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum VerticalAlignment {
    /// `top`.
    Top,
    /// `center`.
    Center,
    /// `bottom`.
    Bottom,
}

/// How an image fills its frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ContentMode {
    /// `fit`.
    Fit,
    /// `fill`.
    Fill,
    /// `stretch`: fill both axes, ignoring the aspect ratio.
    Stretch,
    /// `center`: natural size, centered, no scaling.
    Center,
}

/// Progress indicator style.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ProgressStyle {
    /// `linear`.
    Linear,
    /// `circular`.
    Circular,
}

/// Gauge visual style.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum GaugeStyle {
    /// `circular`.
    Circular,
    /// `linear`.
    Linear,
}

/// Date / relative-time display style.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum DateStyle {
    /// `time`.
    Time,
    /// `date`.
    Date,
    /// `relative`.
    Relative,
    /// `offset`.
    Offset,
    /// `timer`.
    Timer,
}

/// Chart kind.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ChartType {
    /// `bar`.
    #[default]
    Bar,
    /// `line`.
    Line,
    /// `area`.
    Area,
    /// `pie` (donut with `innerRadius`).
    Pie,
    /// `point`: scatter dots.
    Point,
}

/// Shape kind for [`crate::models::ShapeElement`].
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ShapeType {
    /// `circle`.
    #[default]
    Circle,
    /// `capsule`.
    Capsule,
    /// `rectangle`.
    Rectangle,
    /// `roundedRectangle` (uses `cornerRadius`).
    RoundedRectangle,
    /// `ellipse`.
    Ellipse,
}

/// Countdown/countup direction for [`crate::models::TimerElement`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum TimerCounting {
    /// `up`.
    Up,
    /// `down`.
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Any solid / adaptive colour and any uniform / per-edge padding survives
        /// serialize → deserialize with the same variant and values.
        #[test]
        fn color_and_padding_roundtrip(
            a in "[#a-zA-Z0-9]{0,8}",
            b in "[#a-zA-Z0-9]{0,8}",
            u in -4_000_000i32..4_000_000,
            edges in proptest::array::uniform4(proptest::option::of(0u32..40_000)),
        ) {
            // Quarter steps: exact in f64 and in JSON (serde_json's default float parser
            // is not round-trip exact for arbitrary f64).
            let u = f64::from(u) / 4.0;
            let edges = edges.map(|e| e.map(|q| f64::from(q) / 4.0));
            for c in [ColorValue::Solid(a.clone()), ColorValue::Adaptive { light: a.clone(), dark: b.clone() }] {
                let back: ColorValue = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
                match (&c, &back) {
                    (ColorValue::Solid(x), ColorValue::Solid(y)) => prop_assert_eq!(x, y),
                    (ColorValue::Adaptive { light: l1, dark: d1 }, ColorValue::Adaptive { light: l2, dark: d2 }) => {
                        prop_assert_eq!((l1, d1), (l2, d2))
                    }
                    _ => prop_assert!(false, "{:?} -> {:?}", c, back),
                }
            }
            let back: PaddingValue = serde_json::from_str(&serde_json::to_string(&PaddingValue::Uniform(u)).unwrap()).unwrap();
            prop_assert!(matches!(back, PaddingValue::Uniform(v) if v == u));
            let [top, bottom, leading, trailing] = edges;
            let p = PaddingValue::Edges { top, bottom, leading, trailing };
            let back: PaddingValue = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
            match back {
                PaddingValue::Edges { top: t, bottom: b, leading: l, trailing: tr } => {
                    prop_assert_eq!((t, b, l, tr), (top, bottom, leading, trailing))
                }
                // All-`None` edges serialize to `{}`, which is still an edges object.
                other => prop_assert!(false, "{:?}", other),
            }
        }
    }

    /// Wire spellings and defaults of the style vocabulary: uniform padding is a bare
    /// number → an empty style is `{}` → border width defaults to 1 → chart / shape
    /// defaults → camelCase enums → `infinity` is the only frame keyword.
    #[test]
    fn style_wire_spellings_and_defaults() {
        assert_eq!(
            serde_json::to_string(&PaddingValue::Uniform(8.0)).unwrap(),
            "8.0"
        );
        assert_eq!(
            serde_json::to_string(&ElementStyle::default()).unwrap(),
            "{}"
        );
        let b: BorderConfig = serde_json::from_str(r##"{"color":"#ffffff"}"##).unwrap();
        assert!((b.width - 1.0).abs() < f64::EPSILON);
        assert_eq!(b.color, "#ffffff");
        assert!(matches!(ChartType::default(), ChartType::Bar));
        assert!(matches!(ShapeType::default(), ShapeType::Circle));
        let w: FontWeight = serde_json::from_str(r#""semibold""#).unwrap();
        assert!(matches!(w, FontWeight::Semibold));
        let a: TextAlignment = serde_json::from_str(r#""trailing""#).unwrap();
        assert!(matches!(a, TextAlignment::Trailing));
        let t: TimerCounting = serde_json::from_str(r#""down""#).unwrap();
        assert!(matches!(t, TimerCounting::Down));
        let d: FrameDimension = serde_json::from_str(r#""infinity""#).unwrap();
        assert!(matches!(d, FrameDimension::Infinity));
        assert!(serde_json::from_str::<FrameDimension>(r#""auto""#).is_err());
    }
}

/// Distribution of stack children along the stack axis.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Justify {
    /// Packed at the start (default).
    Start,
    /// Packed in the middle.
    Center,
    /// Packed at the end.
    End,
    /// First and last at the edges, equal gaps between.
    SpaceBetween,
    /// Equal space around each child.
    SpaceAround,
    /// Equal gaps everywhere, edges included.
    SpaceEvenly,
}

/// Where text is cut when it does not fit.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum TruncationMode {
    /// `…end`.
    Head,
    /// `sta…end`.
    Middle,
    /// `start…` (default).
    Tail,
}

/// SF Symbol rendering mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum SymbolRendering {
    /// One color (default).
    Monochrome,
    /// One color, layered opacity.
    Hierarchical,
    /// Up to three `paletteColors`.
    Palette,
    /// The symbol's own colors.
    Multicolor,
}

/// Button look.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ButtonVariant {
    /// Tinted rounded background (default).
    Bordered,
    /// Filled with `backgroundColor` / accent, white label.
    Prominent,
    /// Label only.
    Plain,
    /// Outline in the label color, no fill.
    Outline,
}

/// Toggle look.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ToggleVariant {
    /// Switch (default).
    Switch,
    /// Checkbox.
    Checkbox,
}

/// Line / area curve.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ChartInterpolation {
    /// Straight segments (default).
    Linear,
    /// Smooth curve through every point.
    Monotone,
    /// Steps.
    Step,
}

/// Content change animation hint (`transition`).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Transition {
    /// Swap without animation.
    None,
    /// Cross-fade.
    Opacity,
    /// Push the new content in from the bottom.
    Slide,
    /// Roll changed digits (`contentTransition(.numericText())`).
    Numeric,
}
