/**
 * Generated widget IR types — do not edit by hand.
 * Source of truth: Rust `src/models` via `cargo run --bin gen-ts --features codegen`.
 * Emitter: `src/codegen.rs` IR_SPEC (+ `src/codegen/preamble.ts`).
 */


// ── Enums ──

export type FontWeight = "ultralight" | "thin" | "light" | "regular" | "medium" | "semibold" | "bold" | "heavy" | "black";
export type FontDesign = "default" | "monospaced" | "rounded" | "serif";
export type TextAlignment = "leading" | "center" | "trailing";
export type HorizontalAlignment = "leading" | "center" | "trailing";
export type VerticalAlignment = "top" | "center" | "bottom";
export type ContentMode = "fit" | "fill" | "stretch" | "center";
export type ProgressStyle = "linear" | "circular";
export type GaugeStyle = "circular" | "linear";
export type DateStyle = "time" | "date" | "relative" | "offset" | "timer";
export type ChartType = "bar" | "line" | "area" | "pie" | "point";
export type ShapeType = "circle" | "capsule" | "rectangle" | "roundedRectangle" | "ellipse";
export type TimerCounting = "up" | "down";
export type ClipShape = "circle" | "capsule" | "rectangle" | "roundedRectangle" | "ellipse";
export type Justify = "start" | "center" | "end" | "spaceBetween" | "spaceAround" | "spaceEvenly";
export type TruncationMode = "head" | "middle" | "tail";
export type SymbolRendering = "monochrome" | "hierarchical" | "palette" | "multicolor";
export type ButtonVariant = "bordered" | "prominent" | "plain" | "outline";
export type ToggleVariant = "switch" | "checkbox";
/** How content changes between entries / redraws (Apple and desktop animate). */
export type Transition = "none" | "opacity" | "slide" | "numeric";
export type ChartInterpolation = "linear" | "monotone" | "step";
export type TextStyle =
  | "largeTitle" | "title" | "title2" | "title3"
  | "headline" | "subheadline"
  | "body" | "callout"
  | "footnote" | "caption" | "caption2";
export type GradientType = "linear" | "radial" | "angular";
export type GradientDirection =
  | "topToBottom" | "bottomToTop"
  | "leadingToTrailing" | "trailingToLeading"
  | "topLeadingToBottomTrailing" | "topTrailingToBottomLeading";

// ── Supporting types ──

export type ColorValue = string | { light: string; dark: string };

export interface ChartDataPoint {
  label: string;
  value: number;
  color?: ColorValue;
  /** Points with the same series form one line / bar group. */
  series?: string;
}

export interface FrameConfig {
  width?: number;
  height?: number;
  maxWidth?: number | "infinity";
  maxHeight?: number | "infinity";
  minWidth?: number;
  minHeight?: number;
}

export interface BorderConfig {
  color: string;
  width?: number;
}

export interface GradientConfig {
  gradientType: GradientType;
  colors?: string[];
  direction?: GradientDirection;
  /** Colors at explicit positions (0…1); replaces `colors`. */
  stops?: { color: string; location: number }[];
  /** Unit points (`{x:0,y:0}` = top leading); override `direction`. */
  startPoint?: { x: number; y: number };
  endPoint?: { x: number; y: number };
}

export interface ShadowConfig {
  color?: string;
  radius?: number;
  x?: number;
  y?: number;
}

export type BackgroundValue = string | GradientConfig | { light: string; dark: string };

export type PaddingValue = number | {
  top?: number;
  bottom?: number;
  leading?: number;
  trailing?: number;
};

export interface ElementStyle {
  padding?: PaddingValue;
  background?: BackgroundValue;
  cornerRadius?: number;
  opacity?: number;
  frame?: FrameConfig;
  border?: BorderConfig;
  shadow?: ShadowConfig;
  clipShape?: ClipShape;
  flex?: number;
  /** Redacted where the OS hides private data (Apple locked screen / StandBy). */
  privacySensitive?: boolean;
  /** Screen-reader text instead of the visible content. */
  accessibilityLabel?: string;
  /** Set fields from data with their JSON type: `{ value: "data.progress" }`. */
  bind?: Record<string, string>;
  /** Outer spacing outside background and border. */
  margin?: PaddingValue;
  /** Keep width / height at this ratio. */
  aspectRatio?: number;
  /** Higher keeps its size first in a crowded stack. */
  layoutPriority?: number;
  /** Move after layout (points). */
  offsetX?: number;
  offsetY?: number;
  /** Rotate (degrees, clockwise) / scale. */
  rotation?: number;
  scale?: number;
  /** Apple tinted Home Screen: draw in the accent color. */
  accentable?: boolean;
  /** Show only when the condition holds (`{ path: "data.n", gt: 0 }`); removed before rendering. */
  when?: WhenCondition;
  /** How this element changes between entries / redraws: `numeric`, `opacity`, `slide`, `none`. */
  transition?: Transition;
}

/** `set` on a button / toggle / link: local state written on tap without the app.
 *  `"$toggle"`, `"$inc"`, `"$dec"`, `null` (remove the key) or a JSON value. */
export type StatePatch = Record<string, unknown>;

/** Structured condition for `when`. */
export type WhenCondition =
  | { all: WhenCondition[] }
  | { any: WhenCondition[] }
  | { not: WhenCondition }
  | {
      path: string;
      eq?: unknown;
      ne?: unknown;
      gt?: number;
      gte?: number;
      lt?: number;
      lte?: number;
      exists?: boolean;
      in?: unknown[];
    };

export interface VStackElement extends ElementStyle {
  type: "vstack";
  children: WidgetElement[];
  spacing?: number;
  alignment?: HorizontalAlignment;
  justify?: Justify;
}

export interface HStackElement extends ElementStyle {
  type: "hstack";
  children: WidgetElement[];
  spacing?: number;
  alignment?: VerticalAlignment;
  justify?: Justify;
}

export interface ZStackElement extends ElementStyle {
  type: "zstack";
  children: WidgetElement[];
  alignment?: string;
}

export interface GridElement extends ElementStyle {
  type: "grid";
  children: WidgetElement[];
  columns?: number;
  spacing?: number;
  rowSpacing?: number;
}

export interface ContainerElement extends ElementStyle {
  type: "container";
  children?: WidgetElement[];
  contentAlignment?: string;
}

export interface TextElement extends ElementStyle {
  type: "text";
  content: string;
  fontSize?: number;
  fontWeight?: FontWeight;
  fontDesign?: FontDesign;
  textStyle?: TextStyle;
  color?: ColorValue;
  alignment?: TextAlignment;
  lineLimit?: number;
  truncationMode?: TruncationMode;
  minimumScaleFactor?: number;
  lineHeight?: number;
  letterSpacing?: number;
  underline?: boolean;
  strikethrough?: boolean;
  italic?: boolean;
  monospacedDigit?: boolean;
}

export interface ImageElement extends ElementStyle {
  type: "image";
  systemName?: string;
  data?: string;
  url?: string;
  size?: number;
  color?: ColorValue;
  contentMode?: ContentMode;
  fallbackSystemName?: string;
  symbolRendering?: SymbolRendering;
  symbolWeight?: FontWeight;
  paletteColors?: ColorValue[];
}

export interface ProgressElement extends ElementStyle {
  type: "progress";
  value: number;
  total?: number;
  label?: string;
  tint?: ColorValue;
  color?: ColorValue;
  barStyle?: ProgressStyle;
  startAt?: number;
  endAt?: number;
  trackColor?: ColorValue;
  thickness?: number;
}

export interface GaugeElement extends ElementStyle {
  type: "gauge";
  value: number;
  min?: number;
  max?: number;
  label?: string;
  currentValueLabel?: string;
  tint?: ColorValue;
  color?: ColorValue;
  gaugeStyle?: GaugeStyle;
  startAt?: number;
  endAt?: number;
  minLabel?: string;
  maxLabel?: string;
}

export interface ButtonElement extends ElementStyle {
  type: "button";
  label: string;
  url?: string;
  action?: string;
  color?: ColorValue;
  backgroundColor?: ColorValue;
  fontSize?: number;
  textAlignment?: TextAlignment;
  variant?: ButtonVariant;
  systemName?: string;
  disabled?: boolean;
  set?: StatePatch;
}

export interface ToggleElement extends ElementStyle {
  type: "toggle";
  isOn: boolean;
  label?: string;
  tint?: ColorValue;
  action?: string;
  variant?: ToggleVariant;
  uncheckedTint?: ColorValue;
  set?: StatePatch;
}

export interface DividerElement extends ElementStyle {
  type: "divider";
  color?: ColorValue;
  thickness?: number;
}

export interface DateElement extends ElementStyle {
  type: "date";
  date: string;
  dateStyle?: DateStyle;
  fontSize?: number;
  color?: ColorValue;
}

export interface ChartElement extends ElementStyle {
  type: "chart";
  chartType: ChartType;
  chartData: ChartDataPoint[];
  tint?: ColorValue;
  showXAxis?: boolean;
  showYAxis?: boolean;
  showLegend?: boolean;
  yMin?: number;
  yMax?: number;
  interpolation?: ChartInterpolation;
  innerRadius?: number;
}

export interface ListElement extends ElementStyle {
  type: "list";
  items: ListItem[];
  children?: WidgetElement[];
  spacing?: number;
  fontSize?: number;
  color?: ColorValue;
}

export interface LinkElement extends ElementStyle {
  type: "link";
  children: WidgetElement[];
  url?: string;
  action?: string;
  set?: StatePatch;
}

export interface ShapeElement extends ElementStyle {
  type: "shape";
  shapeType: ShapeType;
  fill?: ColorValue;
  stroke?: ColorValue;
  strokeWidth?: number;
  size?: number;
}

export interface TimerElement extends ElementStyle {
  type: "timer";
  targetDate: string;
  counting?: TimerCounting;
  fontSize?: number;
  fontWeight?: FontWeight;
  color?: ColorValue;
  startDate?: string;
  showHours?: boolean;
  completedText?: string;
}

export interface CanvasElement extends ElementStyle {
  type: "canvas";
  width: number;
  height: number;
  elements: CanvasDrawCommand[];
}

export interface LabelElement extends ElementStyle {
  type: "label";
  text: string;
  systemName: string;
  iconColor?: ColorValue;
  fontSize?: number;
  fontWeight?: FontWeight;
  color?: ColorValue;
  spacing?: number;
}

export interface RepeatElement extends ElementStyle {
  type: "repeat";
  each: string;
  as?: string;
  limit?: number;
  template: WidgetElement;
}

export interface UseElement extends ElementStyle {
  type: "use";
  template: string;
  args?: Record<string, unknown>;
}

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
  | DateElement
  | ChartElement
  | ListElement
  | LinkElement
  | ShapeElement
  | TimerElement
  | CanvasElement
  | LabelElement
  | RepeatElement
  | UseElement
  | SpacerElement;

export interface SpacerElement {
  type: "spacer";
  minLength?: number;
}

export interface ListItem {
  text: string;
  checked?: boolean;
  action?: string;
  payload?: string;
}

export interface CanvasCircle {
  draw: "circle";
  cx: number; cy: number; r: number;
  fill?: ColorValue; stroke?: ColorValue; strokeWidth?: number;
}
export interface CanvasLine {
  draw: "line";
  x1: number; y1: number; x2: number; y2: number;
  stroke?: ColorValue; strokeWidth?: number; lineCap?: "butt" | "round" | "square";
}
export interface CanvasRect {
  draw: "rect";
  x: number; y: number; width: number; height: number;
  fill?: ColorValue; stroke?: ColorValue; strokeWidth?: number; cornerRadius?: number;
}
export interface CanvasArc {
  draw: "arc";
  cx: number; cy: number; r: number;
  startAngle: number; endAngle: number;
  fill?: ColorValue; stroke?: ColorValue; strokeWidth?: number;
}
export interface CanvasText {
  draw: "text";
  x: number; y: number; content: string;
  fontSize?: number; color?: ColorValue; anchor?: "start" | "middle" | "end";
}
export interface CanvasPath {
  draw: "path";
  d: string;
  fill?: ColorValue; stroke?: ColorValue; strokeWidth?: number;
}
export type CanvasDrawCommand = CanvasCircle | CanvasLine | CanvasRect | CanvasArc | CanvasText | CanvasPath;

export interface WidgetConfig {
  version?: number;
  small?: WidgetElement;
  medium?: WidgetElement;
  large?: WidgetElement;
  /** iOS Lock Screen accessory families; falls back to `small`. Not resolved (no `{{…}}`). */
  accessory?: WidgetElement;
  /** Scheduled states: from `at` (Unix ms) on, an entry's layouts replace the base ones. */
  entries?: WidgetEntry[];
  /** Localized strings per locale, used as `{{t.key}}` in any text. */
  strings?: Record<string, Record<string, string>>;
  /** Locale used when the device locale has no `strings` table. */
  defaultLocale?: string;
  /** JSON the widget fetches by itself; its fields are `{{data.path}}`. */
  source?: DataSource;
  /** Default values for `{{data.path}}` (overridden by `setWidgetData` and `source`). */
  data?: Record<string, unknown>;
  /** Local state defaults, `{{state.path}}`; `set` taps and `setWidgetState` change it. */
  state?: Record<string, unknown>;
  /** How content changes between entries / redraws (Apple and desktop animate). */
  transition?: Transition;
  /** Reusable elements expanded by `{ type: "use", template: "<key>" }`. */
  templates?: Record<string, WidgetElement>;
  /** Minimum renderer contract; older renderers show `fallback`. */
  requires?: { resolve?: number };
  /** Layouts for renderers below `requires`. */
  fallback?: { small?: WidgetElement; medium?: WidgetElement; large?: WidgetElement };
}

/** One scheduled state of a widget. */
export interface WidgetEntry {
  /** Unix epoch ms from which this entry is shown. */
  at: number;
  small?: WidgetElement;
  medium?: WidgetElement;
  large?: WidgetElement;
}

/** Remote JSON the widget fetches without the app. `url`, header values and `body` may use `{{instance}}` / `{{locale}}`. */
export interface DataSource {
  /** `https://` URL returning a JSON object (at most 256 KiB). */
  url: string;
  /** Minutes between fetches (15..1440, default 30). */
  refreshMinutes?: number;
  /** `GET` (default) or `POST`. */
  method?: string;
  headers?: Record<string, string>;
  body?: string;
}
