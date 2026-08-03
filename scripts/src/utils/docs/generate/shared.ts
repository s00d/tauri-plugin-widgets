// @ts-nocheck — large schema walk; typed gradually
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { join, relative, dirname } from "node:path";
import { replaceBlock } from "../../markdown.js";
import { repoRoot } from "../../workspace.js";

const ROOT = repoRoot();
export { ROOT };
export const DOCS = join(ROOT, "docs");
export const PUBLIC = join(DOCS, "public");

export const REQUIRED_PLATFORMS = ["desktop", "ios", "macos", "android", "linux"];
export const ALL_PLATFORMS = [...REQUIRED_PLATFORMS, "windows"];

export const CORE_ELEMENTS = [
  "vstack",
  "hstack",
  "zstack",
  "container",
  "grid",
  "text",
  "image",
  "spacer",
  "divider",
  "progress",
  "button",
  "link",
  "shape",
];

export const EXTENDED_ELEMENTS = [
  "gauge",
  "toggle",
  "date",
  "chart",
  "list",
  "timer",
  "canvas",
  "label",
];

/** @type {Record<string, { file: string; marker: string; types: string[] }>} */
export const ELEMENT_GROUPS = {
  layout: {
    file: "docs/elements/layout.md",
    marker: "elements-layout",
    types: ["vstack", "hstack", "zstack", "grid", "container"],
  },
  text: {
    file: "docs/elements/text.md",
    marker: "elements-text",
    types: ["text", "label", "date", "timer"],
  },
  media: {
    file: "docs/elements/media.md",
    marker: "elements-media",
    types: ["image", "shape", "canvas"],
  },
  data: {
    file: "docs/elements/data.md",
    marker: "elements-data",
    types: ["progress", "gauge", "chart", "list"],
  },
  interactive: {
    file: "docs/elements/interactive.md",
    marker: "elements-interactive",
    types: ["button", "toggle", "link"],
  },
  spacing: {
    file: "docs/elements/spacing.md",
    marker: "elements-spacing",
    types: ["spacer", "divider"],
  },
};

/** @typedef {{ written: string[], stale: string[] }} Report */

export function writeOrCheck(report, absPath, content, check, opts = {}) {
  const tracked = opts.tracked === true;
  const rel = relative(ROOT, absPath);
  mkdirSync(dirname(absPath), { recursive: true });
  if (check && tracked) {
    if (!existsSync(absPath) || readFileSync(absPath, "utf8") !== content) {
      report.stale.push(rel);
    }
    return;
  }
  writeFileSync(absPath, content, "utf8");
  report.written.push(rel);
}

export function writeBlock(report, relPath, id, body, check) {
  const abs = join(ROOT, relPath);
  if (!existsSync(abs)) {
    throw new Error(`missing shell page ${relPath} (need ${BLOCK_START(id)})`);
  }
  const next = replaceBlock(readFileSync(abs, "utf8"), id, body);
  writeOrCheck(report, abs, next, check, { tracked: true });
}

export function listCases() {
  const dir = join(ROOT, "tests/cases");
  return readdirSync(dir)
    .filter((f) => f.endsWith(".json"))
    .map((f) => f.replace(/\.json$/, ""))
    .sort();
}

export function loadCase(name) {
  return JSON.parse(readFileSync(join(ROOT, "tests/cases", `${name}.json`), "utf8"));
}

/** Platforms that must have a PNG golden for this case (default: all required). */
export function caseRequiredPlatforms(name) {
  const c = loadCase(name);
  if (Array.isArray(c.platforms) && c.platforms.length > 0) {
    return c.platforms.filter((p) => ALL_PLATFORMS.includes(p));
  }
  return [...REQUIRED_PLATFORMS];
}

export function resolveFixture(fixture) {
  const base = join(ROOT, "tests/fixtures");
  const direct = join(base, `${fixture}.json`);
  if (existsSync(direct)) return direct;
  throw new Error(`fixture not found: ${fixture}`);
}

export function loadFixture(fixture) {
  return JSON.parse(readFileSync(resolveFixture(fixture), "utf8"));
}

export function walkTypes(node, out = new Set()) {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const item of node) walkTypes(item, out);
    return out;
  }
  if (typeof node.type === "string") out.add(node.type);
  for (const [k, v] of Object.entries(node)) {
    if (k === "type") continue;
    if (v && typeof v === "object") walkTypes(v, out);
  }
  return out;
}

export function collectElementUsage() {
  /** @type {Map<string, string[]>} */
  const usage = new Map();
  for (const name of listCases()) {
    const c = loadCase(name);
    const cfg = loadFixture(c.fixture);
    for (const t of walkTypes(cfg)) {
      const list = usage.get(t) ?? [];
      list.push(name);
      usage.set(t, list);
    }
  }
  return usage;
}

export function loadSchema() {
  return JSON.parse(readFileSync(join(ROOT, "schemas/widget-config.v1.json"), "utf8"));
}

/**
 * @returns {{ name: string, description: string, properties: Record<string, any> }[]}
 */
export function extractElementsFromSchema(schema) {
  const rootDesc = schema?.definitions?.WidgetElement?.description || "";
  const oneOf = schema?.definitions?.WidgetElement?.oneOf;
  if (!Array.isArray(oneOf)) throw new Error("schema missing WidgetElement.oneOf");
  const elements = [];
  for (const variant of oneOf) {
    const name = variant?.properties?.type?.enum?.[0];
    if (!name) continue;
    const description = variant.description || "";
    elements.push({
      name,
      description,
      rootDesc,
      properties: variant.properties || {},
    });
  }
  return elements.sort((a, b) => a.name.localeCompare(b.name));
}
