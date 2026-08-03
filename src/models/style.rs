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
    pub colors: Vec<String>,
    /// Direction for linear gradients
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<GradientDirection>,
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
}

/// A frame dimension — either a fixed point value or `"infinity"`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub enum FrameDimension {
    /// Fixed point value.
    Fixed(f64),
    /// `"infinity"`.
    Infinity,
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
    /// `pie`.
    Pie,
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

    #[test]
    fn color_value_roundtrip_solid_and_adaptive() {
        let solid = ColorValue::Solid("#ff0000".into());
        let s = serde_json::to_string(&solid).unwrap();
        let back: ColorValue = serde_json::from_str(&s).unwrap();
        assert!(matches!(back, ColorValue::Solid(ref x) if x == "#ff0000"));

        let adaptive = ColorValue::Adaptive {
            light: "#fff".into(),
            dark: "#000".into(),
        };
        let s = serde_json::to_string(&adaptive).unwrap();
        let back: ColorValue = serde_json::from_str(&s).unwrap();
        assert!(matches!(
            back,
            ColorValue::Adaptive {
                ref light,
                ref dark
            } if light == "#fff" && dark == "#000"
        ));
    }

    #[test]
    fn padding_value_roundtrip() {
        let u = PaddingValue::Uniform(8.0);
        let s = serde_json::to_string(&u).unwrap();
        assert_eq!(s, "8.0");
        let back: PaddingValue = serde_json::from_str(&s).unwrap();
        assert!(matches!(back, PaddingValue::Uniform(v) if (v - 8.0).abs() < f64::EPSILON));

        let edges = PaddingValue::Edges {
            top: Some(1.0),
            bottom: Some(2.0),
            leading: Some(3.0),
            trailing: None,
        };
        let s = serde_json::to_string(&edges).unwrap();
        let back: PaddingValue = serde_json::from_str(&s).unwrap();
        assert!(matches!(back, PaddingValue::Edges { .. }));
    }

    #[test]
    fn element_style_default_is_empty_object() {
        let s = serde_json::to_string(&ElementStyle::default()).unwrap();
        assert_eq!(s, "{}");
    }

    #[test]
    fn border_config_default_width() {
        let b: BorderConfig =
            serde_json::from_str(r##"{"color":"#ffffff"}"##).unwrap();
        assert!((b.width - 1.0).abs() < f64::EPSILON);
        assert_eq!(b.color, "#ffffff");
    }

    #[test]
    fn chart_shape_defaults() {
        assert!(matches!(ChartType::default(), ChartType::Bar));
        assert!(matches!(ShapeType::default(), ShapeType::Circle));
    }

    #[test]
    fn enums_deserialize_camel_case() {
        let w: FontWeight = serde_json::from_str(r#""semibold""#).unwrap();
        assert!(matches!(w, FontWeight::Semibold));
        let a: TextAlignment = serde_json::from_str(r#""trailing""#).unwrap();
        assert!(matches!(a, TextAlignment::Trailing));
        let t: TimerCounting = serde_json::from_str(r#""down""#).unwrap();
        assert!(matches!(t, TimerCounting::Down));
    }

    #[test]
    fn frame_dimension_accepts_infinity() {
        let d: FrameDimension = serde_json::from_str(r#""infinity""#).unwrap();
        assert!(matches!(d, FrameDimension::Infinity));
    }

    #[test]
    fn frame_dimension_rejects_unknown_keyword() {
        assert!(serde_json::from_str::<FrameDimension>(r#""auto""#).is_err());
    }
}
