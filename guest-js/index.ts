/**
 * `tauri-plugin-widgets` — JavaScript/TypeScript API.
 *
 * Build native widgets on Android (AppWidget), iOS/macOS (WidgetKit) and
 * desktop (frameless Tauri windows) from a single JSON configuration.
 *
 * Two API layers:
 *
 * 1. **Widget Config API** — send a declarative UI config, rendered natively.
 * 2. **Data API** — raw key-value storage shared with native widget extensions.
 *
 * @example
 * ```ts
 * import { setWidgetConfig, setItems, reloadAllTimelines } from "tauri-plugin-widgets-api";
 *
 * // Declarative UI
 * await setWidgetConfig({
 *   small: {
 *     type: "vstack", padding: 12, background: "#1a1a2e",
 *     children: [
 *       { type: "text", content: "72°", fontSize: 36, fontWeight: "bold", color: "#fff" },
 *       { type: "progress", value: 0.7, tint: "#4CAF50" },
 *     ],
 *   },
 * }, "group.com.example.myapp");
 *
 * // Or raw data for custom native widgets
 * await setItems("temperature", "72", "group.com.example.myapp");
 * await reloadAllTimelines();
 * ```
 *
 * @module
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

const PLUGIN_ID = "plugin:widgets";

// ─── Data API ────────────────────────────────────────────────────────────────

/**
 * Store a key-value pair in the widget data store.
 *
 * The storage backend depends on the platform:
 * - **Android** — `SharedPreferences` scoped by `group`.
 * - **iOS / macOS** — JSON file in App Group shared container.
 * - **Desktop** — JSON file in the app data directory (macOS: shared container when available).
 *
 * @param key   - Setting name (e.g. `"temperature"`).
 * @param value - Setting value. Use `JSON.stringify()` for complex data.
 * @param group - Widget group identifier:
 *                Android: SharedPreferences name;
 *                iOS/macOS: App Group ID (e.g. `"group.com.example.myapp"`);
 *                Desktop: arbitrary string used as filename.
 */
export async function setItems(
  key: string,
  value: string,
  group: string,
): Promise<boolean> {
  if (!key) throw new Error("setItems: 'key' must not be empty");
  if (!group) throw new Error("setItems: 'group' must not be empty");
  return await invoke<boolean>(`${PLUGIN_ID}|set_items`, { key, value, group });
}

/**
 * Read a previously stored value from the widget data store.
 *
 * @param key   - The key to look up.
 * @param group - The widget group identifier.
 * @returns The stored string value, or `null` if the key was never set.
 */
export async function getItems(
  key: string,
  group: string,
): Promise<string | null> {
  if (!key) throw new Error("getItems: 'key' must not be empty");
  if (!group) throw new Error("getItems: 'group' must not be empty");
  return await invoke<string | null>(`${PLUGIN_ID}|get_items`, { key, group });
}

// ─── Reload API ──────────────────────────────────────────────────────────────

/**
 * Reload all registered widget timelines.
 *
 * - **Android** — sends `ACTION_APPWIDGET_UPDATE` broadcast for every registered provider.
 * - **iOS / macOS** — calls `WidgetCenter.shared.reloadAllTimelines()`.
 * - **Desktop** — emits a `"widget-reload"` Tauri event that widget windows can listen for.
 */
export async function reloadAllTimelines(): Promise<boolean> {
  return await invoke<boolean>(`${PLUGIN_ID}|reload_all_timelines`);
}

/**
 * Reload timelines for a specific widget kind or class.
 *
 * @param ofKind - Widget class name (Android: fully-qualified class, e.g.
 *                 `"com.example.app.MyWidget"`; iOS: widget kind string from
 *                 your `WidgetConfiguration`).
 */
export async function reloadTimelines(ofKind: string): Promise<boolean> {
  if (!ofKind) throw new Error("reloadTimelines: 'ofKind' must not be empty");
  return await invoke<boolean>(`${PLUGIN_ID}|reload_timelines`, { ofKind });
}

// ─── Registration & Pinning ─────────────────────────────────────────────────

/**
 * Register widget provider class names or kind strings.
 *
 * Must be called before `reloadAllTimelines()` or `requestWidget()`.
 *
 * @param widgets - Array of provider identifiers:
 *   - **Android**: fully-qualified class names (e.g. `"com.example.app.MyWidget"`)
 *   - **iOS / macOS**: widget kind strings from your `WidgetConfiguration`
 */
export async function setRegisterWidget(
  widgets: string[],
): Promise<boolean> {
  if (!widgets?.length) {
    throw new Error("setRegisterWidget: 'widgets' must be a non-empty array");
  }
  return await invoke<boolean>(`${PLUGIN_ID}|set_register_widget`, { widgets });
}

/**
 * Request the OS to pin (add) a widget to the home screen.
 *
 * - **Android** (API 26+) — calls `AppWidgetManager.requestPinAppWidget`.
 * - **iOS / macOS / Desktop** — no-op (users add widgets manually).
 */
export async function requestWidget(): Promise<boolean> {
  return await invoke<boolean>(`${PLUGIN_ID}|request_widget`);
}

// ─── Widget Window API (desktop only) ───────────────────────────────────────

/** Configuration for creating a desktop widget window. */
export interface WidgetWindowConfig {
  /** Unique window label (e.g. `"cpu-widget"`). Used to reference the window later. */
  label: string;
  /**
   * Frontend route to load.  When omitted (or empty) the plugin
   * serves its built-in widget renderer automatically — no extra
   * files needed.  Set to a path (e.g. `"/widget.html"`) only if
   * you want to use your own custom renderer.
   */
  url?: string;
  /** Window width in logical pixels. */
  width: number;
  /** Window height in logical pixels. */
  height: number;
  /** X position on screen (pixels from left). */
  x?: number;
  /** Y position on screen (pixels from top). */
  y?: number;
  /** Keep above all other windows. Default: `false`. */
  alwaysOnTop?: boolean;
  /** Hide from taskbar / dock. Default: `true`. */
  skipTaskbar?: boolean;
  /**
   * Widget group identifier.  Required when using the built-in
   * renderer — tells it which config to load via `getWidgetConfig`.
   * Ignored when a custom `url` is provided.
   */
  group?: string;
  /**
   * Size family the built-in renderer should display:
   * `"small"`, `"medium"`, or `"large"`.  Defaults to `"small"`.
   */
  size?: "small" | "medium" | "large";
}

/**
 * Create a frameless, transparent desktop widget window.
 *
 * When no `url` is specified the plugin serves its built-in renderer
 * automatically — it reads the config via `getWidgetConfig(group)` and
 * renders it using vanilla HTML/CSS/SVG.  The window has a drag handle
 * at the top and a close button that appears on hover.
 *
 * You may provide your own `url` (e.g. `"/my-widget.html"`) if you
 * need full control over the rendering.
 *
 * **Desktop only.** Returns an error on mobile.
 *
 * @example
 * ```ts
 * // Built-in renderer (recommended):
 * await createWidgetWindow({
 *   label: "weather",
 *   width: 280,
 *   height: 200,
 *   group: "group.com.example.myapp",
 *   size: "small",
 * });
 *
 * // Custom renderer:
 * await createWidgetWindow({
 *   label: "weather",
 *   url: "/my-widget.html",
 *   width: 280,
 *   height: 200,
 * });
 * ```
 */
export async function createWidgetWindow(
  config: WidgetWindowConfig,
): Promise<boolean> {
  if (!config.label) throw new Error("createWidgetWindow: 'label' is required");
  return await invoke<boolean>(`${PLUGIN_ID}|create_widget_window`, { config });
}

/**
 * Close a previously created desktop widget window.
 *
 * @param label - The window label used when creating it.
 *
 * **Desktop only.**
 */
export async function closeWidgetWindow(label: string): Promise<boolean> {
  if (!label) throw new Error("closeWidgetWindow: 'label' is required");
  return await invoke<boolean>(`${PLUGIN_ID}|close_widget_window`, { label });
}

// ─── Widget Config API ──────────────────────────────────────────────────────

// ── Enums ──

/**
 * Font weight for text elements.
 * Maps to SwiftUI `Font.Weight` / CSS `font-weight`.
 */
export type FontWeight = "ultralight" | "thin" | "light" | "regular" | "medium" | "semibold" | "bold" | "heavy" | "black";

/** Font design style. Maps to SwiftUI `Font.Design`. */
export type FontDesign = "default" | "monospaced" | "rounded" | "serif";

/** Text horizontal alignment. */
export type TextAlignment = "leading" | "center" | "trailing";

/** Horizontal alignment for VStack children. */
export type HorizontalAlignment = "leading" | "center" | "trailing";

/** Vertical alignment for HStack children. */
export type VerticalAlignment = "top" | "center" | "bottom";

/** How an image fills its frame. */
export type ContentMode = "fit" | "fill";

/** Progress bar visual style. */
export type ProgressStyle = "linear" | "circular";

/** Gauge visual style. */
export type GaugeStyle = "circular" | "linear";

/**
 * Date display style.
 * - `time` — shows time only (e.g. `"10:00 AM"`)
 * - `date` — shows date only (e.g. `"Mar 1, 2026"`)
 * - `relative` — relative to now (e.g. `"in 2 days"`)
 * - `offset` — offset from now (e.g. `"+2 days"`)
 * - `timer` — countdown/up timer (e.g. `"48:00:00"`)
 */
export type DateStyle = "time" | "date" | "relative" | "offset" | "timer";

/** Chart visualization type. */
export type ChartType = "bar" | "line" | "area" | "pie";

/** Shape type for the `shape` element. */
export type ShapeType = "circle" | "capsule" | "rectangle";

/** Timer counting direction. */
export type TimerCounting = "up" | "down";

/** Clip shape for content masking (e.g. circular avatar). */
export type ClipShape = "circle" | "capsule" | "rectangle";

/**
 * Semantic text style — respects Dynamic Type / accessibility settings.
 * Overrides `fontSize` when set. Uses platform-native scaling.
 */
export type TextStyle =
  | "largeTitle" | "title" | "title2" | "title3"
  | "headline" | "subheadline"
  | "body" | "callout"
  | "footnote" | "caption" | "caption2";

/** Gradient type for background gradients. */
export type GradientType = "linear" | "radial" | "angular";

/** Direction for linear gradients. */
export type GradientDirection =
  | "topToBottom"
  | "bottomToTop"
  | "leadingToTrailing"
  | "trailingToLeading"
  | "topLeadingToBottomTrailing"
  | "topTrailingToBottomLeading";

// ── Supporting types ──

/**
 * Color value — hex string, semantic name, or adaptive `{ light, dark }` pair.
 *
 * Semantic names (auto-adapt to dark mode):
 * `"label"`, `"secondaryLabel"`, `"systemBackground"`,
 * `"secondarySystemBackground"`, `"accent"`, `"separator"`
 *
 * @example
 * ```ts
 * // Hex string
 * color: "#FF5733"
 *
 * // Semantic (auto-adapts to dark mode)
 * color: "label"
 *
 * // Adaptive pair
 * color: { light: "#000000", dark: "#FFFFFF" }
 * ```
 */
export type ColorValue = string | { light: string; dark: string };

/** A single data point in a chart. */
export interface ChartDataPoint {
  /** X-axis label. */
  label: string;
  /** Numeric value. */
  value: number;
  /** Override color for this data point. Falls back to chart `tint`. */
  color?: ColorValue;
}

/**
 * Frame size constraints.
 * Use `"infinity"` for `maxWidth`/`maxHeight` to fill available space.
 */
export interface FrameConfig {
  /** Fixed width in points. */
  width?: number;
  /** Fixed height in points. */
  height?: number;
  /** Maximum width. Use `"infinity"` to expand. */
  maxWidth?: number | "infinity";
  /** Maximum height. Use `"infinity"` to expand. */
  maxHeight?: number | "infinity";
}

/** Border configuration. */
export interface BorderConfig {
  /** Border color (hex). */
  color: string;
  /** Border width in points. Default: `1`. */
  width?: number;
}

/** Gradient background configuration. */
export interface GradientConfig {
  /** Gradient type. */
  gradientType: GradientType;
  /** Array of hex color stops. */
  colors: string[];
  /** Direction for linear gradients. */
  direction?: GradientDirection;
}

/** Shadow configuration. */
export interface ShadowConfig {
  /** Shadow color (hex). */
  color?: string;
  /** Blur radius in points. */
  radius?: number;
  /** Horizontal offset. */
  x?: number;
  /** Vertical offset. */
  y?: number;
}

/**
 * Background value — solid hex, adaptive `{ light, dark }`, or gradient config.
 *
 * @example
 * ```ts
 * // Solid color
 * background: "#1a1a2e"
 *
 * // Adaptive (auto dark mode)
 * background: { light: "#FFFFFF", dark: "#1a1a2e" }
 *
 * // Linear gradient
 * background: { gradientType: "linear", colors: ["#ff0000", "#0000ff"], direction: "topToBottom" }
 * ```
 */
export type BackgroundValue = string | GradientConfig | { light: string; dark: string };

/**
 * Padding value — either uniform (a single number) or per-edge.
 *
 * @example
 * ```ts
 * // Uniform: 12pt on all sides
 * padding: 12
 *
 * // Per-edge
 * padding: { top: 8, bottom: 16, leading: 12, trailing: 12 }
 * ```
 */
export type PaddingValue = number | {
  top?: number;
  bottom?: number;
  leading?: number;
  trailing?: number;
};

/**
 * Common style properties applicable to any element (except `spacer`).
 *
 * These are flattened into the element JSON — just add them alongside
 * the element-specific properties.
 */
export interface ElementStyle {
  /** Padding inside the element. */
  padding?: PaddingValue;
  /** Background — solid hex color, adaptive pair, or gradient config. */
  background?: BackgroundValue;
  /** Corner radius in points. */
  cornerRadius?: number;
  /** Opacity from 0.0 (transparent) to 1.0 (opaque). */
  opacity?: number;
  /** Frame size constraints. */
  frame?: FrameConfig;
  /** Border around the element. */
  border?: BorderConfig;
  /** Drop shadow. */
  shadow?: ShadowConfig;
  /** Clip content to a shape (e.g. `"circle"` for round avatar). */
  clipShape?: ClipShape;
  /**
   * Layout weight for flexible sizing inside stacks.
   * Elements with `flex > 0` expand to fill available space.
   * Maps to SwiftUI `layoutPriority` / Android `layout_weight`.
   */
  flex?: number;
}

// ── Element interfaces ──

/**
 * Vertical stack — arranges children top-to-bottom.
 *
 * @example
 * ```json
 * { "type": "vstack", "spacing": 8, "alignment": "leading", "children": [...] }
 * ```
 */
export interface VStackElement extends ElementStyle {
  type: "vstack";
  children: WidgetElement[];
  /** Space between children (points). */
  spacing?: number;
  /** Horizontal alignment of children. */
  alignment?: HorizontalAlignment;
}

/**
 * Horizontal stack — arranges children left-to-right.
 *
 * @example
 * ```json
 * { "type": "hstack", "spacing": 10, "alignment": "center", "children": [...] }
 * ```
 */
export interface HStackElement extends ElementStyle {
  type: "hstack";
  children: WidgetElement[];
  /** Space between children (points). */
  spacing?: number;
  /** Vertical alignment of children. */
  alignment?: VerticalAlignment;
}

/** Overlay stack — layers children on top of each other. */
export interface ZStackElement extends ElementStyle {
  type: "zstack";
  children: WidgetElement[];
  /** Alignment within the stack. */
  alignment?: string;
}

/**
 * Grid layout — arranges children in a grid with configurable columns.
 *
 * @example
 * ```json
 * { "type": "grid", "columns": 2, "spacing": 8, "rowSpacing": 8, "children": [...] }
 * ```
 */
export interface GridElement extends ElementStyle {
  type: "grid";
  children: WidgetElement[];
  /** Number of columns. Default: `2`. */
  columns?: number;
  /** Column spacing (points). */
  spacing?: number;
  /** Row spacing (points). */
  rowSpacing?: number;
}

/**
 * Container (Box) — wraps children with alignment and styling.
 *
 * Use for cards, badges, overlays, or any case where you need a
 * styled wrapper around a single element with precise alignment.
 *
 * @example
 * ```json
 * {
 *   "type": "container",
 *   "contentAlignment": "center",
 *   "background": "#1a1a2e",
 *   "cornerRadius": 12,
 *   "padding": 16,
 *   "children": [
 *     { "type": "text", "content": "Centered!", "color": "#fff" }
 *   ]
 * }
 * ```
 */
export interface ContainerElement extends ElementStyle {
  type: "container";
  /** Child elements rendered inside container. */
  children?: WidgetElement[];
  /**
   * Content alignment within the container.
   * Values: `"center"`, `"topLeading"`, `"top"`, `"topTrailing"`,
   * `"leading"`, `"trailing"`, `"bottomLeading"`, `"bottom"`, `"bottomTrailing"`.
   */
  contentAlignment?: string;
}

/**
 * Text element — displays a string with configurable typography.
 *
 * @example
 * ```json
 * { "type": "text", "content": "Hello", "fontSize": 24, "fontWeight": "bold", "color": "#fff" }
 * ```
 */
export interface TextElement extends ElementStyle {
  type: "text";
  /** The text string to display. */
  content: string;
  /**
   * Font size in points. Ignored when `textStyle` is set.
   */
  fontSize?: number;
  /** Font weight. */
  fontWeight?: FontWeight;
  /** Font design style. */
  fontDesign?: FontDesign;
  /**
   * Semantic text style — uses Dynamic Type (Apple) or sp scaling (Android).
   * When set, overrides `fontSize` for platform-appropriate sizing.
   *
   * @example `"headline"`, `"body"`, `"caption"`, `"title"`
   */
  textStyle?: TextStyle;
  /** Text color — hex, semantic name, or adaptive pair. */
  color?: ColorValue;
  /** Text alignment. */
  alignment?: TextAlignment;
  /** Maximum number of lines. Text is truncated beyond this. */
  lineLimit?: number;
}

/**
 * Image element — displays an SF Symbol, base64 data, or remote URL.
 *
 * @example
 * ```json
 * { "type": "image", "systemName": "cloud.sun.fill", "size": 32, "color": "#ffcc00" }
 * ```
 */
export interface ImageElement extends ElementStyle {
  type: "image";
  /** SF Symbol name (Apple) or icon hint (Android). */
  systemName?: string;
  /** Base64-encoded image data. */
  data?: string;
  /** Remote image URL. */
  url?: string;
  /** Display size in points. */
  size?: number;
  /** Tint color — hex, semantic, or adaptive. */
  color?: ColorValue;
  /** How the image fills its frame. */
  contentMode?: ContentMode;
}

/**
 * Progress bar — shows completion as a linear or circular indicator.
 *
 * @example
 * ```json
 * { "type": "progress", "value": 0.7, "total": 1.0, "tint": "#4CAF50", "label": "Steps" }
 * ```
 */
export interface ProgressElement extends ElementStyle {
  type: "progress";
  /** Current progress value. */
  value: number;
  /** Maximum value. Default: `1.0`. */
  total?: number;
  /** Label text shown alongside the progress bar. */
  label?: string;
  /** Bar color — hex, semantic, or adaptive. */
  tint?: ColorValue;
  /** Text color for label — hex, semantic, or adaptive. */
  color?: ColorValue;
  /** Visual style: `"linear"` (default) or `"circular"`. */
  barStyle?: ProgressStyle;
}

/**
 * Gauge — circular or linear meter showing a value within a range.
 *
 * @example
 * ```json
 * {
 *   "type": "gauge", "value": 0.72,
 *   "min": 0, "max": 1,
 *   "label": "CPU", "currentValueLabel": "72%",
 *   "tint": "#7aa2f7", "gaugeStyle": "circular"
 * }
 * ```
 */
export interface GaugeElement extends ElementStyle {
  type: "gauge";
  /** Current value. */
  value: number;
  /** Minimum value. Default: `0`. */
  min?: number;
  /** Maximum value. Default: `1`. */
  max?: number;
  /** Caption below the gauge. */
  label?: string;
  /** Value label displayed inside the gauge. */
  currentValueLabel?: string;
  /** Gauge color — hex, semantic, or adaptive. */
  tint?: ColorValue;
  /** Text color for value/label — hex, semantic, or adaptive. */
  color?: ColorValue;
  /** Visual style: `"circular"` or `"linear"`. */
  gaugeStyle?: GaugeStyle;
}

/**
 * Button — tappable element that opens a URL or triggers an action event.
 *
 * Two modes:
 * - **`url`** — opens a deep-link URL when tapped (e.g. `"myapp://home"`).
 * - **`action`** — emits a `widget-action` Tauri event with the action name
 *   as payload, so the main app can react (e.g. refresh data, navigate, etc.).
 *
 * At least one of `url` or `action` should be set. If both are set, `action`
 * takes priority on desktop; on native widgets both are encoded into the URL.
 *
 * @example
 * ```json
 * { "type": "button", "label": "Refresh", "action": "refresh_data", "backgroundColor": "#2196F3", "color": "#fff" }
 * ```
 */
export interface ButtonElement extends ElementStyle {
  type: "button";
  /** Button text. */
  label: string;
  /** Deep-link URL to open when tapped. */
  url?: string;
  /** Action identifier — emits a `widget-action` event when tapped. */
  action?: string;
  /** Text color — hex, semantic, or adaptive. */
  color?: ColorValue;
  /** Background color — hex, semantic, or adaptive. */
  backgroundColor?: ColorValue;
  /** Font size in points. */
  fontSize?: number;
  /** Button label alignment. */
  textAlignment?: TextAlignment;
}

/**
 * Toggle — visual on/off indicator (read-only in widgets).
 *
 * @example
 * ```json
 * { "type": "toggle", "isOn": true, "label": "Dark Mode", "tint": "#4CAF50" }
 * ```
 */
export interface ToggleElement extends ElementStyle {
  type: "toggle";
  /** Whether the toggle is on. */
  isOn: boolean;
  /** Label text next to the toggle. */
  label?: string;
  /** Toggle tint color when on — hex, semantic, or adaptive. */
  tint?: ColorValue;
  /** Label text color — hex, semantic, or adaptive. */
  color?: ColorValue;
  /** Action identifier sent back to the app (for future interactivity). */
  action?: string;
}

/**
 * Divider — a horizontal line separating content.
 *
 * @example
 * ```json
 * { "type": "divider", "color": "#333333", "thickness": 1 }
 * ```
 */
export interface DividerElement extends ElementStyle {
  type: "divider";
  /** Line color — hex, semantic, or adaptive. */
  color?: ColorValue;
  /** Line thickness in points. */
  thickness?: number;
}

/**
 * Spacer — flexible space that pushes siblings apart within a stack.
 *
 * @example
 * ```json
 * { "type": "spacer", "minLength": 10 }
 * ```
 */
export interface SpacerElement {
  type: "spacer";
  /** Minimum space in points. */
  minLength?: number;
}

/**
 * Date element — displays a date/time with live formatting.
 *
 * @example
 * ```json
 * { "type": "date", "date": "2026-03-01T10:00:00Z", "dateStyle": "relative", "color": "#fff" }
 * ```
 */
export interface DateElement extends ElementStyle {
  type: "date";
  /** ISO 8601 date string. */
  date: string;
  /** How to format the date. */
  dateStyle?: DateStyle;
  /** Font size in points. */
  fontSize?: number;
  /** Text color — hex, semantic, or adaptive. */
  color?: ColorValue;
}

/**
 * Chart element — bar or line chart from data points.
 *
 * @example
 * ```json
 * {
 *   "type": "chart", "chartType": "bar", "tint": "#89b4fa",
 *   "chartData": [
 *     { "label": "Mon", "value": 120 },
 *     { "label": "Tue", "value": 180, "color": "#a6e3a1" }
 *   ]
 * }
 * ```
 */
export interface ChartElement extends ElementStyle {
  type: "chart";
  /** Chart visualization type. */
  chartType: ChartType;
  /** Array of data points. */
  chartData: ChartDataPoint[];
  /** Default color for bars/lines — hex, semantic, or adaptive. */
  tint?: ColorValue;
}

/** Android list item for `list` widgets. */
export interface ListItem {
  /** Row label text. */
  text: string;
  /** Optional checked state marker. */
  checked?: boolean;
  /** Optional action emitted on row tap. */
  action?: string;
  /** Optional payload forwarded with row action. */
  payload?: string;
}

/**
 * Android-only collection list backed by `RemoteViewsService`.
 *
 * Use this for larger lists to avoid binder payload limits of deeply nested layouts.
 */
export interface ListElement extends ElementStyle {
  type: "list";
  items: ListItem[];
  /** Space between rows (points). */
  spacing?: number;
  /** Row font size (points). */
  fontSize?: number;
  /** Row text color. */
  color?: ColorValue;
}

// ── Canvas drawing commands ──

/** Circle draw command. */
export interface CanvasCircle {
  draw: "circle";
  cx: number; cy: number; r: number;
  fill?: ColorValue; stroke?: ColorValue; strokeWidth?: number;
}
/** Line draw command. */
export interface CanvasLine {
  draw: "line";
  x1: number; y1: number; x2: number; y2: number;
  stroke?: ColorValue; strokeWidth?: number; lineCap?: "butt" | "round" | "square";
}
/** Rectangle draw command. */
export interface CanvasRect {
  draw: "rect";
  x: number; y: number; width: number; height: number;
  fill?: ColorValue; stroke?: ColorValue; strokeWidth?: number; cornerRadius?: number;
}
/** Arc draw command (angles in degrees). */
export interface CanvasArc {
  draw: "arc";
  cx: number; cy: number; r: number;
  startAngle: number; endAngle: number;
  fill?: ColorValue; stroke?: ColorValue; strokeWidth?: number;
}
/** Text draw command. */
export interface CanvasText {
  draw: "text";
  x: number; y: number; content: string;
  fontSize?: number; color?: ColorValue; anchor?: "start" | "middle" | "end";
}
/** SVG path draw command. */
export interface CanvasPath {
  draw: "path";
  d: string;
  fill?: ColorValue; stroke?: ColorValue; strokeWidth?: number;
}

/** Union of all canvas drawing commands. */
export type CanvasDrawCommand = CanvasCircle | CanvasLine | CanvasRect | CanvasArc | CanvasText | CanvasPath;

/**
 * Canvas element — draw arbitrary shapes via declarative JSON commands.
 *
 * The app computes all coordinates and passes them in the config.
 * The widget renders them using platform-native drawing:
 * - **iOS/macOS**: SwiftUI `Canvas` / `Path`
 * - **Android**: `Bitmap` + `android.graphics.Canvas`
 * - **Desktop**: SVG
 *
 * @example
 * ```json
 * {
 *   "type": "canvas", "width": 120, "height": 120,
 *   "elements": [
 *     { "draw": "circle", "cx": 60, "cy": 60, "r": 55, "fill": "#1a1a2e", "stroke": "#fff", "strokeWidth": 2 },
 *     { "draw": "line", "x1": 60, "y1": 60, "x2": 60, "y2": 20, "stroke": "#fff", "strokeWidth": 3 },
 *     { "draw": "circle", "cx": 60, "cy": 60, "r": 4, "fill": "#e94560" }
 *   ]
 * }
 * ```
 */
export interface CanvasElement extends ElementStyle {
  type: "canvas";
  /** Canvas width in points. */
  width: number;
  /** Canvas height in points. */
  height: number;
  /** Array of draw commands. */
  elements: CanvasDrawCommand[];
}

/**
 * Tappable wrapper — makes nested content clickable.
 *
 * @example
 * ```json
 * {
 *   "type": "link", "action": "open_detail",
 *   "children": [
 *     { "type": "hstack", "spacing": 8, "children": [
 *       { "type": "image", "systemName": "star.fill", "color": "#ffcc00", "size": 20 },
 *       { "type": "text", "content": "Tap me!", "fontSize": 16, "color": "#fff" }
 *     ]}
 *   ]
 * }
 * ```
 */
export interface LinkElement extends ElementStyle {
  type: "link";
  children: WidgetElement[];
  /** Deep-link URL to open. */
  url?: string;
  /** Action identifier — emits `widget-action` event. */
  action?: string;
}

/**
 * Colored shape — circle, capsule, or rounded rectangle.
 *
 * @example
 * ```json
 * { "type": "shape", "shapeType": "circle", "fill": "#ff0000", "size": 12 }
 * ```
 */
export interface ShapeElement extends ElementStyle {
  type: "shape";
  /** Shape kind. */
  shapeType: ShapeType;
  /** Fill color — hex, semantic, or adaptive. */
  fill?: ColorValue;
  /** Stroke color — hex, semantic, or adaptive. */
  stroke?: ColorValue;
  /** Stroke width in points. */
  strokeWidth?: number;
  /** Size in points (width & height). */
  size?: number;
}

/**
 * Live countdown/countup timer that updates in real-time without timeline refresh.
 *
 * - **iOS/macOS**: `Text(date, style: .timer)`
 * - **Android**: `Chronometer`
 * - **Desktop**: JavaScript `setInterval`
 *
 * @example
 * ```json
 * { "type": "timer", "targetDate": "2026-04-06T00:00:00Z", "counting": "down", "fontSize": 24, "color": "#fff" }
 * ```
 */
export interface TimerElement extends ElementStyle {
  type: "timer";
  /** ISO 8601 target date. */
  targetDate: string;
  /** Count direction. Default: `"down"`. */
  counting?: TimerCounting;
  /** Font size in points. */
  fontSize?: number;
  /** Font weight. */
  fontWeight?: FontWeight;
  /** Text color — hex, semantic, or adaptive. */
  color?: ColorValue;
}

/**
 * Convenience element: icon + text combined.
 *
 * @example
 * ```json
 * { "type": "label", "text": "Favorites", "systemName": "star.fill", "iconColor": "#ffcc00", "fontSize": 14 }
 * ```
 */
export interface LabelElement extends ElementStyle {
  type: "label";
  /** Text string. */
  text: string;
  /** SF Symbol name (Apple) or icon hint (Android). */
  systemName: string;
  /** Icon tint color — hex, semantic, or adaptive. */
  iconColor?: ColorValue;
  /** Font size in points. */
  fontSize?: number;
  /** Font weight. */
  fontWeight?: FontWeight;
  /** Text color — hex, semantic, or adaptive. */
  color?: ColorValue;
  /** Space between icon and text (points). */
  spacing?: number;
}

/**
 * Union of all widget element types.
 *
 * Elements are differentiated by the `type` field. Layout containers
 * (`vstack`, `hstack`, `zstack`, `grid`, `link`) contain `children` arrays.
 * Leaf elements render specific content.
 */
export type WidgetElement =
  | VStackElement
  | HStackElement
  | ZStackElement
  | GridElement
  | ContainerElement
  | TextElement
  | ImageElement
  | ProgressElement
  | GaugeElement
  | ButtonElement
  | ToggleElement
  | DividerElement
  | SpacerElement
  | DateElement
  | ChartElement
  | ListElement
  | LinkElement
  | ShapeElement
  | TimerElement
  | LabelElement
  | CanvasElement;

/**
 * Widget configuration with layouts per size family.
 *
 * Each property defines the root element for that widget size.
 * The native widget picks the layout matching its display size.
 * Unset sizes fall back in order: `small` → `medium` → `large`.
 *
 * @example
 * ```ts
 * const config: WidgetConfig = {
 *   small: {
 *     type: "vstack", padding: 12, background: "#1a1a2e",
 *     children: [
 *       { type: "text", content: "Hello!", fontSize: 24, color: "#fff" },
 *     ],
 *   },
 *   medium: { ... },
 *   large: { ... },
 * };
 * ```
 */
export interface WidgetConfig {
  /** Schema version (default: `1`). Reserved for future changes. */
  version?: number;
  /** Root element for small widgets (e.g. 2x2 on iOS). */
  small?: WidgetElement;
  /** Root element for medium widgets (e.g. 4x2 on iOS). */
  medium?: WidgetElement;
  /** Root element for large widgets (e.g. 4x4 on iOS). */
  large?: WidgetElement;
}

/**
 * Send a declarative UI configuration to native widgets.
 *
 * The JSON config is stored in the widget data store and read by the native
 * widget extension, which renders it using platform-native components:
 * - **iOS / macOS** — SwiftUI views (`VStack`, `Text`, `Gauge`, etc.)
 * - **Android** — `RemoteViews` (`LinearLayout`, `TextView`, etc.)
 * - **Desktop** — HTML/CSS in the widget window
 *
 * After setting the config, all widget timelines are automatically reloaded.
 *
 * @param config - The widget UI configuration.
 * @param group  - Widget group identifier (same as `setItems` group).
 *
 * @example
 * ```ts
 * await setWidgetConfig({
 *   small: {
 *     type: "vstack",
 *     padding: 12,
 *     background: "#1a1a2e",
 *     children: [
 *       { type: "text", content: "72°", fontSize: 36, fontWeight: "bold", color: "#fff" },
 *       { type: "progress", value: 0.7, tint: "#4CAF50" },
 *     ],
 *   },
 * }, "group.com.example.myapp");
 * ```
 */
export async function setWidgetConfig(
  config: WidgetConfig,
  group: string,
  /** Skip native widget reload (WidgetKit / AppWidgetManager).
   *  Desktop widget windows are always updated instantly via eval push. */
  skipReload = false,
): Promise<boolean> {
  if (!group) throw new Error("setWidgetConfig: 'group' must not be empty");
  return await invoke<boolean>(`${PLUGIN_ID}|set_widget_config`, {
    config, group, skipReload,
  });
}

/**
 * Read the current widget UI configuration from the data store.
 *
 * @param group - Widget group identifier.
 * @returns The current `WidgetConfig`, or `null` if none has been set.
 */
export async function getWidgetConfig(
  group: string,
): Promise<WidgetConfig | null> {
  if (!group) throw new Error("getWidgetConfig: 'group' must not be empty");
  return await invoke<WidgetConfig | null>(`${PLUGIN_ID}|get_widget_config`, { group });
}

// ─── Widget Action API ──────────────────────────────────────────────────────

/** Payload delivered by the `widget-action` event. */
export interface WidgetActionPayload {
  /** The action identifier from the button config. */
  action: string;
  /** Optional extra data (e.g. serialized JSON from the widget). */
  payload?: string;
}

/**
 * Trigger a `widget-action` event via the Rust plugin.
 *
 * The Rust command broadcasts the event through `app.emit()`, so every
 * window that called `onWidgetAction()` (or `listen("widget-action", …)`)
 * will receive it — including the main application window.
 *
 * @param action  - Action identifier (matches `ButtonElement.action`).
 * @param payload - Optional extra string payload.
 */
export async function widgetAction(
  action: string,
  payload?: string,
): Promise<boolean> {
  if (!action) throw new Error("widgetAction: 'action' must not be empty");
  return await invoke<boolean>(`${PLUGIN_ID}|widget_action`, { action, payload });
}

/**
 * Listen for `widget-action` events emitted by widget buttons.
 *
 * @param callback - Called with the action payload each time a widget button
 *                   with an `action` field is tapped.
 * @returns A function to stop listening.
 *
 * @example
 * ```ts
 * import { onWidgetAction } from "tauri-plugin-widgets-api";
 *
 * const unlisten = await onWidgetAction((data) => {
 *   console.log("Widget action:", data.action, data.payload);
 *   if (data.action === "refresh_data") {
 *     fetchLatestData();
 *   }
 * });
 * ```
 */
export async function onWidgetAction(
  callback: (data: WidgetActionPayload) => void,
): Promise<UnlistenFn> {
  return await listen<WidgetActionPayload>("widget-action", (event) => {
    callback(event.payload);
  });
}

/** Poll pending widget actions queued by native side (Android fallback path). */
export async function pollPendingWidgetActions(
  group: string,
): Promise<WidgetActionPayload[]> {
  let res: unknown;
  try {
    res = await invoke<unknown>(`${PLUGIN_ID}|poll_pending_actions`, { group });
  } catch (e) {
    const msg = String(e ?? "");
    // iOS/macOS builds may not expose this Android-specific command path.
    if (msg.includes("No command pollPendingActions") || msg.includes("pollPendingActions")) {
      return [];
    }
    throw e;
  }
  const raw = Array.isArray(res)
    ? res
    : ((res as { results?: unknown } | null | undefined)?.results ?? null);
  if (!Array.isArray(raw)) return [];
  const parsed: Array<WidgetActionPayload | null> = raw.map((item) => {
    if (!item || typeof item !== "object") return null;
    const obj = item as { action?: unknown; payload?: unknown };
    if (typeof obj.action !== "string" || obj.action.length === 0) return null;
    return {
      action: obj.action,
      ...(typeof obj.payload === "string" ? { payload: obj.payload } : {}),
    };
  });
  return parsed.filter((v): v is WidgetActionPayload => v !== null);
}

// ─── Widget Updater ─────────────────────────────────────────────────────────

/**
 * Start a periodic widget updater that calls `builder` on a fixed interval,
 * sends the returned config to the native widget, and optionally reloads
 * widget timelines.
 *
 * This is a convenience wrapper around `setInterval` + `setWidgetConfig` +
 * `reloadAllTimelines` so you don't have to write boilerplate timers.
 *
 * **Important (iOS/macOS):** Apple enforces a daily widget reload budget
 * (~40-70 reloads/day). The Rust backend throttles `reloadAllTimelines()`
 * to at most once every 15 minutes. For frequent data updates, set
 * `reload: false` (the default) — desktop webview widgets update instantly
 * via events without counting against the budget.
 *
 * @param builder    - Called every `intervalMs`. Return the new `WidgetConfig`.
 *                     May be async.
 * @param group      - App Group / SharedPreferences group identifier.
 * @param options    - Optional settings.
 * @param options.intervalMs  - Update interval in milliseconds (default `1000`).
 *                              For native mobile widgets, consider using a
 *                              longer interval (60000+) to avoid wasted work.
 * @param options.immediate   - If `true`, run the builder immediately before
 *                              the first interval tick (default `true`).
 * @param options.reload      - If `true`, call `reloadAllTimelines()` after
 *                              each update so native widgets refresh
 *                              (default `false`). Set to `true` for native
 *                              widgets; leave `false` for desktop-only.
 *                              Note: backend throttles this to once per 15 min
 *                              on iOS/macOS to respect the budget.
 * @returns A `stop()` function. Call it to clear the timer.
 *
 * @example
 * ```ts
 * import { startWidgetUpdater } from "tauri-plugin-widgets-api";
 *
 * const stop = await startWidgetUpdater(
 *   () => ({
 *     small: {
 *       type: "vstack", padding: 12, background: "#1a1a2e",
 *       children: [
 *         { type: "text", content: new Date().toLocaleTimeString(),
 *           fontSize: 24, fontWeight: "bold", color: "#fff" },
 *       ],
 *     },
 *   }),
 *   "group.com.example.myapp",
 *   {
 *     intervalMs: 60_000,
 *     reload: true,
 *     onAction: (action, payload) => {
 *       console.log("Widget action:", action, payload);
 *     },
 *   },
 * );
 *
 * // later — stop updating and action listener:
 * stop();
 * ```
 */
export async function startWidgetUpdater(
  builder: () => WidgetConfig | Promise<WidgetConfig>,
  group: string,
  options?: {
    /** Tick interval in ms.  Use `0` for a one-shot call (no periodic updates). Default: `1000`. */
    intervalMs?: number;
    /** Call `builder` immediately before starting the interval. Default: `true`. */
    immediate?: boolean;
    /** Also call `reloadAllTimelines()` after each tick. Default: `false`.
     *  Backend throttles to once per 15 min on iOS/macOS. */
    reload?: boolean;
    /** Subscribe to `widget-action` events for the lifetime of this updater.
     *  The listener is automatically removed when the returned stop function is called. */
    onAction?: (action: string, payload?: string) => void;
  },
): Promise<() => void> {
  const intervalMs = options?.intervalMs ?? 1000;
  const immediate = options?.immediate ?? true;
  const reload = options?.reload ?? false;

  let running = false;

  async function tick() {
    if (running) return;
    running = true;
    try {
      const config = await builder();
      await setWidgetConfig(config, group);
      if (reload) {
        await reloadAllTimelines();
      }
    } catch (e) {
      console.error("[widget-updater] tick failed:", e);
    } finally {
      running = false;
    }
  }

  if (immediate) {
    await tick();
  }

  const id = intervalMs > 0 ? setInterval(tick, intervalMs) : null;

  let actionUnsub: (() => void) | null = null;
  if (options?.onAction) {
    const handler = options.onAction;
    actionUnsub = await onWidgetAction((data) => {
      handler(data.action, data.payload);
    });
  }

  return () => {
    if (id !== null) clearInterval(id);
    if (actionUnsub) { actionUnsub(); actionUnsub = null; }
  };
}
