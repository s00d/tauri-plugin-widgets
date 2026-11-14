//! Widget IR types: [`WidgetConfig`], [`WidgetElement`], and shared style values.
//!
//! Element structs, [`WidgetConfig`]/[`WidgetElement`], builders, and tests
//! live here; shared style values are in [`style`] and data payloads
//! ([`ChartDataPoint`], [`ListItem`], [`CanvasDrawCommand`]) are in [`data`].

mod activity;
mod data;
mod style;
pub use activity::*;
pub use data::*;
pub use style::*;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    /// Placement id for the built-in renderer: reads `config:{widgetId}#{instance}`
    /// first and fills `{{instance}}`. Defaults to none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    /// Size family the renderer should display: `"small"`, `"medium"`,
    /// or `"large"`.  Defaults to `"small"` when omitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Stacking layer. Unset: `top` when `alwaysOnTop`, else `desktop` on Linux (the
    /// historical DESKTOP pin) and `normal` on macOS / Windows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer: Option<WindowLayer>,
    /// Let clicks pass through to whatever is below. Default `false`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub click_through: Option<bool>,
    /// Whether the window can take keyboard focus. Default `true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focusable: Option<bool>,
    /// Show on every virtual desktop / Space. Default: `true` for the `desktop` layer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all_workspaces: Option<bool>,
    /// System window shadow (macOS / Windows). Default: platform default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow: Option<bool>,
    /// Backdrop material hint (macOS vibrancy, Windows 11 mica / acrylic). Needs a
    /// transparent window; ignored where the platform has no such material (Linux).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<WindowEffectHint>,
    /// Place relative to a monitor's work area instead of absolute `x` / `y`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement: Option<WidgetPlacement>,
    /// Remember where the user moved the window (per label) and restore it next time.
    /// Default `true`; a remembered position wins over `x` / `y` / `placement` while it
    /// is still on a connected monitor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remember_position: Option<bool>,
}

impl Default for WidgetWindowConfig {
    fn default() -> Self {
        Self {
            label: String::new(),
            url: None,
            width: 0.0,
            height: 0.0,
            x: None,
            y: None,
            always_on_top: false,
            skip_taskbar: true,
            group: None,
            widget_id: None,
            instance: None,
            size: None,
            layer: None,
            click_through: None,
            focusable: None,
            all_workspaces: None,
            shadow: None,
            effects: None,
            placement: None,
            remember_position: None,
        }
    }
}

/// Stacking layer of a desktop widget window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum WindowLayer {
    /// Ordinary window.
    Normal,
    /// Above other windows (always on top; Wayland layer-shell `Top`).
    Top,
    /// Below other windows but above the desktop (always on bottom; Wayland `Bottom`).
    Bottom,
    /// Part of the desktop: X11 `_NET_WM_WINDOW_TYPE_DESKTOP`, Wayland `Background`,
    /// macOS desktop window level. Windows: parented behind the desktop icons (WorkerW,
    /// or a Progman child on 24H2), re-attached after an Explorer restart.
    Desktop,
}

/// Backdrop material hint for [`WidgetWindowConfig::effects`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum WindowEffectHint {
    /// macOS HUD vibrancy; Windows acrylic.
    Vibrancy,
    /// Windows 11 mica; macOS under-window vibrancy.
    Mica,
    /// Windows acrylic; macOS HUD vibrancy.
    Acrylic,
    /// Windows blur; macOS under-window vibrancy.
    Blur,
}

/// Monitor-relative placement for [`WidgetWindowConfig::placement`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct WidgetPlacement {
    /// Monitor index (in `availableMonitors` order) or monitor name. Default: primary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monitor: Option<MonitorSelector>,
    /// Corner / edge of the work area to stick to. Default `top-left`.
    #[serde(default)]
    pub anchor: PlacementAnchor,
    /// Logical px inward from the anchored vertical edge (ignored for centered axes,
    /// where it shifts right).
    #[serde(default)]
    pub offset_x: f64,
    /// Logical px inward from the anchored horizontal edge (shifts down when centered).
    #[serde(default)]
    pub offset_y: f64,
}

/// Monitor by index or by name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum MonitorSelector {
    /// Index in the list of available monitors.
    Index(u32),
    /// Monitor name as reported by the OS.
    Name(String),
}

/// Work-area anchor for [`WidgetPlacement`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "kebab-case")]
pub enum PlacementAnchor {
    /// Top-left corner.
    #[default]
    TopLeft,
    /// Top edge, centered.
    Top,
    /// Top-right corner.
    TopRight,
    /// Left edge, centered.
    Left,
    /// Center.
    Center,
    /// Right edge, centered.
    Right,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom edge, centered.
    Bottom,
    /// Bottom-right corner.
    BottomRight,
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
    /// iOS / watchOS Lock Screen accessory families (circular, rectangular, inline).
    /// Falls back to `small`. Ignored elsewhere. Not a resolve family: `{{…}}`, `when`,
    /// `repeat` and `entries` do not apply to it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessory: Option<WidgetElement>,
    /// Scheduled states: from `at` on, an entry's layouts replace the base ones.
    /// The OS switches them while the app is closed (WidgetKit timeline, Android alarm,
    /// desktop timer). See [`crate::resolve`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<WidgetEntry>,
    /// Localized strings per locale (`"en"`, `"pt-BR"`), used as `{{t.key}}` in any text.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub strings: BTreeMap<String, BTreeMap<String, String>>,
    /// Locale used when the device locale has no `strings` table. Default: first table.
    #[serde(
        rename = "defaultLocale",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub default_locale: Option<String>,
    /// JSON the widget fetches by itself on a schedule; its fields are `{{data.path}}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<DataSource>,
    /// Default values for `{{data.path}}` (overridden by `setWidgetData` and `source`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<BTreeMap<String, serde_json::Value>>")
    )]
    pub data: Option<serde_json::Value>,
    /// Defaults of the widget's local state, read as `{{state.key}}` / `when` /
    /// `bind` / `repeat.each: "state.list"`. Taps on elements with `set` change it
    /// in-process (no app needed); `setWidgetState` changes it from the app. The stored
    /// state (`state:{widgetId}`, `state:{widgetId}#{instance}`) is merged over these per
    /// top-level key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<serde_json::Map<String, serde_json::Value>>")
    )]
    pub state: Option<serde_json::Map<String, serde_json::Value>>,
    /// How content changes between timeline entries / redraws (a hint):
    /// `numeric` animates digits, `opacity` cross-fades, `slide` pushes from the bottom.
    /// Apple and desktop animate; Android and Adaptive Cards ignore it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition: Option<Transition>,
    /// Reusable elements expanded by `{"type": "use", "template": "<key>"}`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub templates: BTreeMap<String, WidgetElement>,
    /// Minimum renderer contract; older renderers show `fallback` instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires: Option<Requires>,
    /// Layouts shown by renderers that do not meet `requires`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<FallbackLayouts>,
}

/// Renderer requirements of a config.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Requires {
    /// Render-time contract version ([`crate::resolve::CONTRACT_VERSION`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolve: Option<u64>,
}

/// Layouts for renderers below `requires`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct FallbackLayouts {
    /// Small layout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub small: Option<WidgetElement>,
    /// Medium layout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub medium: Option<WidgetElement>,
    /// Large layout.
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
            accessory: None,
            entries: Vec::new(),
            strings: BTreeMap::new(),
            default_locale: None,
            source: None,
            data: None,
            state: None,
            transition: None,
            templates: BTreeMap::new(),
            requires: None,
            fallback: None,
        }
    }
}

/// One scheduled state of a widget ([`WidgetConfig::entries`]).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct WidgetEntry {
    /// Unix epoch milliseconds from which this entry is shown.
    pub at: u64,
    /// Small layout from `at` on (base layout when omitted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub small: Option<WidgetElement>,
    /// Medium layout from `at` on (base layout when omitted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub medium: Option<WidgetElement>,
    /// Large layout from `at` on (base layout when omitted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub large: Option<WidgetElement>,
    /// Lock Screen accessory layout from `at` on (base layout when omitted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessory: Option<WidgetElement>,
}

/// Remote JSON the widget fetches without the app ([`WidgetConfig::source`]).
///
/// `url`, header values and `body` may use `{{instance}}` and `{{locale}}`, so each
/// placement of a widget can ask for its own data.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DataSource {
    /// `https://` URL returning a JSON object (at most 256 KiB).
    pub url: String,
    /// Minutes between fetches. Default 30; the OS may stretch it (minimum 15).
    #[serde(
        rename = "refreshMinutes",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub refresh_minutes: Option<u32>,
    /// `GET` (default) or `POST`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Extra request headers.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    /// Request body for `POST`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

impl WidgetConfig {
    /// Config with only the `small` size family set.
    pub fn small(el: impl Into<WidgetElement>) -> Self {
        Self {
            small: Some(el.into()),
            ..Default::default()
        }
    }

    /// Every layout root, scheduled entries included (for host-side passes such as
    /// normalization and image prefetch).
    pub fn layouts_mut(&mut self) -> impl Iterator<Item = &mut WidgetElement> {
        let fallback = self
            .fallback
            .as_mut()
            .map(|f| [&mut f.small, &mut f.medium, &mut f.large]);
        [
            &mut self.small,
            &mut self.medium,
            &mut self.large,
            &mut self.accessory,
        ]
        .into_iter()
        .chain(
            self.entries
                .iter_mut()
                .flat_map(|e| [&mut e.small, &mut e.medium, &mut e.large, &mut e.accessory]),
        )
        .chain(fallback.into_iter().flatten())
        .flatten()
        .chain(self.templates.values_mut())
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
    /// One `template` copy per item of a data array, spliced into the parent's children
    /// (resolved before rendering; see [`crate::resolve`]).
    #[serde(rename = "repeat")]
    Repeat(RepeatElement),
    /// A named entry of [`WidgetConfig::templates`] with arguments (resolved before
    /// rendering).
    #[serde(rename = "use")]
    Use(UseElement),
}

/// Repeat a template for every item of an array.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct RepeatElement {
    /// Path of the array: `data.tasks`, or a loop / template variable (`task.subtasks`).
    pub each: String,
    /// Name of the item inside the template (`{{task.title}}`). Default `item`.
    #[serde(rename = "as", default, skip_serializing_if = "Option::is_none")]
    pub as_name: Option<String>,
    /// At most this many items (hard cap 50).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Element drawn per item; `{{index}}` is its 0-based position.
    pub template: Box<WidgetElement>,
    #[serde(flatten)]
    /// Shared style (only `when` is used).
    pub style: ElementStyle,
}

/// Expand a named template.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct UseElement {
    /// Key in [`WidgetConfig::templates`].
    pub template: String,
    /// Arguments, read as `{{arg.name}}` / `bind: {"value": "arg.name"}`. A value that is
    /// exactly one `{{expr}}` keeps the expression's JSON type.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "BTreeMap<String, serde_json::Value>")
    )]
    pub args: BTreeMap<String, serde_json::Value>,
    #[serde(flatten)]
    /// Style fields here override the template root (instance wins).
    pub style: ElementStyle,
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
    /// Distribution of children along the stack axis (`spaceBetween`, …). Default `start`.
    #[serde(rename = "justify", default, skip_serializing_if = "Option::is_none")]
    pub justify: Option<Justify>,
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
    /// Distribution of children along the stack axis (`spaceBetween`, …). Default `start`.
    #[serde(rename = "justify", default, skip_serializing_if = "Option::is_none")]
    pub justify: Option<Justify>,
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
    /// Where text is cut when it does not fit. Default `tail`.
    #[serde(
        rename = "truncationMode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub truncation_mode: Option<TruncationMode>,
    /// Allow shrinking the font down to this factor (`0.5` = half) before truncating.
    #[serde(
        rename = "minimumScaleFactor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub minimum_scale_factor: Option<f64>,
    /// Line height in points.
    #[serde(
        rename = "lineHeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub line_height: Option<f64>,
    /// Extra space between characters in points.
    #[serde(
        rename = "letterSpacing",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub letter_spacing: Option<f64>,
    /// Underline the text.
    #[serde(rename = "underline", default, skip_serializing_if = "Option::is_none")]
    pub underline: Option<bool>,
    /// Strike the text through.
    #[serde(
        rename = "strikethrough",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub strikethrough: Option<bool>,
    /// Italic.
    #[serde(rename = "italic", default, skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    /// Equal-width digits (counters, prices) without a monospaced font.
    #[serde(
        rename = "monospacedDigit",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub monospaced_digit: Option<bool>,
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
    /// Symbol shown when `url` / `data` cannot be loaded.
    #[serde(
        rename = "fallbackSystemName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub fallback_system_name: Option<String>,
    /// SF Symbol rendering mode (`hierarchical`, `palette`, `multicolor`). Apple; others draw monochrome.
    #[serde(
        rename = "symbolRendering",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub symbol_rendering: Option<SymbolRendering>,
    /// SF Symbol stroke weight. Apple only.
    #[serde(
        rename = "symbolWeight",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub symbol_weight: Option<FontWeight>,
    /// Colors for `symbolRendering: palette` (primary, secondary, tertiary).
    #[serde(
        rename = "paletteColors",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub palette_colors: Vec<ColorValue>,
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
    #[serde(rename = "barStyle", default, skip_serializing_if = "Option::is_none")]
    pub bar_style: Option<ProgressStyle>,
    /// Unix ms: with `endAt`, the bar fills by itself between the two times (no refresh needed on Apple / desktop).
    /// WidgetKit animates it live only for spans up to 10 minutes (a live bar keeps Notification
    /// Center busy); a longer span shows the value at each timeline entry / reload.
    #[serde(rename = "startAt", default, skip_serializing_if = "Option::is_none")]
    pub start_at: Option<u64>,
    /// Unix ms the time-driven bar reaches full (see `startAt`).
    #[serde(rename = "endAt", default, skip_serializing_if = "Option::is_none")]
    pub end_at: Option<u64>,
    /// Color of the unfilled track.
    #[serde(
        rename = "trackColor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub track_color: Option<ColorValue>,
    /// Bar height (linear) or ring width (circular) in points.
    #[serde(rename = "thickness", default, skip_serializing_if = "Option::is_none")]
    pub thickness: Option<f64>,
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
            start_at: None,
            end_at: None,
            track_color: None,
            thickness: None,
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
    /// Unix ms: with `endAt`, the gauge moves by itself between the two times.
    #[serde(rename = "startAt", default, skip_serializing_if = "Option::is_none")]
    pub start_at: Option<u64>,
    /// Unix ms the time-driven gauge reaches `max`.
    #[serde(rename = "endAt", default, skip_serializing_if = "Option::is_none")]
    pub end_at: Option<u64>,
    /// Label at the minimum end.
    #[serde(rename = "minLabel", default, skip_serializing_if = "Option::is_none")]
    pub min_label: Option<String>,
    /// Label at the maximum end.
    #[serde(rename = "maxLabel", default, skip_serializing_if = "Option::is_none")]
    pub max_label: Option<String>,
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
    /// Local state written on tap, in-process (no app needed): `{"done": "$toggle"}`.
    /// `"$toggle"` flips a boolean (missing → `true`), `"$inc"` / `"$dec"` add / subtract 1
    /// (missing → 0 first), `null` removes the key (the default shows again), any other
    /// JSON is stored as is. With `action` too, the action is sent after the write.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<serde_json::Map<String, serde_json::Value>>")
    )]
    pub set: Option<serde_json::Map<String, serde_json::Value>>,
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
    /// Visual style (`bordered`, `prominent`, `plain`, `outline`). Unset keeps each
    /// platform's classic button look; set it for the same look everywhere.
    #[serde(rename = "variant", default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<ButtonVariant>,
    /// Icon (SF Symbol name; Material / emoji elsewhere) before the label.
    #[serde(
        rename = "systemName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub system_name: Option<String>,
    /// Draw dimmed and ignore taps.
    #[serde(rename = "disabled", default, skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
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
    /// Local state written on tap, in-process (no app needed): `{"done": "$toggle"}`.
    /// `"$toggle"` flips a boolean (missing → `true`), `"$inc"` / `"$dec"` add / subtract 1
    /// (missing → 0 first), `null` removes the key (the default shows again), any other
    /// JSON is stored as is. With `action` too, the action is sent after the write.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<serde_json::Map<String, serde_json::Value>>")
    )]
    pub set: Option<serde_json::Map<String, serde_json::Value>>,
    /// `switch` or `checkbox`. Unset keeps each platform's classic toggle (a glyph on
    /// Android); set it for the same control everywhere.
    #[serde(rename = "variant", default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<ToggleVariant>,
    /// Color when off.
    #[serde(
        rename = "uncheckedTint",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub unchecked_tint: Option<ColorValue>,
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
    /// Show the x axis labels (bar / line / area). Unset keeps each platform's classic
    /// chart (labels on Apple and desktop, none on Android); set it to match.
    #[serde(rename = "showXAxis", default, skip_serializing_if = "Option::is_none")]
    pub show_x_axis: Option<bool>,
    /// Show the y axis. Default `false`.
    #[serde(rename = "showYAxis", default, skip_serializing_if = "Option::is_none")]
    pub show_y_axis: Option<bool>,
    /// Show a legend of `series` (or pie slices). Default `false`.
    #[serde(
        rename = "showLegend",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub show_legend: Option<bool>,
    /// Lower bound of the y scale (default: data minimum or 0).
    #[serde(rename = "yMin", default, skip_serializing_if = "Option::is_none")]
    pub y_min: Option<f64>,
    /// Upper bound of the y scale (default: data maximum).
    #[serde(rename = "yMax", default, skip_serializing_if = "Option::is_none")]
    pub y_max: Option<f64>,
    /// Line / area curve: `linear` (default), `monotone`, `step`.
    #[serde(
        rename = "interpolation",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub interpolation: Option<ChartInterpolation>,
    /// Pie hole as a fraction of the radius (`0.6` = donut). Default `0`.
    #[serde(
        rename = "innerRadius",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub inner_radius: Option<f64>,
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
    /// Rich rows as nested elements (alternative to typed [`ListItem`]s).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<WidgetElement>,
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
    /// Local state written on tap, in-process (no app needed): `{"done": "$toggle"}`.
    /// `"$toggle"` flips a boolean (missing → `true`), `"$inc"` / `"$dec"` add / subtract 1
    /// (missing → 0 first), `null` removes the key (the default shows again), any other
    /// JSON is stored as is. With `action` too, the action is sent after the write.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "schema",
        schemars(with = "Option<serde_json::Map<String, serde_json::Value>>")
    )]
    pub set: Option<serde_json::Map<String, serde_json::Value>>,
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
    /// ISO-8601 start of a range timer (`startDate`…`targetDate`); counting up starts here.
    #[serde(rename = "startDate", default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    /// Always show hours (`0:05:00`). Default: only when needed.
    #[serde(rename = "showHours", default, skip_serializing_if = "Option::is_none")]
    pub show_hours: Option<bool>,
    /// Text shown once a countdown reached `targetDate` (default: `0:00`).
    #[serde(
        rename = "completedText",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub completed_text: Option<String>,
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

    /// Build → serialize → parse back: builders nest layouts, ergonomic setters land as
    /// camelCase fields, `None` is omitted (never `null`), parsing fills defaults (version,
    /// grid columns, window flags) and `layouts_mut` visits entry layouts too.
    #[test]
    fn build_serialize_parse_and_walk_layouts() {
        // 1. Builders.
        let c = WidgetConfig::small(text("s"))
            .with_medium(text("m"))
            .with_large(hstack(vec![text("l").into()]));
        assert!(c.small.is_some() && c.medium.is_some());
        match c.large {
            Some(WidgetElement::HStack(h)) => assert_eq!(h.children.len(), 1),
            other => panic!("{other:?}"),
        }
        let cfg = WidgetConfig::small(vstack(vec![text("72°")
            .font_size(36.0)
            .font_weight(FontWeight::Bold)
            .color("#fff")
            .into()]));
        let v = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v["small"]["type"], "vstack");
        let child = &v["small"]["children"][0];
        assert_eq!(child["type"], "text");
        assert_eq!(child["content"], "72°");
        assert_eq!(child["fontSize"], 36.0);
        assert_eq!(child["fontWeight"], "bold");
        assert_eq!(child["color"], "#fff");

        // 2. Unset options are omitted and stay unset after the round trip.
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

        // 3. Parse defaults.
        let parsed: WidgetConfig = serde_json::from_str(r#"{"small":{"type":"grid"}}"#).unwrap();
        assert_eq!(parsed.version, 1);
        match parsed.small {
            Some(WidgetElement::Grid(g)) => assert_eq!(g.columns, 2),
            other => panic!("{other:?}"),
        }
        let w: WidgetWindowConfig =
            serde_json::from_str(r#"{"label":"a","width":1,"height":1}"#).unwrap();
        assert!(w.skip_taskbar);
        assert!(!w.always_on_top);

        // 4. Layout walk includes timeline entries.
        let mut c: WidgetConfig = serde_json::from_value(serde_json::json!({
            "small": {"type": "spacer"},
            "entries": [{"at": 1, "medium": {"type": "spacer"}, "large": {"type": "spacer"}}]
        }))
        .unwrap();
        assert_eq!(c.layouts_mut().count(), 3);
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
