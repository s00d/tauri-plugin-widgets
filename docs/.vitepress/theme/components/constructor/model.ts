/** Shared groups with docs codegen (`scripts/.../shared.ts` ELEMENT_GROUPS). */
export const ELEMENT_GROUPS: { id: string; label: string; types: string[] }[] = [
  {
    id: "layout",
    label: "Layout",
    types: ["vstack", "hstack", "zstack", "grid", "container"],
  },
  {
    id: "text",
    label: "Text",
    types: ["text", "label", "date", "timer"],
  },
  {
    id: "media",
    label: "Media",
    types: ["image", "shape", "canvas"],
  },
  {
    id: "data",
    label: "Data",
    types: ["progress", "gauge", "chart", "list"],
  },
  {
    id: "interactive",
    label: "Interactive",
    types: ["button", "toggle", "link"],
  },
  {
    id: "spacing",
    label: "Spacing",
    types: ["spacer", "divider"],
  },
];

export const ALL_TYPES = ELEMENT_GROUPS.flatMap((g) => g.types);

export type SizeKey = "small" | "medium" | "large";
export type WidgetNode = Record<string, unknown> & {
  type: string;
  children?: WidgetNode[];
  items?: WidgetNode[];
};

export type WidgetConfigState = {
  version?: number;
  small?: WidgetNode | null;
  medium?: WidgetNode | null;
  large?: WidgetNode | null;
};

/** Minimal defaults when dropping a type — not a full property encyclopedia. */
const DEFAULTS: Record<string, () => WidgetNode> = {
  vstack: () => ({
    type: "vstack",
    padding: 12,
    spacing: 8,
    background: "#1a1a2e",
    children: [],
  }),
  hstack: () => ({ type: "hstack", spacing: 8, children: [] }),
  zstack: () => ({ type: "zstack", children: [] }),
  grid: () => ({ type: "grid", columns: 2, spacing: 8, children: [] }),
  container: () => ({
    type: "container",
    padding: 8,
    background: "#222",
    cornerRadius: 8,
    children: [],
  }),
  text: () => ({ type: "text", content: "Text", color: "#fff", fontSize: 16 }),
  label: () => ({
    type: "label",
    text: "Label",
    systemName: "star.fill",
    color: "#fff",
  }),
  date: () => ({
    type: "date",
    date: new Date().toISOString(),
    dateStyle: "date",
    color: "#fff",
    fontSize: 14,
  }),
  timer: () => ({
    type: "timer",
    targetDate: new Date(Date.now() + 3600_000).toISOString(),
    color: "#fff",
  }),
  image: () => ({
    type: "image",
    systemName: "photo",
    size: 32,
    color: "#38bdf8",
  }),
  shape: () => ({
    type: "shape",
    shapeType: "rectangle",
    fill: "#38bdf8",
    size: 40,
    cornerRadius: 8,
  }),
  canvas: () => ({
    type: "canvas",
    width: 80,
    height: 40,
    elements: [
      {
        draw: "rect",
        x: 0,
        y: 0,
        width: 80,
        height: 40,
        fill: "#334155",
      },
    ],
  }),
  progress: () => ({ type: "progress", value: 0.6, total: 1, tint: "#38bdf8" }),
  gauge: () => ({ type: "gauge", value: 0.7, min: 0, max: 1, tint: "#38bdf8" }),
  chart: () => ({
    type: "chart",
    chartType: "bar",
    tint: "#38bdf8",
    chartData: [
      { label: "a", value: 3 },
      { label: "b", value: 5 },
      { label: "c", value: 2 },
    ],
  }),
  list: () => ({
    type: "list",
    children: [
      { type: "text", content: "Item 1", color: "#fff", fontSize: 14 },
      { type: "text", content: "Item 2", color: "#fff", fontSize: 14 },
    ],
  }),
  button: () => ({
    type: "button",
    label: "Tap",
    action: "tap",
    backgroundColor: "#0284c7",
  }),
  toggle: () => ({ type: "toggle", isOn: true, action: "toggle" }),
  link: () => ({
    type: "link",
    url: "https://example.com",
    children: [{ type: "text", content: "Link", color: "#38bdf8", fontSize: 14 }],
  }),
  spacer: () => ({ type: "spacer" }),
  divider: () => ({ type: "divider", color: "#475569" }),
};

export function createDefaultConfig(): WidgetConfigState {
  return {
    version: 1,
    small: {
      type: "vstack",
      padding: 12,
      spacing: 8,
      background: "#1a1a2e",
      children: [{ type: "text", content: "Hello", color: "#fff", fontSize: 18 }],
    },
  };
}

export function createNode(type: string): WidgetNode {
  const factory = DEFAULTS[type];
  return factory ? structuredClone(factory()) : { type };
}

export function isContainer(node: WidgetNode | null | undefined): boolean {
  if (!node) return false;
  return (
    Array.isArray(node.children) ||
    node.type === "vstack" ||
    node.type === "hstack" ||
    node.type === "zstack" ||
    node.type === "grid" ||
    node.type === "container" ||
    node.type === "list" ||
    node.type === "link" ||
    node.type === "button"
  );
}

function childKey(node: WidgetNode): "children" | "items" | null {
  // List typed `items` are ListItem rows — edit via inspector JSON.
  // Tree for lists always uses `children` (rich rows).
  if (node.type === "list") return "children";
  if (Array.isArray(node.children)) return "children";
  if (Array.isArray(node.items)) return "items";
  if (isContainer(node)) return "children";
  return null;
}

export function getChildren(node: WidgetNode): WidgetNode[] {
  const key = childKey(node);
  if (!key) return [];
  if (!Array.isArray(node[key])) node[key] = [];
  return node[key] as WidgetNode[];
}

export type NodePath = number[]; // indices into children arrays from size root

export function getAtPath(root: WidgetNode | null | undefined, path: NodePath): WidgetNode | null {
  if (!root) return null;
  let cur: WidgetNode = root;
  for (const idx of path) {
    const kids = getChildren(cur);
    if (idx < 0 || idx >= kids.length) return null;
    cur = kids[idx];
  }
  return cur;
}

export function removeAtPath(root: WidgetNode, path: NodePath): WidgetNode | null {
  if (path.length === 0) return null;
  const parent = getAtPath(root, path.slice(0, -1));
  if (!parent) return null;
  const kids = getChildren(parent);
  const idx = path[path.length - 1];
  if (idx < 0 || idx >= kids.length) return null;
  const [removed] = kids.splice(idx, 1);
  return removed ?? null;
}

export function insertAtPath(
  root: WidgetNode,
  parent: NodePath,
  index: number,
  node: WidgetNode,
): boolean {
  const p = parent.length === 0 ? root : getAtPath(root, parent);
  if (!p || !isContainer(p)) return false;
  const kids = getChildren(p);
  const i = Math.max(0, Math.min(index, kids.length));
  kids.splice(i, 0, node);
  return true;
}

export function moveSibling(root: WidgetNode, path: NodePath, dir: -1 | 1): NodePath | null {
  if (path.length === 0) return null;
  const parent = getAtPath(root, path.slice(0, -1));
  if (!parent) return null;
  const kids = getChildren(parent);
  const idx = path[path.length - 1];
  const next = idx + dir;
  if (next < 0 || next >= kids.length) return null;
  const [item] = kids.splice(idx, 1);
  if (!item) return null;
  kids.splice(next, 0, item);
  return [...path.slice(0, -1), next];
}

export function duplicateAtPath(root: WidgetNode, path: NodePath): NodePath | null {
  if (path.length === 0) {
    if (!isContainer(root)) return null;
    const clone = structuredClone(root);
    const kids = getChildren(root);
    kids.push(clone);
    return [kids.length - 1];
  }
  const parent = getAtPath(root, path.slice(0, -1));
  if (!parent) return null;
  const kids = getChildren(parent);
  const idx = path[path.length - 1];
  const src = kids[idx];
  if (!src) return null;
  kids.splice(idx + 1, 0, structuredClone(src));
  return [...path.slice(0, -1), idx + 1];
}

export function insertAllTypes(): WidgetNode {
  const root = createNode("vstack");
  root.padding = 8;
  root.spacing = 6;
  root.background = "#0f172a";
  const kids = getChildren(root);
  for (const type of ALL_TYPES) {
    if (type === "vstack") continue;
    kids.push(createNode(type));
  }
  return root;
}

export function softValidate(config: unknown): string[] {
  const warnings: string[] = [];
  if (!config || typeof config !== "object") {
    return ["Config must be an object"];
  }
  const c = config as WidgetConfigState;
  const sizes: SizeKey[] = ["small", "medium", "large"];
  let any = false;
  for (const s of sizes) {
    if (c[s]) {
      any = true;
      walkWarn(c[s] as WidgetNode, s, warnings);
    }
  }
  if (!any) warnings.push("No size root (small / medium / large)");
  return warnings;
}

function walkWarn(node: WidgetNode, where: string, warnings: string[]) {
  if (!node?.type || typeof node.type !== "string") {
    warnings.push(`${where}: missing type`);
    return;
  }
  if (!ALL_TYPES.includes(node.type)) {
    warnings.push(`${where}: unknown type "${node.type}"`);
  }
  if (node.type === "list") {
    const items = Array.isArray(node.items) ? node.items : [];
    for (const [i, row] of items.entries()) {
      if (!row || typeof row !== "object") {
        warnings.push(`${where}/list.items[${i}]: expected object`);
        continue;
      }
      const r = row as Record<string, unknown>;
      if (typeof r.text !== "string" || !r.text) {
        warnings.push(`${where}/list.items[${i}]: missing text`);
      }
    }
    for (const [i, child] of getChildren(node).entries()) {
      walkWarn(child, `${where}/list.children[${i}]`, warnings);
    }
    return;
  }
  for (const [i, child] of getChildren(node).entries()) {
    walkWarn(child, `${where}/${node.type}[${i}]`, warnings);
  }
}
