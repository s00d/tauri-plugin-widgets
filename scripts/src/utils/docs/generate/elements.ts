// @ts-nocheck
import { join } from "node:path";
import { cell, code, table } from "../../markdown.js";
import {
  CORE_ELEMENTS,
  ELEMENT_GROUPS,
  EXTENDED_ELEMENTS,
  extractElementsFromSchema,
  loadCase,
  loadFixture,
  loadSchema,
  writeBlock,
} from "./shared.js";
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
