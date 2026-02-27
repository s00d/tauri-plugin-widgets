use serde::{Deserialize, Serialize};

/// A single key-value item belonging to a widget group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetItem {
    pub key: String,
    pub value: String,
    pub group: String,
}

/// Configuration for creating a desktop widget window.
///
/// When `url` is omitted the plugin serves its built-in renderer
/// automatically (via a custom URI-scheme protocol).  In that case
/// `group` tells the renderer which config to load, and `size`
/// selects the layout family (`"small"`, `"medium"`, or `"large"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetWindowConfig {
    pub label: String,
    /// Frontend route or URL.  Leave empty / omit to use the built-in
    /// widget renderer that ships with the plugin.
    #[serde(default)]
    pub url: Option<String>,
    pub width: f64,
    pub height: f64,
    pub x: Option<f64>,
    pub y: Option<f64>,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default = "default_true")]
    pub skip_taskbar: bool,
    /// Widget group identifier — passed to the built-in renderer so it
    /// knows which config to load via `get_widget_config`.
    #[serde(default)]
    pub group: Option<String>,
    /// Size family the renderer should display: `"small"`, `"medium"`,
    /// or `"large"`.  Defaults to `"small"` when omitted.
    #[serde(default)]
    pub size: Option<String>,
}

fn default_true() -> bool {
    true
}

// ─── Widget UI Configuration ─────────────────────────────────────────────────

/// Top-level widget config with layouts per size family.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    pub small: Option<WidgetElement>,
    pub medium: Option<WidgetElement>,
    pub large: Option<WidgetElement>,
}

fn default_version() -> u32 {
    1
}

/// A UI element that can be a layout container or a leaf widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WidgetElement {
    // ── Layout containers ──
    #[serde(rename = "vstack")]
    VStack {
        #[serde(default)]
        children: Vec<WidgetElement>,
        #[serde(default)]
        spacing: Option<f64>,
        #[serde(default)]
        alignment: Option<HorizontalAlignment>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "hstack")]
    HStack {
        #[serde(default)]
        children: Vec<WidgetElement>,
        #[serde(default)]
        spacing: Option<f64>,
        #[serde(default)]
        alignment: Option<VerticalAlignment>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "zstack")]
    ZStack {
        #[serde(default)]
        children: Vec<WidgetElement>,
        #[serde(default)]
        alignment: Option<String>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "grid")]
    Grid {
        #[serde(default)]
        children: Vec<WidgetElement>,
        #[serde(default = "default_columns")]
        columns: u32,
        #[serde(default)]
        spacing: Option<f64>,
        #[serde(rename = "rowSpacing", default)]
        row_spacing: Option<f64>,
        #[serde(flatten)]
        style: ElementStyle,
    },

    /// Container with alignment — for cards, badges, overlays.
    #[serde(rename = "container")]
    Container {
        #[serde(default)]
        children: Vec<WidgetElement>,
        #[serde(rename = "contentAlignment", default)]
        content_alignment: Option<String>,
        #[serde(flatten)]
        style: ElementStyle,
    },

    // ── Leaf elements ──
    #[serde(rename = "text")]
    Text {
        content: String,
        #[serde(rename = "fontSize", default)]
        font_size: Option<f64>,
        #[serde(rename = "fontWeight", default)]
        font_weight: Option<FontWeight>,
        #[serde(rename = "fontDesign", default)]
        font_design: Option<FontDesign>,
        /// Semantic text style (uses Dynamic Type on Apple, sp on Android).
        /// Overrides `fontSize` when set.
        #[serde(rename = "textStyle", default)]
        text_style: Option<TextStyle>,
        #[serde(default)]
        color: Option<ColorValue>,
        #[serde(default)]
        alignment: Option<TextAlignment>,
        #[serde(rename = "lineLimit", default)]
        line_limit: Option<u32>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "image")]
    Image {
        /// SF Symbol name (Apple) or Material icon name (Android)
        #[serde(rename = "systemName", default)]
        system_name: Option<String>,
        /// Base64-encoded image data
        #[serde(default)]
        data: Option<String>,
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        size: Option<f64>,
        #[serde(default)]
        color: Option<ColorValue>,
        #[serde(rename = "contentMode", default)]
        content_mode: Option<ContentMode>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "progress")]
    Progress {
        value: f64,
        #[serde(default = "default_total")]
        total: f64,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        tint: Option<ColorValue>,
        #[serde(default)]
        color: Option<ColorValue>,
        #[serde(default)]
        bar_style: Option<ProgressStyle>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "gauge")]
    Gauge {
        value: f64,
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
        #[serde(default)]
        label: Option<String>,
        #[serde(rename = "currentValueLabel", default)]
        current_value_label: Option<String>,
        #[serde(default)]
        tint: Option<ColorValue>,
        #[serde(default)]
        color: Option<ColorValue>,
        #[serde(rename = "gaugeStyle", default)]
        gauge_style: Option<GaugeStyle>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "button")]
    Button {
        label: String,
        /// Deep link URL to open the app (used when no action is set)
        #[serde(default)]
        url: Option<String>,
        /// Action identifier — emits a `widget-action` Tauri event when tapped
        #[serde(default)]
        action: Option<String>,
        #[serde(default)]
        color: Option<ColorValue>,
        #[serde(rename = "backgroundColor", default)]
        background_color: Option<ColorValue>,
        #[serde(rename = "fontSize", default)]
        font_size: Option<f64>,
        #[serde(rename = "textAlignment", default)]
        text_alignment: Option<TextAlignment>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "toggle")]
    Toggle {
        #[serde(rename = "isOn")]
        is_on: bool,
        #[serde(default)]
        label: Option<String>,
        #[serde(default)]
        tint: Option<String>,
        /// Action identifier sent back to the app
        #[serde(default)]
        action: Option<String>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "divider")]
    Divider {
        #[serde(default)]
        color: Option<ColorValue>,
        #[serde(default)]
        thickness: Option<f64>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "spacer")]
    Spacer {
        #[serde(rename = "minLength", default)]
        min_length: Option<f64>,
    },
    #[serde(rename = "date")]
    Date {
        /// ISO 8601 date string
        date: String,
        #[serde(rename = "dateStyle", default)]
        date_style: Option<DateStyle>,
        #[serde(rename = "fontSize", default)]
        font_size: Option<f64>,
        #[serde(default)]
        color: Option<ColorValue>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    #[serde(rename = "chart")]
    Chart {
        #[serde(rename = "chartType")]
        chart_type: ChartType,
        #[serde(rename = "chartData")]
        chart_data: Vec<ChartDataPoint>,
        #[serde(default)]
        tint: Option<ColorValue>,
        #[serde(flatten)]
        style: ElementStyle,
    },
    /// Android-only collection list rendered via RemoteViewsService/ListView.
    #[serde(rename = "list")]
    List {
        #[serde(default)]
        items: Vec<ListItem>,
        #[serde(default)]
        spacing: Option<f64>,
        #[serde(rename = "fontSize", default)]
        font_size: Option<f64>,
        #[serde(default)]
        color: Option<ColorValue>,
        #[serde(flatten)]
        style: ElementStyle,
    },

    // ── New elements ──

    /// Tappable wrapper — makes nested content clickable.
    #[serde(rename = "link")]
    Link {
        #[serde(default)]
        children: Vec<WidgetElement>,
        /// Deep-link URL to open
        #[serde(default)]
        url: Option<String>,
        /// Action identifier — emits `widget-action` event
        #[serde(default)]
        action: Option<String>,
        #[serde(flatten)]
        style: ElementStyle,
    },

    /// Colored shape — circle, capsule, or rectangle.
    #[serde(rename = "shape")]
    Shape {
        #[serde(rename = "shapeType")]
        shape_type: ShapeType,
        #[serde(default)]
        fill: Option<ColorValue>,
        #[serde(default)]
        stroke: Option<ColorValue>,
        #[serde(rename = "strokeWidth", default)]
        stroke_width: Option<f64>,
        #[serde(default)]
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
        #[serde(default)]
        counting: Option<TimerCounting>,
        #[serde(rename = "fontSize", default)]
        font_size: Option<f64>,
        #[serde(rename = "fontWeight", default)]
        font_weight: Option<FontWeight>,
        #[serde(default)]
        color: Option<ColorValue>,
        #[serde(flatten)]
        style: ElementStyle,
    },

    /// Declarative canvas — draw arbitrary shapes via JSON commands.
    #[serde(rename = "canvas")]
    Canvas {
        width: f64,
        height: f64,
        #[serde(default)]
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
        #[serde(rename = "iconColor", default)]
        icon_color: Option<ColorValue>,
        #[serde(rename = "fontSize", default)]
        font_size: Option<f64>,
        #[serde(rename = "fontWeight", default)]
        font_weight: Option<FontWeight>,
        #[serde(default)]
        color: Option<ColorValue>,
        #[serde(default)]
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
#[serde(rename_all = "camelCase")]
pub struct ElementStyle {
    pub padding: Option<PaddingValue>,
    pub background: Option<BackgroundValue>,
    #[serde(rename = "cornerRadius")]
    pub corner_radius: Option<f64>,
    pub opacity: Option<f64>,
    pub frame: Option<FrameConfig>,
    pub border: Option<BorderConfig>,
    pub shadow: Option<ShadowConfig>,
    /// Clip content to a shape (e.g. circle avatar from square image).
    #[serde(rename = "clipShape")]
    pub clip_shape: Option<ClipShape>,
    /// Layout weight for flexible sizing inside stacks (like Android `layout_weight`).
    pub flex: Option<f64>,
}

/// Color value — hex string, semantic name, or adaptive `{ light, dark }` pair.
///
/// Semantic names: `"label"`, `"secondaryLabel"`, `"systemBackground"`,
/// `"secondarySystemBackground"`, `"accent"`, `"separator"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ColorValue {
    Solid(String),
    Adaptive { light: String, dark: String },
}

/// Clip shape for content masking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClipShape {
    Circle,
    Capsule,
    Rectangle,
}

/// Semantic text style — respects Dynamic Type / accessibility settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[serde(untagged)]
pub enum BackgroundValue {
    Solid(String),
    Gradient(GradientConfig),
    Adaptive { light: String, dark: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GradientConfig {
    /// `"linear"`, `"radial"`, or `"angular"`
    #[serde(rename = "gradientType")]
    pub gradient_type: GradientType,
    pub colors: Vec<String>,
    /// Direction for linear gradients
    #[serde(default)]
    pub direction: Option<GradientDirection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GradientType {
    Linear,
    Radial,
    Angular,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[serde(rename_all = "camelCase")]
pub struct ShadowConfig {
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub radius: Option<f64>,
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PaddingValue {
    Uniform(f64),
    Edges {
        top: Option<f64>,
        bottom: Option<f64>,
        leading: Option<f64>,
        trailing: Option<f64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameConfig {
    pub width: Option<f64>,
    pub height: Option<f64>,
    #[serde(rename = "maxWidth")]
    pub max_width: Option<FrameDimension>,
    #[serde(rename = "maxHeight")]
    pub max_height: Option<FrameDimension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FrameDimension {
    Fixed(f64),
    Keyword(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[serde(rename_all = "camelCase")]
pub enum FontDesign {
    Default,
    Monospaced,
    Rounded,
    Serif,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextAlignment {
    Leading,
    Center,
    Trailing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HorizontalAlignment {
    Leading,
    Center,
    Trailing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentMode {
    Fit,
    Fill,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProgressStyle {
    Linear,
    Circular,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GaugeStyle {
    Circular,
    Linear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DateStyle {
    Time,
    Date,
    Relative,
    Offset,
    Timer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ChartType {
    Bar,
    Line,
    Area,
    Pie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShapeType {
    Circle,
    Capsule,
    Rectangle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimerCounting {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartDataPoint {
    pub label: String,
    pub value: f64,
    pub color: Option<ColorValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub text: String,
    #[serde(default)]
    pub checked: Option<bool>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub payload: Option<String>,
}

// ─── Canvas drawing commands ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "draw", rename_all = "camelCase")]
pub enum CanvasDrawCommand {
    #[serde(rename = "circle")]
    Circle {
        cx: f64,
        cy: f64,
        r: f64,
        #[serde(default)]
        fill: Option<ColorValue>,
        #[serde(default)]
        stroke: Option<ColorValue>,
        #[serde(rename = "strokeWidth", default)]
        stroke_width: Option<f64>,
    },
    #[serde(rename = "line")]
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        #[serde(default)]
        stroke: Option<ColorValue>,
        #[serde(rename = "strokeWidth", default)]
        stroke_width: Option<f64>,
        #[serde(rename = "lineCap", default)]
        line_cap: Option<String>,
    },
    #[serde(rename = "rect")]
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        #[serde(default)]
        fill: Option<ColorValue>,
        #[serde(default)]
        stroke: Option<ColorValue>,
        #[serde(rename = "strokeWidth", default)]
        stroke_width: Option<f64>,
        #[serde(rename = "cornerRadius", default)]
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
        #[serde(default)]
        fill: Option<ColorValue>,
        #[serde(default)]
        stroke: Option<ColorValue>,
        #[serde(rename = "strokeWidth", default)]
        stroke_width: Option<f64>,
    },
    #[serde(rename = "text")]
    Text {
        x: f64,
        y: f64,
        content: String,
        #[serde(rename = "fontSize", default)]
        font_size: Option<f64>,
        #[serde(default)]
        color: Option<ColorValue>,
        #[serde(default)]
        anchor: Option<String>,
    },
    #[serde(rename = "path")]
    Path {
        /// SVG path data (e.g. `"M10 10 L90 90"`)
        d: String,
        #[serde(default)]
        fill: Option<ColorValue>,
        #[serde(default)]
        stroke: Option<ColorValue>,
        #[serde(rename = "strokeWidth", default)]
        stroke_width: Option<f64>,
    },
}
