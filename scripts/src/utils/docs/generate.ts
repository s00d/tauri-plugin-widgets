// @ts-nocheck — large schema walk; typed gradually
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative } from "node:path";
import {
  BLOCK_START,
  cell,
  code,
  replaceBlock,
  table,
} from "../markdown.js";
import { repoRoot } from "../workspace.js";

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

function schemaTypeLabel(prop) {
  if (!prop) return "unknown";
  if (prop.$ref) return prop.$ref.replace("#/definitions/", "");
  if (prop.anyOf) {
    const refs = prop.anyOf
      .filter((x) => x && x.type !== "null")
      .map((x) => schemaTypeLabel(x));
    return refs.filter(Boolean).join(" | ") || "any";
  }
  if (Array.isArray(prop.type)) {
    return prop.type.filter((t) => t !== "null").join(" | ") || "any";
  }
  if (prop.type === "array") {
    const items = prop.items ? schemaTypeLabel(prop.items) : "any";
    return `${items}[]`;
  }
  if (prop.enum) return prop.enum.map((e) => JSON.stringify(e)).join(" | ");
  return prop.type || "any";
}

function defaultLabel(prop) {
  if (!prop || prop.default === undefined) return "—";
  return JSON.stringify(prop.default);
}

function tierOf(name) {
  if (CORE_ELEMENTS.includes(name)) return "core";
  if (EXTENDED_ELEMENTS.includes(name)) return "extended";
  return "other";
}

function groupHref(name) {
  for (const g of Object.values(ELEMENT_GROUPS)) {
    if (g.types.includes(name)) return `/elements/${g.file.split("/").pop().replace(".md", "")}#el-${name}`;
  }
  return "/elements/";
}

function minimalExample(name) {
  const examples = {
    vstack: {
      type: "vstack",
      spacing: 8,
      padding: 14,
      cornerRadius: 16,
      background: { light: "#E8F4FD", dark: "#1a1a2e" },
      children: [
        { type: "text", content: "72°", textStyle: "largeTitle", fontWeight: "bold", color: "label" },
        { type: "text", content: "Sunny", fontSize: 14, color: "secondaryLabel" },
      ],
    },
    hstack: {
      type: "hstack",
      spacing: 8,
      alignment: "center",
      children: [
        { type: "image", systemName: "cloud.sun.fill", size: 28, color: "#ffcc00" },
        { type: "text", content: "72°", fontSize: 28, fontWeight: "bold", color: "label" },
      ],
    },
    zstack: {
      type: "zstack",
      alignment: "center",
      children: [
        { type: "shape", shapeType: "circle", fill: "#1e293b", size: 64 },
        { type: "text", content: "OK", fontWeight: "bold", color: "#fff" },
      ],
    },
    grid: {
      type: "grid",
      columns: 2,
      spacing: 6,
      children: [
        { type: "button", label: "7", action: "digit", payload: "7" },
        { type: "button", label: "8", action: "digit", payload: "8" },
        { type: "button", label: "9", action: "digit", payload: "9" },
        { type: "button", label: "÷", action: "op", payload: "/" },
      ],
    },
    container: {
      type: "container",
      contentAlignment: "center",
      padding: 12,
      cornerRadius: 14,
      background: "#0f172a",
      children: [{ type: "text", content: "Badge", color: "#38bdf8", fontWeight: "semibold" }],
    },
    text: {
      type: "text",
      content: "Hello",
      fontSize: 16,
      fontWeight: "semibold",
      color: { light: "#0f172a", dark: "#f8fafc" },
      alignment: "leading",
      lineLimit: 2,
    },
    label: {
      type: "label",
      text: "Inbox",
      systemName: "envelope.fill",
      iconColor: "#38bdf8",
      fontSize: 15,
      fontWeight: "medium",
      spacing: 6,
    },
    date: {
      type: "date",
      date: "2026-03-01T10:00:00Z",
      dateStyle: "relative",
      fontSize: 14,
      color: "secondaryLabel",
    },
    timer: {
      type: "timer",
      targetDate: "2026-12-31T23:59:59Z",
      counting: "down",
      fontSize: 22,
      fontWeight: "bold",
      color: "#22c55e",
    },
    image: {
      type: "image",
      systemName: "cloud.sun.fill",
      size: 28,
      color: "#ffcc00",
      contentMode: "fit",
    },
    shape: {
      type: "shape",
      shapeType: "capsule",
      fill: "#ef4444",
      size: 12,
      stroke: "#fff",
      strokeWidth: 1,
    },
    canvas: {
      type: "canvas",
      width: 80,
      height: 80,
      background: "#0f172a",
      cornerRadius: 12,
      elements: [
        { draw: "circle", cx: 40, cy: 40, r: 28, fill: "#1e293b", stroke: "#38bdf8", strokeWidth: 2 },
        { draw: "text", x: 40, y: 44, content: "72", fontSize: 18, color: "#fff", anchor: "middle" },
      ],
    },
    progress: {
      type: "progress",
      value: 0.7,
      total: 1,
      tint: "#4CAF50",
      color: "secondaryLabel",
      label: "Humidity",
      barStyle: "linear",
    },
    gauge: {
      type: "gauge",
      value: 0.72,
      min: 0,
      max: 1,
      label: "CPU",
      currentValueLabel: "72%",
      tint: "#38bdf8",
      gaugeStyle: "circular",
    },
    chart: {
      type: "chart",
      chartType: "bar",
      tint: "#38bdf8",
      chartData: [
        { label: "Mon", value: 120 },
        { label: "Tue", value: 90 },
        { label: "Wed", value: 150 },
      ],
    },
    list: {
      type: "list",
      spacing: 4,
      fontSize: 14,
      items: [
        { text: "Alpha", checked: true, action: "row", payload: "a" },
        { text: "Beta", checked: false, action: "row", payload: "b" },
        { text: "Gamma" },
      ],
    },
    button: {
      type: "button",
      label: "Refresh",
      action: "refresh",
      color: "#fff",
      backgroundColor: "#0284c7",
      fontSize: 14,
      cornerRadius: 10,
    },
    toggle: {
      type: "toggle",
      isOn: true,
      label: "Dark mode",
      tint: "#38bdf8",
      action: "toggle-dark",
    },
    link: {
      type: "link",
      action: "open-details",
      payload: "item-1",
      children: [
        {
          type: "hstack",
          spacing: 6,
          alignment: "center",
          children: [
            { type: "text", content: "Details", color: "#38bdf8", fontWeight: "medium" },
            { type: "image", systemName: "chevron.right", size: 12, color: "#38bdf8" },
          ],
        },
      ],
    },
    spacer: { type: "spacer", minLength: 8 },
    divider: { type: "divider", thickness: 1, color: "separator" },
  };
  return examples[name] ?? { type: name };
}

function stripNullDeep(value) {
  if (Array.isArray(value)) return value.map(stripNullDeep);
  if (!value || typeof value !== "object") return value;
  /** @type {Record<string, unknown>} */
  const out = {};
  for (const [k, v] of Object.entries(value)) {
    if (v === null || v === undefined) continue;
    out[k] = stripNullDeep(v);
  }
  return out;
}

function findNodesOfType(node, type, out = []) {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const item of node) findNodesOfType(item, type, out);
    return out;
  }
  if (node.type === type) out.push(node);
  for (const [k, v] of Object.entries(node)) {
    if (k === "type") continue;
    if (v && typeof v === "object") findNodesOfType(v, type, out);
  }
  return out;
}

function compactExampleNode(node, type) {
  let n = stripNullDeep(structuredClone(node));
  if (type === "list" && Array.isArray(n.items) && n.items.length > 4) {
    n.items = n.items.slice(0, 4);
  }
  if (type === "canvas" && Array.isArray(n.elements) && n.elements.length > 6) {
    n.elements = n.elements.slice(0, 6);
  }
  if (type === "chart" && Array.isArray(n.chartData) && n.chartData.length > 6) {
    n.chartData = n.chartData.slice(0, 6);
  }

  const trimTree = (el, depth) => {
    if (!el || typeof el !== "object" || Array.isArray(el)) return el;
    if (!Array.isArray(el.children)) return el;
    const max = depth === 0 ? 3 : 2;
    const total = el.children.length;
    let kids = el.children.slice(0, max).map((c) => trimTree(c, depth + 1));
    if (total > max) {
      kids.push({
        type: "text",
        content: `… +${total - max} more`,
        fontSize: 11,
        color: "secondaryLabel",
      });
    }
    return { ...el, children: kids };
  };

  if (["vstack", "hstack", "zstack", "grid", "container", "link"].includes(type)) {
    n = trimTree(n, 0);
    // Hard cap so nested dashboards don't blow up the page.
    if (JSON.stringify(n).length > 1600 && Array.isArray(n.children)) {
      n = {
        ...n,
        children: [
          ...n.children.slice(0, 2),
          {
            type: "text",
            content: "… truncated for docs",
            fontSize: 11,
            color: "secondaryLabel",
          },
        ],
      };
    }
  }
  return n;
}

function signatureOf(node) {
  try {
    return JSON.stringify(node);
  } catch {
    return String(node);
  }
}

/**
 * Collect real JSON snippets from visual fixtures for an element type.
 * @returns {{ title: string, json: unknown }[]}
 */
function fixtureExamples(elementName, cases) {
  /** @type {{ title: string, json: unknown, sig: string }[]} */
  const picked = [];
  const seen = new Set();

  const push = (title, node) => {
    if (!node) return;
    const compact = compactExampleNode(node, elementName);
    const sig = signatureOf(compact);
    if (seen.has(sig)) return;
    seen.add(sig);
    picked.push({ title, json: compact, sig });
  };

  for (const caseName of cases.slice(0, 12)) {
    let c;
    let cfg;
    try {
      c = loadCase(caseName);
      cfg = loadFixture(c.fixture);
    } catch {
      continue;
    }
    const size = c.size || "small";
    const root = cfg[size] || cfg.small || cfg.medium || cfg.large;
    if (!root) continue;

    if (root.type === elementName) {
      push(`From showcase \`${caseName}\` (size root)`, root);
    } else {
      const nodes = findNodesOfType(root, elementName);
      if (nodes[0]) push(`From showcase \`${caseName}\``, nodes[0]);
    }
    if (picked.length >= 2) break;
  }

  return picked.map(({ title, json }) => ({ title, json }));
}

function formatExamplesMarkdown(elementName, usageCases) {
  /** @type {{ title: string, json: unknown }[]} */
  const blocks = [{ title: "Minimal", json: minimalExample(elementName) }];

  for (const fx of fixtureExamples(elementName, usageCases)) {
    // Skip fixture clone of minimal
    if (signatureOf(fx.json) === signatureOf(blocks[0].json)) continue;
    blocks.push(fx);
  }

  // Always show how it sits in a WidgetConfig size family.
  blocks.push({
    title: "Inside a `WidgetConfig`",
    json: { small: minimalExample(elementName) },
  });

  return blocks
    .map(
      (b) => `#### ${b.title}

\`\`\`json
${JSON.stringify(b.json, null, 2)}
\`\`\``,
    )
    .join("\n\n");
}

function firstShotCase(cases) {
  return cases?.[0] ?? null;
}

export function copyShots(report, check) {
  const dest = join(PUBLIC, "shots");
  if (existsSync(dest)) rmSync(dest, { recursive: true, force: true });
  mkdirSync(dest, { recursive: true });
  for (const platform of ALL_PLATFORMS) {
    const src = join(ROOT, "tests/golden", platform);
    if (!existsSync(src)) continue;
    for (const file of readdirSync(src)) {
      if (!file.endsWith(".png")) continue;
      const to = join(dest, platform, file);
      mkdirSync(dirname(to), { recursive: true });
      copyFileSync(join(src, file), to);
      report.written.push(relative(ROOT, to));
    }
  }
  void check;
}

export function copySandbox(report, check) {
  const widget = readFileSync(join(ROOT, "widget.html"), "utf8");
  const shim = `<script>
(function(){
  var pendingConfig = null;
  var callbacks = {};
  var nextId = 1;
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: 'docs-sandbox' } },
    transformCallback: function(cb) {
      var id = nextId++;
      callbacks[id] = cb;
      return id;
    },
    invoke: function(cmd, args) {
      if (cmd === 'plugin:widgets|get_widget_config') {
        return Promise.resolve(pendingConfig || { small: { type: 'text', content: 'Waiting for config…', color: '#fff' } });
      }
      if (cmd === 'plugin:widgets|widget_action') {
        console.log('[sandbox] action', args);
        return Promise.resolve();
      }
      if (cmd === 'plugin:widgets|close_widget_window') return Promise.resolve();
      if (cmd === 'plugin:event|listen') return Promise.resolve();
      if (cmd.indexOf('report_receipt') !== -1 || cmd.indexOf('report-receipt') !== -1) return Promise.resolve();
      return Promise.resolve(null);
    }
  };
  window.addEventListener('message', function(ev) {
    var data = ev.data;
    if (!data || data.type !== 'render') return;
    pendingConfig = data.config;
    if (typeof window.__WIDGET_SANDBOX_RENDER__ === 'function') {
      window.__WIDGET_SANDBOX_RENDER__(pendingConfig);
    } else {
      window.__WIDGET_SANDBOX_PENDING__ = pendingConfig;
    }
  });
})();
</script>`;

  let html = widget.replace(
    "<script>\n(function(){",
    `${shim}\n<script>\n(function(){`,
  );
  html = html.replace(
    "function render(cfg, source){",
    "window.__WIDGET_SANDBOX_RENDER__=function(c){render(c,'sandbox')};\nif(window.__WIDGET_SANDBOX_PENDING__){var __p=window.__WIDGET_SANDBOX_PENDING__;window.__WIDGET_SANDBOX_PENDING__=null;setTimeout(function(){render(__p,'sandbox')},0);}\nfunction render(cfg, source){",
  );

  writeOrCheck(report, join(PUBLIC, "widget-sandbox.html"), html, check);
}

/** Group case names by preset base (strip .small/.medium/.large). */
export function groupCasesByPreset() {
  /** @type {Map<string, { name: string, case: any }[]>} */
  const groups = new Map();
  for (const name of listCases()) {
    const c = loadCase(name);
    const base = name.replace(/\.(small|medium|large)$/, "") || name;
    const list = groups.get(base) ?? [];
    list.push({ name, case: c });
    groups.set(base, list);
  }
  for (const list of groups.values()) {
    list.sort((a, b) => {
      const order = { small: 0, medium: 1, large: 2 };
      return (order[a.case.size] ?? 9) - (order[b.case.size] ?? 9);
    });
  }
  return groups;
}

export function genShowcase(report, check) {
  const groups = groupCasesByPreset();
  const parts = [];

  for (const [base, entries] of [...groups.entries()].sort((a, b) => a[0].localeCompare(b[0]))) {
    const sizes = entries.map((e) => e.case.size).join(", ");
    parts.push(`## ${base}\n`);
    parts.push(`Sizes: ${sizes}\n`);

    // Size tabs via ShotGrid for each case — show all size variants' desktop shot + platform tabs on primary
    for (const { name, case: c } of entries) {
      parts.push(`### ${c.size}\n`);
      parts.push(`<ShotGrid case="${name}" />\n`);
    }

    const primary = entries[0];
    let fixtureJson = "";
    try {
      fixtureJson = JSON.stringify(loadFixture(primary.case.fixture), null, 2);
    } catch (e) {
      fixtureJson = `/* ${e} */`;
    }

    writeOrCheck(
      report,
      join(PUBLIC, "gallery-data", `${primary.name}.json`),
      fixtureJson.endsWith("\n") ? fixtureJson : fixtureJson + "\n",
      check,
    );

    const b64 = Buffer.from(fixtureJson, "utf8").toString("base64");
    parts.push(`<details>\n<summary>Config JSON</summary>\n\n\`\`\`json\n${fixtureJson}\n\`\`\`\n\n</details>\n`);
    parts.push(
      `<Playground case="${primary.name}" size="${primary.case.size || "small"}" config-b64="${b64}" />\n`,
    );
  }

  writeBlock(report, "docs/showcase.md", "showcase", parts.join("\n"), check);
}

function elementSection(el, usage) {
  const cases = usage.get(el.name) ?? [];
  const shotCase = firstShotCase(cases);
  const propRows = Object.entries(el.properties)
    .filter(([k]) => k !== "type")
    .map(([name, prop]) => [
      code(name),
      code(schemaTypeLabel(prop)),
      code(defaultLabel(prop)),
      cell(prop.description || ""),
    ]);

  const examplesMd = formatExamplesMarkdown(el.name, cases);
  const shot = shotCase
    ? `![${el.name} (${shotCase})](/shots/desktop/${shotCase}.png)\n\n_From showcase preset — case \`${shotCase}\`._`
    : "_No screenshot case._";

  return `## \`${el.name}\` {#el-${el.name}}

**Tier:** ${tierOf(el.name)}

${el.description}

${shot}

### Properties

${propRows.length ? table(["Property", "Type", "Default", "Description"], propRows) : "_No additional properties._"}

### Examples

${examplesMd}
`;
}

export function genElements(report, check, usage) {
  const schema = loadSchema();
  const elements = extractElementsFromSchema(schema);
  const byName = new Map(elements.map((e) => [e.name, e]));

  for (const group of Object.values(ELEMENT_GROUPS)) {
    const body = group.types
      .map((t) => {
        const el = byName.get(t);
        if (!el) return `## \`${t}\`\n\n_Missing from schema._\n`;
        return elementSection(el, usage);
      })
      .join("\n");
    writeBlock(report, group.file, group.marker, body, check);
  }

  // Index cards
  const cards = elements
    .map((el) => {
      const cases = usage.get(el.name) ?? [];
      const shot = firstShotCase(cases);
      const href = groupHref(el.name);
      const img = shot ? `![${el.name}](/shots/desktop/${shot}.png)` : "";
      return `| [\`${el.name}\`](${href}) | ${tierOf(el.name)} | ${img} |`;
    })
    .join("\n");

  const indexBody = `Generated from the JSON schema. Prefer the thematic pages for choosing between siblings.

| Element | Tier | Preview |
| --- | --- | --- |
${cards}
`;
  writeBlock(report, "docs/elements/index.md", "elements-index", indexBody, check);
  return elements;
}

export function genPermissions(report, check) {
  const src = join(ROOT, "permissions/autogenerated/reference.md");
  const body = readFileSync(src, "utf8");
  writeBlock(report, "docs/api/permissions.md", "permissions", body, check);
}

export function genCapabilityMatrix(report, check) {
  const generated = join(DOCS, "guide", "_generated", "capability-matrix.md");
  // Prefer cargo-owned file if present; else keep marker empty-ish
  let body = "_Run `cargo test --lib capabilities::write_docs::capability_matrix_doc_matches` to refresh._";
  if (existsSync(generated)) {
    body = readFileSync(generated, "utf8")
      .replace(/^# .*\n+/, "")
      .replace(/^Generated from.*\n+/, "")
      .trim();
  }
  writeBlock(report, "docs/guide/tiers.md", "capability-matrix", body, check);
}

export function auditSchemaDocs(elements) {
  const errors = [];
  const enumBlurb = elements[0]?.rootDesc || "A UI element that can be a layout container or a leaf widget.";
  for (const el of elements) {
    if (!el.description || !el.description.trim()) {
      errors.push(`element \`${el.name}\` missing variant description in schema`);
    } else if (el.description.trim() === enumBlurb.trim()) {
      errors.push(`element \`${el.name}\` still uses enum-level description`);
    }
    for (const [name, prop] of Object.entries(el.properties)) {
      if (name === "type") continue;
      if (!prop.description || !String(prop.description).trim()) {
        errors.push(`\`${el.name}.${name}\` missing description`);
      }
    }
  }
  return errors;
}

export function auditCoverage(elementNames, usage) {
  const errors = [];
  for (const name of elementNames) {
    if ((usage.get(name) ?? []).length === 0) {
      errors.push(`element \`${name}\` has no fixture coverage`);
    }
  }
  for (const name of listCases()) {
    for (const platform of REQUIRED_PLATFORMS) {
      const png = join(ROOT, "tests/golden", platform, `${name}.png`);
      if (!existsSync(png)) {
        errors.push(`case \`${name}\` missing golden for ${platform}`);
      }
    }
  }
  return errors;
}

export async function runGenerate({ check = false, audit = true } = {}) {
  /** @type {Report} */
  const report = { written: [], stale: [] };
  mkdirSync(PUBLIC, { recursive: true });
  mkdirSync(join(DOCS, "guide", "_generated"), { recursive: true });

  const usage = collectElementUsage();
  copyShots(report, check);
  copySandbox(report, check);
  genShowcase(report, check);
  const elements = genElements(report, check, usage);
  genPermissions(report, check);
  genCapabilityMatrix(report, check);

  let auditErrors = [];
  if (audit) {
    auditErrors = [
      ...auditCoverage(
        elements.map((e) => e.name),
        usage,
      ),
      ...auditSchemaDocs(elements),
    ];
  }

  return { report, auditErrors, elementNames: elements.map((e) => e.name), usage };
}
