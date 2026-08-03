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
export type ContentMode = "fit" | "fill";
export type ProgressStyle = "linear" | "circular";
export type GaugeStyle = "circular" | "linear";
export type DateStyle = "time" | "date" | "relative" | "offset" | "timer";
export type ChartType = "bar" | "line" | "area" | "pie";
export type ShapeType = "circle" | "capsule" | "rectangle";
export type TimerCounting = "up" | "down";
export type ClipShape = "circle" | "capsule" | "rectangle";
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
}

export interface FrameConfig {
  width?: number;
  height?: number;
  maxWidth?: number | "infinity";
  maxHeight?: number | "infinity";
}

export interface BorderConfig {
  color: string;
  width?: number;
}

export interface GradientConfig {
  gradientType: GradientType;
  colors: string[];
  direction?: GradientDirection;
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
}

export interface VStackElement extends ElementStyle {
  type: "vstack";
  children: WidgetElement[];
  spacing?: number;
  alignment?: HorizontalAlignment;
}

export interface HStackElement extends ElementStyle {
  type: "hstack";
  children: WidgetElement[];
  spacing?: number;
  alignment?: VerticalAlignment;
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
}

export interface ImageElement extends ElementStyle {
  type: "image";
  systemName?: string;
  data?: string;
  url?: string;
  size?: number;
  color?: ColorValue;
  contentMode?: ContentMode;
}

export interface ProgressElement extends ElementStyle {
  type: "progress";
  value: number;
  total?: number;
  label?: string;
  tint?: ColorValue;
  color?: ColorValue;
  barStyle?: ProgressStyle;
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
}

export interface ToggleElement extends ElementStyle {
  type: "toggle";
  isOn: boolean;
  label?: string;
  tint?: ColorValue;
  action?: string;
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
}

export interface ListElement extends ElementStyle {
  type: "list";
  items: ListItem[];
  spacing?: number;
  fontSize?: number;
  color?: ColorValue;
}

export interface LinkElement extends ElementStyle {
  type: "link";
  children: WidgetElement[];
  url?: string;
  action?: string;
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
}
