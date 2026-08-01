import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
export const REPO_ROOT = path.resolve(__dirname, "../..");
export const FIXTURES_ROOT = path.join(REPO_ROOT, "tests/fixtures");
export const EXPECTED_GEOMETRY = path.join(REPO_ROOT, "tests/expected/geometry");
export const EXPECTED_PIXELS = path.join(REPO_ROOT, "tests/expected/pixels");
export const WIDGET_HTML = path.join(REPO_ROOT, "widget.html");

export const SIZE_VIEWPORTS: Record<string, { width: number; height: number }> = {
  small: { width: 170, height: 170 },
  medium: { width: 360, height: 170 },
  large: { width: 360, height: 380 },
};

export type GeometryNode = {
  kind: string;
  rect: [number, number, number, number];
  text?: string | null;
  px?: number | null;
  color?: string | null;
  children?: GeometryNode[];
};

/** Walk fixture dirs and return { relPath, absPath, sizes } for each JSON. */
export function listFixtures(): Array<{ id: string; absPath: string; sizes: string[] }> {
  const out: Array<{ id: string; absPath: string; sizes: string[] }> = [];
  for (const folder of ["core", "bugs", "presets"]) {
    const dir = path.join(FIXTURES_ROOT, folder);
    if (!fs.existsSync(dir)) continue;
    for (const name of fs.readdirSync(dir).filter((f) => f.endsWith(".json")).sort()) {
      const absPath = path.join(dir, name);
      const raw = JSON.parse(fs.readFileSync(absPath, "utf8")) as Record<string, unknown>;
      const sizes = (["small", "medium", "large"] as const).filter((s) => raw[s] != null);
      if (sizes.length === 0) continue;
      out.push({ id: `${folder}/${name.replace(/\.json$/, "")}`, absPath, sizes: [...sizes] });
    }
  }
  return out;
}

export function widgetHtmlUrl(size: string): string {
  const base = pathToFileURL(WIDGET_HTML).href;
  return `${base}?group=g&widgetId=w&size=${encodeURIComponent(size)}`;
}

export function geometryPath(id: string, size: string, platform: "desktop" | "android" = "desktop"): string {
  const safe = id.replace(/\//g, "__");
  return path.join(EXPECTED_GEOMETRY, `${safe}.${size}.${platform}.json`);
}

export function pixelPath(
  id: string,
  size: string,
  platform: "desktop" | "android" | "ios",
): string {
  const safe = id.replace(/\//g, "__");
  return path.join(EXPECTED_PIXELS, platform, `${safe}.${size}.png`);
}

/** Drop zero-size wrappers; round rects. */
export function normalizeTree(node: GeometryNode): GeometryNode | null {
  const kids = (node.children ?? [])
    .map(normalizeTree)
    .filter((c): c is GeometryNode => c != null);
  const [x, y, w, h] = node.rect.map((n) => Math.round(n)) as [number, number, number, number];
  if (w <= 0 && h <= 0 && kids.length === 0 && !node.text) return null;
  const out: GeometryNode = {
    kind: node.kind,
    rect: [x, y, w, h],
  };
  if (node.text != null && node.text !== "") out.text = node.text;
  if (node.px != null && !Number.isNaN(node.px)) out.px = Math.round(node.px);
  if (node.color) out.color = normalizeColor(node.color);
  if (kids.length) out.children = kids;
  return out;
}

function normalizeColor(c: string): string {
  const m = c.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/i);
  if (m) {
    const hex = (n: string) => Number(n).toString(16).padStart(2, "0");
    return `#${hex(m[1])}${hex(m[2])}${hex(m[3])}`.toUpperCase();
  }
  if (c.startsWith("#")) return c.toUpperCase();
  return c;
}

export function assertNoLiteralNullText(node: GeometryNode): void {
  if (node.text === "null" || node.text === "undefined") {
    throw new Error(`Literal "${node.text}" text in geometry tree (${node.kind})`);
  }
  for (const c of node.children ?? []) assertNoLiteralNullText(c);
}

/** Compare trees with ±tol on rect components. */
export function diffTrees(
  a: GeometryNode,
  b: GeometryNode,
  tol = 2,
  path = "root",
): string[] {
  const errs: string[] = [];
  if (a.kind !== b.kind) errs.push(`${path}: kind ${a.kind} != ${b.kind}`);
  for (let i = 0; i < 4; i++) {
    if (Math.abs(a.rect[i] - b.rect[i]) > tol) {
      errs.push(`${path}: rect[${i}] ${a.rect[i]} vs ${b.rect[i]} (>${tol})`);
    }
  }
  if ((a.text ?? null) !== (b.text ?? null)) {
    errs.push(`${path}: text ${JSON.stringify(a.text)} != ${JSON.stringify(b.text)}`);
  }
  const ac = a.children ?? [];
  const bc = b.children ?? [];
  if (ac.length !== bc.length) {
    errs.push(`${path}: children ${ac.length} != ${bc.length}`);
  }
  const n = Math.min(ac.length, bc.length);
  for (let i = 0; i < n; i++) {
    errs.push(...diffTrees(ac[i], bc[i], tol, `${path}/${ac[i].kind}[${i}]`));
  }
  return errs;
}

export const UPDATE_SNAPSHOTS =
  process.env.UPDATE_SNAPSHOTS === "1" || process.argv.includes("--update-snapshots");
