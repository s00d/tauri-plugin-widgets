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
