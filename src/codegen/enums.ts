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

