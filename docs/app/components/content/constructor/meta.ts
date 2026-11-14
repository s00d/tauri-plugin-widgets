/**
 * Constructor metadata: how each element is presented (icon, one-line purpose) and which
 * properties the inspector shows first, as proper controls. Everything else stays reachable
 * in "All properties" (schema-driven). Enum choices come from the JSON Schema.
 */

export type FieldKind = "text" | "textarea" | "number" | "slider" | "color" | "enum" | "bool"

export type FieldSpec = {
  /** Property name; `frame.width` style paths edit nested objects. */
  key: string
  label: string
  kind: FieldKind
  min?: number
  max?: number
  step?: number
  placeholder?: string
  /** What the renderer uses when the property is unset (shown greyed by sliders). */
  def?: number
}

export type ElementMeta = { label: string; icon: string; description: string; fields: FieldSpec[] }

const text = (key: string, label: string, placeholder?: string): FieldSpec => ({ key, label, kind: "text", placeholder })
const num = (key: string, label: string, min = 0, max = 200, step = 1): FieldSpec => ({ key, label, kind: "number", min, max, step })
const slider = (key: string, label: string, min = 0, max = 1, step = 0.01, def = min): FieldSpec => ({ key, label, kind: "slider", min, max, step, def })
const color = (key: string, label: string): FieldSpec => ({ key, label, kind: "color" })
const pick = (key: string, label: string): FieldSpec => ({ key, label, kind: "enum" })
const bool = (key: string, label: string): FieldSpec => ({ key, label, kind: "bool" })

export const META: Record<string, ElementMeta> = {
  vstack: { label: "VStack", icon: "i-lucide-rows-3", description: "Children top to bottom", fields: [num("spacing", "Spacing", 0, 64), pick("alignment", "Align"), pick("justify", "Distribute")] },
  hstack: { label: "HStack", icon: "i-lucide-columns-3", description: "Children left to right", fields: [num("spacing", "Spacing", 0, 64), pick("alignment", "Align"), pick("justify", "Distribute")] },
  zstack: { label: "ZStack", icon: "i-lucide-layers", description: "Children on top of each other", fields: [pick("alignment", "Align")] },
  grid: { label: "Grid", icon: "i-lucide-layout-grid", description: "Rows of N columns", fields: [num("columns", "Columns", 1, 8), num("spacing", "Column gap", 0, 64), num("rowSpacing", "Row gap", 0, 64)] },
  container: { label: "Container", icon: "i-lucide-square", description: "One box with padding and a background", fields: [pick("contentAlignment", "Align content")] },
  text: { label: "Text", icon: "i-lucide-type", description: "A line or paragraph", fields: [{ key: "content", label: "Text", kind: "textarea" }, num("fontSize", "Size", 6, 96), pick("fontWeight", "Weight"), color("color", "Color"), pick("alignment", "Align"), num("lineLimit", "Max lines", 0, 20)] },
  label: { label: "Label", icon: "i-lucide-tag", description: "Icon and text in one row", fields: [text("text", "Text"), text("systemName", "Icon (SF Symbol)", "star.fill"), color("iconColor", "Icon color"), color("color", "Text color"), num("fontSize", "Size", 6, 64)] },
  date: { label: "Date", icon: "i-lucide-calendar", description: "A date or time, formatted per locale", fields: [text("date", "Date (ISO)"), pick("dateStyle", "Style"), color("color", "Color"), num("fontSize", "Size", 6, 64)] },
  timer: { label: "Timer", icon: "i-lucide-timer", description: "A live countdown or stopwatch", fields: [text("targetDate", "Target (ISO)"), pick("counting", "Counting"), color("color", "Color"), num("fontSize", "Size", 6, 64)] },
  image: { label: "Image", icon: "i-lucide-image", description: "SF Symbol, base64 or URL", fields: [text("systemName", "SF Symbol", "star.fill"), text("url", "URL", "https://…"), num("size", "Size", 8, 200), color("color", "Tint"), pick("contentMode", "Fit")] },
  shape: { label: "Shape", icon: "i-lucide-circle", description: "Rectangle, circle or capsule", fields: [pick("shapeType", "Shape"), color("fill", "Fill"), num("size", "Size", 4, 200), color("stroke", "Stroke"), num("strokeWidth", "Stroke width", 0, 20)] },
  canvas: { label: "Canvas", icon: "i-lucide-pen-tool", description: "Free drawing: lines, arcs, paths", fields: [num("width", "Width", 10, 400), num("height", "Height", 10, 400)] },
  progress: { label: "Progress", icon: "i-lucide-loader", description: "A bar or ring filling up", fields: [slider("value", "Value", 0, 1), text("label", "Label"), color("tint", "Fill"), pick("barStyle", "Style")] },
  gauge: { label: "Gauge", icon: "i-lucide-gauge", description: "A value on a dial", fields: [slider("value", "Value", 0, 1), text("currentValueLabel", "Value text"), text("label", "Label"), color("tint", "Tint"), pick("gaugeStyle", "Style")] },
  chart: { label: "Chart", icon: "i-lucide-chart-column", description: "Bars, lines, points or a pie", fields: [pick("chartType", "Type"), color("tint", "Tint"), bool("showLegend", "Legend"), bool("showYAxis", "Y axis")] },
  list: { label: "List", icon: "i-lucide-list", description: "Rows of items", fields: [num("spacing", "Spacing", 0, 32), color("color", "Text color"), num("fontSize", "Size", 6, 32)] },
  button: { label: "Button", icon: "i-lucide-mouse-pointer-click", description: "Sends an action to your app", fields: [text("label", "Label"), text("action", "Action id"), text("systemName", "Icon (SF Symbol)"), color("backgroundColor", "Background"), color("color", "Text color"), pick("variant", "Variant")] },
  toggle: { label: "Toggle", icon: "i-lucide-toggle-right", description: "On / off, a checkbox or a switch", fields: [text("label", "Label"), bool("isOn", "On"), text("action", "Action id"), color("tint", "Tint"), pick("variant", "Variant")] },
  link: { label: "Link", icon: "i-lucide-link", description: "Makes its children tappable", fields: [text("url", "URL"), text("action", "Action id")] },
  spacer: { label: "Spacer", icon: "i-lucide-move-vertical", description: "Takes the free space", fields: [num("minLength", "Min length", 0, 200)] },
  divider: { label: "Divider", icon: "i-lucide-minus", description: "A thin line between items", fields: [color("color", "Color"), num("thickness", "Thickness", 1, 8)] },
}

/** Box properties every element has, shown under "Layout" and "Style". */
export const LAYOUT_FIELDS: FieldSpec[] = [
  num("padding", "Padding", 0, 64),
  num("frame.width", "Width", 0, 400),
  num("frame.height", "Height", 0, 400),
  num("flex", "Flex", 0, 10, 1),
]
export const STYLE_FIELDS: FieldSpec[] = [
  color("background", "Background"),
  num("cornerRadius", "Corner radius", 0, 64),
  slider("opacity", "Opacity", 0, 1, 0.01, 1),
]

export const metaOf = (type: string): ElementMeta =>
  META[type] ?? { label: type, icon: "i-lucide-box", description: "", fields: [] }

/** Reads `a.b` paths. */
export function getPath(node: Record<string, unknown>, key: string): unknown {
  return key.split(".").reduce<unknown>((o, k) => (o && typeof o === "object" ? (o as Record<string, unknown>)[k] : undefined), node)
}

/** Writes `a.b` paths; `undefined` / "" removes the key (and an emptied parent object). */
export function setPath(node: Record<string, unknown>, key: string, value: unknown) {
  const parts = key.split(".")
  let o = node
  for (const p of parts.slice(0, -1)) {
    if (!o[p] || typeof o[p] !== "object") o[p] = {}
    o = o[p] as Record<string, unknown>
  }
  const last = parts[parts.length - 1]!
  if (value === undefined || value === "" || (typeof value === "number" && Number.isNaN(value))) delete o[last]
  else o[last] = value
  if (parts.length > 1 && Object.keys(o).length === 0) delete node[parts[0]!]
}
