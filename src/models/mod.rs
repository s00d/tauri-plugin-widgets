//! Widget IR types: [`WidgetConfig`], [`WidgetElement`], and shared style values.
//!
//! Element structs, [`WidgetConfig`]/[`WidgetElement`], builders, and tests
//! live here; shared style values are in [`style`] and data payloads
//! ([`ChartDataPoint`], [`ListItem`], [`CanvasDrawCommand`]) are in [`data`].

mod data;
mod style;
pub use data::*;
pub use style::*;

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
    /// Tauri window label (must be unique).
    pub label: String,
    /// Frontend route or URL.  Leave empty / omit to use the built-in
    /// widget renderer that ships with the plugin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Window width in logical pixels.
    pub width: f64,
    /// Window height in logical pixels.
    pub height: f64,
    /// Optional X position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    /// Optional Y position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    /// Keep the widget above other windows.
    #[serde(default)]
    pub always_on_top: bool,
    /// Hide from the taskbar / dock.
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
    /// Schema version. Defaults to `1`.
    #[serde(default = "default_version")]
    pub version: u32,
    /// Layout for the small size family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub small: Option<WidgetElement>,
    /// Layout for the medium size family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub medium: Option<WidgetElement>,
    /// Layout for the large size family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub large: Option<WidgetElement>,
}

impl Default for WidgetConfig {
    fn default() -> Self {
        Self {
            version: 1,
            small: None,
            medium: None,
            large: None,
        }
    }
}

impl WidgetConfig {
    /// Config with only the `small` size family set.
    pub fn small(el: impl Into<WidgetElement>) -> Self {
        Self {
            small: Some(el.into()),
            ..Default::default()
        }
    }

    /// Set the `medium` size family.
    pub fn with_medium(mut self, el: impl Into<WidgetElement>) -> Self {
        self.medium = Some(el.into());
        self
    }

    /// Set the `large` size family.
    pub fn with_large(mut self, el: impl Into<WidgetElement>) -> Self {
        self.large = Some(el.into());
        self
    }
}

fn default_version() -> u32 {
    1
}

/// A UI element that can be a layout container or a leaf widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WidgetElement {
    /// Vertical stack of children.
    #[serde(rename = "vstack")]
    VStack(VStackElement),
    /// Horizontal stack of children.
    #[serde(rename = "hstack")]
    HStack(HStackElement),
    /// Overlay stack — children layered on top of each other.
    #[serde(rename = "zstack")]
    ZStack(ZStackElement),
    /// Fixed-column grid of children.
    #[serde(rename = "grid")]
    Grid(GridElement),
    /// Single-child wrapper for cards, badges, and overlays.
    #[serde(rename = "container")]
    Container(ContainerElement),
    /// Text label with optional semantic typography.
    #[serde(rename = "text")]
    Text(TextElement),
    /// Image from SF Symbol / drawable name, base64 data, or URL.
    #[serde(rename = "image")]
    Image(ImageElement),
    /// Linear or circular progress indicator.
    #[serde(rename = "progress")]
    Progress(ProgressElement),
    /// Circular or capacity-style gauge.
    #[serde(rename = "gauge")]
    Gauge(GaugeElement),
    /// Tappable button that opens a URL or emits `widget-action`.
    #[serde(rename = "button")]
    Button(ButtonElement),
    /// On/off toggle control.
    #[serde(rename = "toggle")]
    Toggle(ToggleElement),
    /// Horizontal or vertical rule.
    #[serde(rename = "divider")]
    Divider(DividerElement),
    /// Flexible empty space.
    #[serde(rename = "spacer")]
    Spacer(SpacerElement),
    /// Formatted date / relative time display.
    #[serde(rename = "date")]
    Date(DateElement),
    /// Bar, line, area, or pie chart.
    #[serde(rename = "chart")]
    Chart(ChartElement),
    /// Collection list of rows (text, optional checked marker and action).
    #[serde(rename = "list")]
    List(ListElement),
    /// Tappable wrapper — makes nested content clickable.
    #[serde(rename = "link")]
    Link(LinkElement),
    /// Colored shape — circle, capsule, or rectangle.
    #[serde(rename = "shape")]
    Shape(ShapeElement),
    /// Live countdown/countup timer that updates without timeline refresh.
    #[serde(rename = "timer")]
    Timer(TimerElement),
    /// Declarative canvas — draw arbitrary shapes via JSON commands.
    #[serde(rename = "canvas")]
    Canvas(CanvasElement),
    /// Convenience element combining an SF Symbol / icon with text.
    #[serde(rename = "label")]
    Label(LabelElement),
}

/// Vertical stack of children.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct VStackElement {
    /// Child elements, top to bottom.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<WidgetElement>,
    /// Space between children (points).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spacing: Option<f64>,
    /// Horizontal alignment of children.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alignment: Option<HorizontalAlignment>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Horizontal stack of children.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct HStackElement {
    /// Child elements, leading to trailing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<WidgetElement>,
    /// Space between children (points).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spacing: Option<f64>,
    /// Vertical alignment of children.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alignment: Option<VerticalAlignment>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Overlay stack — children layered on top of each other.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ZStackElement {
    /// Layered children (later draw on top).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<WidgetElement>,
    /// Alignment of layers within the stack (e.g. `center`, `topLeading`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alignment: Option<String>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Fixed-column grid of children.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GridElement {
    /// Grid cells in row-major order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<WidgetElement>,
    /// Number of columns. Default `2`.
    #[serde(default = "default_columns")]
    pub columns: u32,
    /// Column spacing (points).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spacing: Option<f64>,
    /// Row spacing (points).
    #[serde(
        rename = "rowSpacing",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub row_spacing: Option<f64>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Single-child wrapper for cards, badges, and overlays.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ContainerElement {
    /// Nested content (typically one child).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<WidgetElement>,
    /// Content alignment inside the box (e.g. `center`, `topLeading`).
    #[serde(
        rename = "contentAlignment",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub content_alignment: Option<String>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Text label with optional semantic typography.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TextElement {
    /// String to display.
    pub content: String,
    /// Font size in points (overridden by `textStyle` when set).
    #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    /// Font weight.
    #[serde(
        rename = "fontWeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub font_weight: Option<FontWeight>,
    /// Font design (default, monospaced, rounded, serif).
    #[serde(
        rename = "fontDesign",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub font_design: Option<FontDesign>,
    /// Semantic text style (uses Dynamic Type on Apple, sp on Android).
    /// Overrides `fontSize` when set.
    #[serde(rename = "textStyle", default, skip_serializing_if = "Option::is_none")]
    pub text_style: Option<TextStyle>,
    /// Text color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorValue>,
    /// Text alignment within the line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alignment: Option<TextAlignment>,
    /// Maximum number of lines before truncation.
    #[serde(rename = "lineLimit", default, skip_serializing_if = "Option::is_none")]
    pub line_limit: Option<u32>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

impl TextElement {
    /// Set explicit point size (ignored when `text_style` is set).
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Set font weight.
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = Some(weight);
        self
    }

    /// Set text color.
    pub fn color(mut self, color: impl Into<ColorValue>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// Image from SF Symbol / drawable name, base64 data, or URL.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ImageElement {
    /// SF Symbol name (Apple) or Material / drawable name (Android).
    #[serde(
        rename = "systemName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub system_name: Option<String>,
    /// Base64-encoded image data (with or without `data:image/...;base64,` prefix).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Remote image URL (platform support varies — see capability matrix).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Display size in points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
    /// Tint color for template / symbol images.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorValue>,
    /// How the image fills its frame (`fit` or `fill`).
    #[serde(
        rename = "contentMode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub content_mode: Option<ContentMode>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Linear or circular progress indicator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ProgressElement {
    /// Current value. Clamped to `0..=total` by renderers.
    pub value: f64,
    /// Denominator for the ratio. Default `1.0`.
    #[serde(default = "default_total")]
    pub total: f64,
    /// Caption above the bar. Always set on Android so hosts never show null.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Accent / fill color for the completed portion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tint: Option<ColorValue>,
    /// Track / label color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorValue>,
    /// `linear` (default) or `circular`.
    #[serde(
        rename = "barStyle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub bar_style: Option<ProgressStyle>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

impl Default for ProgressElement {
    fn default() -> Self {
        Self {
            value: 0.0,
            total: 1.0,
            label: None,
            tint: None,
            color: None,
            bar_style: None,
            style: ElementStyle::default(),
        }
    }
}

/// Circular or capacity-style gauge.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GaugeElement {
    /// Current value within `[min, max]`.
    pub value: f64,
    /// Lower bound. Default `0`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Upper bound. Default `1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Optional caption.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Text shown for the current value (e.g. `"72%"`).
    #[serde(
        rename = "currentValueLabel",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub current_value_label: Option<String>,
    /// Accent color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tint: Option<ColorValue>,
    /// Secondary / track color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorValue>,
    /// Visual style (e.g. `circular`).
    #[serde(
        rename = "gaugeStyle",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub gauge_style: Option<GaugeStyle>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Tappable button that opens a URL or emits `widget-action`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ButtonElement {
    /// Button label text.
    pub label: String,
    /// Deep link URL to open the app (used when no action is set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Action identifier — emits a `widget-action` Tauri event when tapped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// Label text color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorValue>,
    /// Button background color.
    #[serde(
        rename = "backgroundColor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub background_color: Option<ColorValue>,
    /// Label font size in points.
    #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    /// Label text alignment.
    #[serde(
        rename = "textAlignment",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub text_alignment: Option<TextAlignment>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// On/off toggle control.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ToggleElement {
    /// Whether the toggle is on.
    #[serde(rename = "isOn")]
    pub is_on: bool,
    /// Optional label beside the control.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Accent color when on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tint: Option<ColorValue>,
    /// Action identifier sent back to the app.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Horizontal or vertical rule.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DividerElement {
    /// Line color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorValue>,
    /// Line thickness in points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thickness: Option<f64>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Flexible empty space.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SpacerElement {
    /// Minimum length along the parent axis (points).
    #[serde(rename = "minLength", default, skip_serializing_if = "Option::is_none")]
    pub min_length: Option<f64>,
}

/// Formatted date / relative time display.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DateElement {
    /// ISO 8601 date string.
    pub date: String,
    /// Display style (`time`, `date`, `relative`, `offset`, `timer`).
    #[serde(rename = "dateStyle", default, skip_serializing_if = "Option::is_none")]
    pub date_style: Option<DateStyle>,
    /// Font size in points.
    #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    /// Text color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorValue>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Bar, line, area, or pie chart.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ChartElement {
    /// Chart kind: `bar`, `line`, `area`, or `pie`.
    #[serde(rename = "chartType")]
    pub chart_type: ChartType,
    /// Data points (`label` + `value`, optional per-point `color`).
    #[serde(default, rename = "chartData", skip_serializing_if = "Vec::is_empty")]
    pub chart_data: Vec<ChartDataPoint>,
    /// Default series tint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tint: Option<ColorValue>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Collection list of rows (text, optional checked marker and action).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ListElement {
    /// Row items.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<ListItem>,
    /// Space between rows (points).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spacing: Option<f64>,
    /// Row text font size.
    #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    /// Row text color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorValue>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Tappable wrapper — makes nested content clickable.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LinkElement {
    /// Nested content to wrap.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<WidgetElement>,
    /// Deep-link URL to open.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Action identifier — emits `widget-action` event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Colored shape — circle, capsule, or rectangle.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ShapeElement {
    /// Shape kind.
    #[serde(rename = "shapeType")]
    pub shape_type: ShapeType,
    /// Fill color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill: Option<ColorValue>,
    /// Stroke color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<ColorValue>,
    /// Stroke width in points.
    #[serde(
        rename = "strokeWidth",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub stroke_width: Option<f64>,
    /// Bounding size in points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Live countdown/countup timer that updates without timeline refresh.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TimerElement {
    /// ISO 8601 target date.
    #[serde(rename = "targetDate")]
    pub target_date: String,
    /// Count direction. Default: `down`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counting: Option<TimerCounting>,
    /// Font size in points.
    #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    /// Font weight.
    #[serde(
        rename = "fontWeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub font_weight: Option<FontWeight>,
    /// Text color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorValue>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Declarative canvas — draw arbitrary shapes via JSON commands.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct CanvasElement {
    /// Canvas width in points.
    pub width: f64,
    /// Canvas height in points.
    pub height: f64,
    /// Draw commands (`circle`, `line`, `rect`, `arc`, `text`, `path`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<CanvasDrawCommand>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

/// Convenience element combining an SF Symbol / icon with text.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LabelElement {
    /// Label text.
    pub text: String,
    /// SF Symbol or platform icon name.
    #[serde(rename = "systemName")]
    pub system_name: String,
    /// Icon tint color.
    #[serde(rename = "iconColor", default, skip_serializing_if = "Option::is_none")]
    pub icon_color: Option<ColorValue>,
    /// Text font size.
    #[serde(rename = "fontSize", default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    /// Text font weight.
    #[serde(
        rename = "fontWeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub font_weight: Option<FontWeight>,
    /// Text color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<ColorValue>,
    /// Space between icon and text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spacing: Option<f64>,
    #[serde(flatten)]
    /// Shared visual style (padding, background, frame, …).
    pub style: ElementStyle,
}

impl From<VStackElement> for WidgetElement {
    fn from(value: VStackElement) -> Self {
        WidgetElement::VStack(value)
    }
}

impl From<HStackElement> for WidgetElement {
    fn from(value: HStackElement) -> Self {
        WidgetElement::HStack(value)
    }
}

impl From<ZStackElement> for WidgetElement {
    fn from(value: ZStackElement) -> Self {
        WidgetElement::ZStack(value)
    }
}

impl From<GridElement> for WidgetElement {
    fn from(value: GridElement) -> Self {
        WidgetElement::Grid(value)
    }
}

impl From<ContainerElement> for WidgetElement {
    fn from(value: ContainerElement) -> Self {
        WidgetElement::Container(value)
    }
}

impl From<TextElement> for WidgetElement {
    fn from(value: TextElement) -> Self {
        WidgetElement::Text(value)
    }
}

impl From<ImageElement> for WidgetElement {
    fn from(value: ImageElement) -> Self {
        WidgetElement::Image(value)
    }
}

impl From<ProgressElement> for WidgetElement {
    fn from(value: ProgressElement) -> Self {
        WidgetElement::Progress(value)
    }
}

impl From<GaugeElement> for WidgetElement {
    fn from(value: GaugeElement) -> Self {
        WidgetElement::Gauge(value)
    }
}

impl From<ButtonElement> for WidgetElement {
    fn from(value: ButtonElement) -> Self {
        WidgetElement::Button(value)
    }
}

impl From<ToggleElement> for WidgetElement {
    fn from(value: ToggleElement) -> Self {
        WidgetElement::Toggle(value)
    }
}

impl From<DividerElement> for WidgetElement {
    fn from(value: DividerElement) -> Self {
        WidgetElement::Divider(value)
    }
}

impl From<SpacerElement> for WidgetElement {
    fn from(value: SpacerElement) -> Self {
        WidgetElement::Spacer(value)
    }
}

impl From<DateElement> for WidgetElement {
    fn from(value: DateElement) -> Self {
        WidgetElement::Date(value)
    }
}

impl From<ChartElement> for WidgetElement {
    fn from(value: ChartElement) -> Self {
        WidgetElement::Chart(value)
    }
}

impl From<ListElement> for WidgetElement {
    fn from(value: ListElement) -> Self {
        WidgetElement::List(value)
    }
}

impl From<LinkElement> for WidgetElement {
    fn from(value: LinkElement) -> Self {
        WidgetElement::Link(value)
    }
}

impl From<ShapeElement> for WidgetElement {
    fn from(value: ShapeElement) -> Self {
        WidgetElement::Shape(value)
    }
}

impl From<TimerElement> for WidgetElement {
    fn from(value: TimerElement) -> Self {
        WidgetElement::Timer(value)
    }
}

impl From<CanvasElement> for WidgetElement {
    fn from(value: CanvasElement) -> Self {
        WidgetElement::Canvas(value)
    }
}

impl From<LabelElement> for WidgetElement {
    fn from(value: LabelElement) -> Self {
        WidgetElement::Label(value)
    }
}

/// Build a [`TextElement`] with the given content.
pub fn text(content: impl Into<String>) -> TextElement {
    TextElement {
        content: content.into(),
        ..Default::default()
    }
}

/// Build a [`VStackElement`] with the given children.
pub fn vstack(children: Vec<WidgetElement>) -> VStackElement {
    VStackElement {
        children,
        ..Default::default()
    }
}

/// Build an [`HStackElement`] with the given children.
pub fn hstack(children: Vec<WidgetElement>) -> HStackElement {
    HStackElement {
        children,
        ..Default::default()
    }
}

fn default_columns() -> u32 {
    2
}
fn default_total() -> f64 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn option_none_omitted_from_json() {
        let cfg = WidgetConfig::small(TextElement {
            content: "hi".into(),
            font_size: Some(12.0),
            ..Default::default()
        });
        let json = serde_json::to_string(&cfg).unwrap();
        assert!(!json.contains("null"), "JSON must not contain null: {json}");
        assert!(!json.contains("\"medium\""));
        assert!(!json.contains("\"fontWeight\""));
        let back: WidgetConfig = serde_json::from_str(&json).unwrap();
        assert!(back.medium.is_none());
        match back.small.unwrap() {
            WidgetElement::Text(TextElement {
                font_weight,
                content,
                ..
            }) => {
                assert!(font_weight.is_none());
                assert_eq!(content, "hi");
            }
            other => panic!("expected text, got {other:?}"),
        }
    }

    #[test]
    fn ergonomic_builders_serialize() {
        let cfg = WidgetConfig::small(vstack(vec![text("72°")
            .font_size(36.0)
            .font_weight(FontWeight::Bold)
            .color("#fff")
            .into()]));
        let v = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v["small"]["type"], "vstack");
        assert_eq!(v["small"]["children"][0]["type"], "text");
        assert_eq!(v["small"]["children"][0]["content"], "72°");
        assert_eq!(v["small"]["children"][0]["fontSize"], 36.0);
        assert_eq!(v["small"]["children"][0]["fontWeight"], "bold");
        assert_eq!(v["small"]["children"][0]["color"], "#fff");
    }

    fn walk_json_files(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_json_files(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
                out.push(path);
            }
        }
    }

    /// Round-trip every fixture through `WidgetConfig` — catches wire drift after
    /// struct-variant refactors (tag + flattened fields must stay identical).
    #[test]
    fn wire_format_unchanged() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let mut files = Vec::new();
        walk_json_files(&root, &mut files);
        assert!(
            !files.is_empty(),
            "expected fixtures under {}",
            root.display()
        );
        for path in files {
            let raw = std::fs::read_to_string(&path).unwrap();
            let cfg: WidgetConfig = serde_json::from_str(&raw).unwrap_or_else(|e| {
                panic!("deserialize {}: {e}", path.display());
            });
            let encoded = serde_json::to_value(&cfg).unwrap();
            let again: WidgetConfig = serde_json::from_value(encoded.clone()).unwrap();
            assert_eq!(
                encoded,
                serde_json::to_value(&again).unwrap(),
                "wire drift in {}",
                path.display()
            );
            let out = serde_json::to_string(&cfg).unwrap();
            assert!(
                !out.contains("null"),
                "{} serialized with null: {out}",
                path.display()
            );
        }
    }
}
