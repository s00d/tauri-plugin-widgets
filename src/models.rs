use serde::{Deserialize, Serialize};

#[cfg(feature = "schema")]
use schemars::JsonSchema;

/// Configuration for creating a desktop widget window.
///
/// When `url` is omitted the plugin serves its built-in renderer
/// automatically (via a custom URI-scheme protocol).  In that case
/// `group` tells the renderer which config to load, and `size`
/// selects the layout family (`"small"`, `"medium"`, or `"large"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct WidgetWindowConfig {
    pub label: String,
    /// Frontend route or URL.  Leave empty / omit to use the built-in
    /// widget renderer that ships with the plugin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub width: f64,
    pub height: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default = "default_true")]
    pub skip_taskbar: bool,
    /// Widget group identifier — passed to the built-in renderer so it
    /// knows which config to load via `get_widget_config`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    /// Widget identity within the group (required for built-in renderer).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub widget_id: Option<String>,
    /// Size family the renderer should display: `"small"`, `"medium"`,
    /// or `"large"`.  Defaults to `"small"` when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
}

fn default_true() -> bool {
    true
}

// ─── Widget UI Configuration ─────────────────────────────────────────────────

/// Top-level widget config with layouts per size family.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct WidgetConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub small: Option<WidgetElement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub medium: Option<WidgetElement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub large: Option<WidgetElement>,
}

fn default_version() -> u32 {
    1
}

/// A UI element that can be a layout container or a leaf widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WidgetElement {
    // ── Layout containers ──
    #[serde(rename = "vstack")]
    VStack {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<WidgetElement>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spacing: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        alignment: Option<HorizontalAlignment>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "hstack")]
    HStack {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<WidgetElement>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spacing: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        alignment: Option<VerticalAlignment>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "zstack")]
    ZStack {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<WidgetElement>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        alignment: Option<String>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "grid")]
    Grid {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<WidgetElement>,
        #[serde(default = "default_columns")]
        columns: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spacing: Option<f64>,
        #[serde(
            rename = "rowSpacing",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        row_spacing: Option<f64>,
        #[serde(flatten)]
        style: ElementStyle,
    },

    /// Container with alignment — for cards, badges, overlays.
    #[serde(rename = "container")]
    Container {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<WidgetElement>,
        #[serde(
            rename = "contentAlignment",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        content_alignment: Option<String>,
        #[serde(flatten)]
        style: ElementStyle,
    },

    // ── Leaf elements ──
    #[serde(rename = "text")]
    Text {
        content: String,
        #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
        font_size: Option<f64>,
        #[serde(
            rename = "fontWeight",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        font_weight: Option<FontWeight>,
        #[serde(
            rename = "fontDesign",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        font_design: Option<FontDesign>,
        /// Semantic text style (uses Dynamic Type on Apple, sp on Android).
        /// Overrides `fontSize` when set.
        #[serde(rename = "textStyle", default, skip_serializing_if = "Option::is_none")]
        text_style: Option<TextStyle>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<ColorValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        alignment: Option<TextAlignment>,
        #[serde(rename = "lineLimit", default, skip_serializing_if = "Option::is_none")]
        line_limit: Option<u32>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "image")]
    Image {
        /// SF Symbol name (Apple) or Material icon name (Android)
        #[serde(
            rename = "systemName",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        system_name: Option<String>,
        /// Base64-encoded image data
        #[serde(default, skip_serializing_if = "Option::is_none")]
        data: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        size: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<ColorValue>,
        #[serde(
            rename = "contentMode",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        content_mode: Option<ContentMode>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "progress")]
    Progress {
        value: f64,
        #[serde(default = "default_total")]
        total: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tint: Option<ColorValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<ColorValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bar_style: Option<ProgressStyle>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "gauge")]
    Gauge {
        value: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(
            rename = "currentValueLabel",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        current_value_label: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tint: Option<ColorValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<ColorValue>,
        #[serde(
            rename = "gaugeStyle",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        gauge_style: Option<GaugeStyle>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "button")]
    Button {
        label: String,
        /// Deep link URL to open the app (used when no action is set)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// Action identifier — emits a `widget-action` Tauri event when tapped
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<ColorValue>,
        #[serde(
            rename = "backgroundColor",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        background_color: Option<ColorValue>,
        #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
        font_size: Option<f64>,
        #[serde(
            rename = "textAlignment",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        text_alignment: Option<TextAlignment>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "toggle")]
    Toggle {
        #[serde(rename = "isOn")]
        is_on: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tint: Option<ColorValue>,
        /// Action identifier sent back to the app
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action: Option<String>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "divider")]
    Divider {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<ColorValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        thickness: Option<f64>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "spacer")]
    Spacer {
        #[serde(rename = "minLength", default, skip_serializing_if = "Option::is_none")]
        min_length: Option<f64>,
    },
    #[serde(rename = "date")]
    Date {
        /// ISO 8601 date string
        date: String,
        #[serde(rename = "dateStyle", default, skip_serializing_if = "Option::is_none")]
        date_style: Option<DateStyle>,
        #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
        font_size: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<ColorValue>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "chart")]
    Chart {
        #[serde(rename = "chartType")]
        chart_type: ChartType,
        #[serde(default, rename = "chartData", skip_serializing_if = "Vec::is_empty")]
        chart_data: Vec<ChartDataPoint>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tint: Option<ColorValue>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    /// Android-only collection list rendered via RemoteViewsService/ListView.
    #[serde(rename = "list")]
    List {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        items: Vec<ListItem>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spacing: Option<f64>,
        #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
        font_size: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<ColorValue>,
        #[serde(flatten)]
        style: ElementStyle,
    },

    // ── New elements ──
    /// Tappable wrapper — makes nested content clickable.
    #[serde(rename = "link")]
    Link {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<WidgetElement>,
        /// Deep-link URL to open
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// Action identifier — emits `widget-action` event
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action: Option<String>,
        #[serde(flatten)]
        style: ElementStyle,
    },

    /// Colored shape — circle, capsule, or rectangle.
    #[serde(rename = "shape")]
    Shape {
        #[serde(rename = "shapeType")]
        shape_type: ShapeType,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<ColorValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<ColorValue>,
        #[serde(
            rename = "strokeWidth",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        stroke_width: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        size: Option<f64>,
        #[serde(flatten)]
        style: ElementStyle,
    },

    /// Live countdown/countup timer that updates without timeline refresh.
    #[serde(rename = "timer")]
    Timer {
        /// ISO 8601 target date
        #[serde(rename = "targetDate")]
        target_date: String,
        /// Count direction. Default: `down`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        counting: Option<TimerCounting>,
        #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
        font_size: Option<f64>,
        #[serde(
            rename = "fontWeight",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        font_weight: Option<FontWeight>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<ColorValue>,
        #[serde(flatten)]
        style: ElementStyle,
    },

    /// Declarative canvas — draw arbitrary shapes via JSON commands.
    #[serde(rename = "canvas")]
    Canvas {
        width: f64,
        height: f64,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        elements: Vec<CanvasDrawCommand>,
        #[serde(flatten)]
        style: ElementStyle,
    },

    /// Convenience element combining an SF Symbol icon with text.
    #[serde(rename = "label")]
    Label {
        text: String,
        #[serde(rename = "systemName")]
        system_name: String,
        #[serde(rename = "iconColor", default, skip_serializing_if = "Option::is_none")]
        icon_color: Option<ColorValue>,
        #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
        font_size: Option<f64>,
        #[serde(
            rename = "fontWeight",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        font_weight: Option<FontWeight>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<ColorValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spacing: Option<f64>,
        #[serde(flatten)]
        style: ElementStyle,
    },
}

fn default_columns() -> u32 {
    2
}
fn default_total() -> f64 {
    1.0
}

// ─── Shared style applied to any element ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ElementStyle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<PaddingValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<BackgroundValue>,
    #[serde(
        rename = "cornerRadius",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub corner_radius: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<FrameConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<BorderConfig>,
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
    Solid(String),
    Adaptive { light: String, dark: String },
}

/// Clip shape for content masking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ClipShape {
    Circle,
    Capsule,
    Rectangle,
}

/// Semantic text style — respects Dynamic Type / accessibility settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum TextStyle {
    LargeTitle,
    Title,
    Title2,
    Title3,
    Headline,
    Subheadline,
    Body,
    Callout,
    Footnote,
    Caption,
    Caption2,
}

/// Background: solid color string, adaptive pair, gradient, or material blur.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum BackgroundValue {
    Solid(String),
    Gradient(GradientConfig),
    Adaptive { light: String, dark: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GradientConfig {
    /// `"linear"`, `"radial"`, or `"angular"`
    #[serde(rename = "gradientType")]
    pub gradient_type: GradientType,
    pub colors: Vec<String>,
    /// Direction for linear gradients
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<GradientDirection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum GradientType {
    Linear,
    Radial,
    Angular,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum GradientDirection {
    TopToBottom,
    BottomToTop,
    LeadingToTrailing,
    TrailingToLeading,
    TopLeadingToBottomTrailing,
    TopTrailingToBottomLeading,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ShadowConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum PaddingValue {
    Uniform(f64),
    Edges {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        top: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bottom: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        leading: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trailing: Option<f64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FrameConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    #[serde(rename = "maxWidth", default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<FrameDimension>,
    #[serde(rename = "maxHeight", default, skip_serializing_if = "Option::is_none")]
    pub max_height: Option<FrameDimension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum FrameDimension {
    Fixed(f64),
    Keyword(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BorderConfig {
    pub color: String,
    #[serde(default = "default_border_width")]
    pub width: f64,
}

fn default_border_width() -> f64 {
    1.0
}

// ─── Enums ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum FontWeight {
    Ultralight,
    Thin,
    Light,
    Regular,
    Medium,
    Semibold,
    Bold,
    Heavy,
    Black,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum FontDesign {
    Default,
    Monospaced,
    Rounded,
    Serif,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum TextAlignment {
    Leading,
    Center,
    Trailing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum HorizontalAlignment {
    Leading,
    Center,
    Trailing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ContentMode {
    Fit,
    Fill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ProgressStyle {
    Linear,
    Circular,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum GaugeStyle {
    Circular,
    Linear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum DateStyle {
    Time,
    Date,
    Relative,
    Offset,
    Timer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ChartType {
    Bar,
    Line,
    Area,
    Pie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum ShapeType {
    Circle,
    Capsule,
    Rectangle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum TimerCounting {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ChartDataPoint {
    pub label: String,
    pub value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
}

// ─── Canvas drawing commands ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "draw", rename_all = "camelCase")]
pub enum CanvasDrawCommand {
    #[serde(rename = "circle")]
    Circle {
        cx: f64,
        cy: f64,
        r: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<ColorValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<ColorValue>,
        #[serde(
            rename = "strokeWidth",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        stroke_width: Option<f64>,
    },
    #[serde(rename = "line")]
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<ColorValue>,
        #[serde(
            rename = "strokeWidth",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        stroke_width: Option<f64>,
        #[serde(rename = "lineCap", default, skip_serializing_if = "Option::is_none")]
        line_cap: Option<String>,
    },
    #[serde(rename = "rect")]
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<ColorValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<ColorValue>,
        #[serde(
            rename = "strokeWidth",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        stroke_width: Option<f64>,
        #[serde(
            rename = "cornerRadius",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        corner_radius: Option<f64>,
    },
    #[serde(rename = "arc")]
    Arc {
        cx: f64,
        cy: f64,
        r: f64,
        #[serde(rename = "startAngle")]
        start_angle: f64,
        #[serde(rename = "endAngle")]
        end_angle: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<ColorValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<ColorValue>,
        #[serde(
            rename = "strokeWidth",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        stroke_width: Option<f64>,
    },
    #[serde(rename = "text")]
    Text {
        x: f64,
        y: f64,
        content: String,
        #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
        font_size: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<ColorValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<String>,
    },
    #[serde(rename = "path")]
    Path {
        /// SVG path data (e.g. `"M10 10 L90 90"`)
        d: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fill: Option<ColorValue>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stroke: Option<ColorValue>,
        #[serde(
            rename = "strokeWidth",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        stroke_width: Option<f64>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_none_omitted_from_json() {
        let cfg = WidgetConfig {
            version: 1,
            small: Some(WidgetElement::Text {
                content: "hi".into(),
                font_size: Some(12.0),
                font_weight: None,
                font_design: None,
                text_style: None,
                color: None,
                alignment: None,
                line_limit: None,
                style: ElementStyle::default(),
            }),
            medium: None,
            large: None,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        assert!(!json.contains("null"), "JSON must not contain null: {json}");
        assert!(!json.contains("\"medium\""));
        assert!(!json.contains("\"fontWeight\""));
        let back: WidgetConfig = serde_json::from_str(&json).unwrap();
        assert!(back.medium.is_none());
        match back.small.unwrap() {
            WidgetElement::Text {
                font_weight,
                content,
                ..
            } => {
                assert!(font_weight.is_none());
                assert_eq!(content, "hi");
            }
            other => panic!("expected text, got {other:?}"),
        }
    }

    #[test]
    fn fixtures_roundtrip_without_nulls() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        for name in [
            "weather.json",
            "tasks.json",
            "canvas.json",
            "android-list.json",
        ] {
            let path = dir.join(name);
            if !path.exists() {
                continue;
            }
            let raw = std::fs::read_to_string(&path).unwrap();
            let cfg: WidgetConfig = serde_json::from_str(&raw).unwrap();
            let out = serde_json::to_string(&cfg).unwrap();
            assert!(!out.contains("null"), "{name} serialized with null: {out}");
            let back: WidgetConfig = serde_json::from_str(&out).unwrap();
            assert_eq!(
                serde_json::to_value(&cfg).unwrap(),
                serde_json::to_value(&back).unwrap()
            );
        }
    }
}
