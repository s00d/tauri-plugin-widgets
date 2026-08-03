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

