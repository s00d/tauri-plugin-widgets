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
