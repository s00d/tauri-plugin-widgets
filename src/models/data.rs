//! Data payloads carried by elements: chart points, list rows, and the
//! declarative canvas draw-command union.

use serde::{Deserialize, Serialize};

#[cfg(feature = "schema")]
use schemars::JsonSchema;

use super::style::ColorValue;

/// A single labeled data point for [`crate::models::ChartElement`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ChartDataPoint {
    /// Category / x-axis label.
    pub label: String,
    /// Value plotted for this point.
    pub value: f64,
    /// Optional per-point color override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorValue>,
}

/// A single row for [`crate::models::ListElement`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    /// Row text.
    pub text: String,
    /// Optional checked/checkbox marker.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    /// Action identifier — emits a `widget-action` event when tapped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// Opaque payload forwarded alongside the action event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
}

/// A single declarative draw command for [`crate::models::CanvasElement`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "draw", rename_all = "camelCase")]
pub enum CanvasDrawCommand {
    /// Filled / stroked circle.
    #[serde(rename = "circle")]
    Circle {
        /// Center x-coordinate.
        cx: f64,
        /// Center y-coordinate.
        cy: f64,
        /// Radius.
        r: f64,
        /// Fill color.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<ColorValue>,
        /// Stroke color.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<ColorValue>,
        /// Stroke width in points.
        #[serde(
            rename = "strokeWidth",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        stroke_width: Option<f64>,
    },
    /// Straight line segment.
    #[serde(rename = "line")]
    Line {
        /// Start x-coordinate.
        x1: f64,
        /// Start y-coordinate.
        y1: f64,
        /// End x-coordinate.
        x2: f64,
        /// End y-coordinate.
        y2: f64,
        /// Stroke color.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<ColorValue>,
        /// Stroke width in points.
        #[serde(
            rename = "strokeWidth",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        stroke_width: Option<f64>,
        /// Line cap style (`"butt"`, `"round"`, or `"square"`).
        #[serde(rename = "lineCap", default, skip_serializing_if = "Option::is_none")]
        line_cap: Option<String>,
    },
    /// Filled / stroked rectangle, optionally rounded.
    #[serde(rename = "rect")]
    Rect {
        /// Top-left x-coordinate.
        x: f64,
        /// Top-left y-coordinate.
        y: f64,
        /// Rectangle width.
        width: f64,
        /// Rectangle height.
        height: f64,
        /// Fill color.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<ColorValue>,
        /// Stroke color.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<ColorValue>,
        /// Stroke width in points.
        #[serde(
            rename = "strokeWidth",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        stroke_width: Option<f64>,
        /// Corner radius in points.
        #[serde(
            rename = "cornerRadius",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        corner_radius: Option<f64>,
    },
    /// Filled / stroked arc.
    #[serde(rename = "arc")]
    Arc {
        /// Center x-coordinate.
        cx: f64,
        /// Center y-coordinate.
        cy: f64,
        /// Radius.
        r: f64,
        /// Start angle in degrees.
        #[serde(rename = "startAngle")]
        start_angle: f64,
        /// End angle in degrees.
        #[serde(rename = "endAngle")]
        end_angle: f64,
        /// Fill color.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<ColorValue>,
        /// Stroke color.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<ColorValue>,
        /// Stroke width in points.
        #[serde(
            rename = "strokeWidth",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        stroke_width: Option<f64>,
    },
    /// Text label drawn at a point.
    #[serde(rename = "text")]
    Text {
        /// x-coordinate of the anchor point.
        x: f64,
        /// y-coordinate of the anchor point.
        y: f64,
        /// Text to draw.
        content: String,
        /// Font size in points.
        #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
        font_size: Option<f64>,
        /// Text color.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<ColorValue>,
        /// Text anchor (`"start"`, `"middle"`, or `"end"`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<String>,
    },
    /// Arbitrary vector path.
    #[serde(rename = "path")]
    Path {
        /// SVG path data (e.g. `"M10 10 L90 90"`)
        d: String,
        /// Fill color.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<ColorValue>,
        /// Stroke color.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<ColorValue>,
        /// Stroke width in points.
        #[serde(
            rename = "strokeWidth",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        stroke_width: Option<f64>,
    },
}
