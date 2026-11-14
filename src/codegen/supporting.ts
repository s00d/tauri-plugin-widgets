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

