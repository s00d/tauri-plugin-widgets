import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { pluginRoot } from "./paths.js";

export interface CapabilityCell {
  support: "full" | "degraded" | "unsupported" | string;
  note?: string;
}

export interface Capabilities {
  version?: number;
  elements?: Record<string, Record<string, CapabilityCell>>;
  features?: Record<string, Record<string, CapabilityCell>>;
  [key: string]: unknown;
}

export interface LoadedCapabilities {
  path: string;
  data: Capabilities;
}

export function loadCapabilities(): LoadedCapabilities {
  const root = pluginRoot();
  const candidates = [
    join(root, "schemas", "capabilities.json"),
    join(root, "docs", "public", "schemas", "capabilities.json"),
  ];
  for (const p of candidates) {
    if (!existsSync(p)) continue;
    return { path: p, data: JSON.parse(readFileSync(p, "utf-8")) as Capabilities };
  }
  throw new Error(
    "schemas/capabilities.json missing — run: cargo test --lib capabilities::write_docs::capabilities_json_matches",
  );
}

export interface ConfigFinding {
  level: "error" | "warn";
  path: string;
  type: string;
  platform: string;
  support: string;
  note?: string;
}

interface WidgetNode {
  type?: string;
  url?: string;
  children?: WidgetNode[];
  items?: WidgetNode[];
  content?: WidgetNode;
  [key: string]: unknown;
}

function supportCell(caps: Capabilities, type: string, platform: string): CapabilityCell {
  const el = caps.elements?.[type] || caps.features?.[type];
  if (!el) return { support: "unsupported", note: "unknown element" };
  return el[platform] || { support: "unsupported", note: "unknown platform" };
}

function walk(
  node: WidgetNode | null | undefined,
  path: string,
  platform: string,
  caps: Capabilities,
  out: ConfigFinding[],
): void {
  if (!node || typeof node !== "object") return;
  const type = node.type;
  if (typeof type === "string") {
    const cell = supportCell(caps, type, platform);
    if (cell.support === "unsupported") {
      out.push({ level: "error", path, type, platform, support: cell.support, note: cell.note });
    } else if (cell.support === "degraded") {
      out.push({ level: "warn", path, type, platform, support: cell.support, note: cell.note });
    }
    // Feature keys
    if (type === "image" && node.url) {
      const f = supportCell(caps, "image.url", platform);
      if (f.support !== "full") {
        out.push({
          level: f.support === "unsupported" ? "error" : "warn",
          path: `${path}.url`,
          type: "image.url",
          platform,
          support: f.support,
          note: f.note,
        });
      }
    }
    if (type === "timer") {
      const f = supportCell(caps, "timer.live", platform);
      if (f.support !== "full") {
        out.push({
          level: f.support === "unsupported" ? "error" : "warn",
          path: `${path} (timer.live)`,
          type: "timer.live",
          platform,
          support: f.support,
          note: f.note,
        });
      }
    }
    if (type === "canvas") {
      const f = supportCell(caps, "canvas.path", platform);
      if (f.support !== "full") {
        out.push({
          level: f.support === "unsupported" ? "error" : "warn",
          path: `${path} (canvas.path)`,
          type: "canvas.path",
          platform,
          support: f.support,
          note: f.note,
        });
      }
    }
  }
  // List `items` are data rows ({text,...}), not element nodes — only recurse
  // explicit `children`. Matches walkShape's list special-case.
  const kids = type === "list" ? node.children || [] : node.children || node.items || [];
  if (Array.isArray(kids)) {
    kids.forEach((ch, i) => {
      const childPath = `${path}/${type || "node"}[${i}]`;
      if (!ch || typeof ch !== "object" || Array.isArray(ch)) {
        out.push({
          level: "error",
          path: childPath,
          type: typeof type === "string" ? type : "",
          platform,
          support: "invalid",
          note: "child must be an element object with type",
        });
        return;
      }
      if (typeof ch.type !== "string") {
        out.push({
          level: "error",
          path: childPath,
          type: "",
          platform,
          support: "invalid",
          note: "child must have string type",
        });
        return;
      }
      walk(ch, childPath, platform, caps, out);
    });
  }
  if (node.content && typeof node.content === "object" && node.content.type) {
    walk(node.content, `${path}/content`, platform, caps, out);
  }
}

/**
 * Structural validation of widget config shape (sizes, required fields per type).
 */
export function validateWidgetShape(config: unknown): ConfigFinding[] {
  const findings: ConfigFinding[] = [];
  if (!config || typeof config !== "object") {
    return [
      {
        level: "error",
        path: "",
        type: "",
        platform: "schema",
        support: "invalid",
        note: "config is not an object",
      },
    ];
  }
  const { $schema: _schema, ...body } = config as Record<string, unknown>;
  const sizes = ["small", "medium", "large"];
  let hasSize = false;
  for (const size of sizes) {
    if (!body[size]) continue;
    hasSize = true;
    const node = body[size];
    if (!node || typeof node !== "object" || Array.isArray(node)) {
      findings.push({
        level: "error",
        path: size,
        type: "",
        platform: "schema",
        support: "invalid",
        note: `${size} must be an element object with type`,
      });
      continue;
    }
    if (typeof (node as WidgetNode).type !== "string") {
      findings.push({
        level: "error",
        path: size,
        type: "",
        platform: "schema",
        support: "invalid",
        note: `${size} must have string type`,
      });
      continue;
    }
    walkShape(node as WidgetNode, size, findings);
  }
  if (!hasSize) {
    findings.push({
      level: "error",
      path: "",
      type: "",
      platform: "schema",
      support: "invalid",
      note: "expected at least one of small/medium/large",
    });
  }
  return findings;
}

function walkShape(node: WidgetNode | null | undefined, path: string, out: ConfigFinding[]): void {
  if (!node || typeof node !== "object") return;
  const type = node.type;
  if (typeof type === "string") {
    if (type === "text") {
      const content = node.content;
      if (typeof content !== "string") {
        out.push({
          level: "error",
          path,
          type,
          platform: "schema",
          support: "invalid",
          note: 'type "text" requires string content',
        });
      }
    }
    if (type === "image") {
      const hasUrl = typeof node.url === "string" && node.url.length > 0;
      const hasSystem = typeof node.systemName === "string" && node.systemName.length > 0;
      const hasData = typeof node.data === "string" && node.data.length > 0;
      if (!hasUrl && !hasSystem && !hasData) {
        out.push({
          level: "error",
          path,
          type,
          platform: "schema",
          support: "invalid",
          note: 'type "image" requires non-empty string url, systemName, or data',
        });
      }
    }
    if (type === "list") {
      if (!Array.isArray(node.items)) {
        out.push({
          level: "error",
          path,
          type,
          platform: "schema",
          support: "invalid",
          note: 'type "list" requires items array',
        });
      } else {
        node.items.forEach((item, i) => {
          if (!item || typeof item !== "object" || Array.isArray(item)) {
            out.push({
              level: "error",
              path: `${path}/items[${i}]`,
              type,
              platform: "schema",
              support: "invalid",
              note: "list item must be an object with string text",
            });
          } else if (typeof (item as WidgetNode).text !== "string") {
            out.push({
              level: "error",
              path: `${path}/items[${i}]`,
              type,
              platform: "schema",
              support: "invalid",
              note: "list item requires string text",
            });
          }
        });
      }
    }
  }
  const kids = type === "list" ? node.children || [] : node.children || node.items || [];
  if (Array.isArray(kids)) {
    kids.forEach((ch, i) => {
      const childPath = `${path}/${type || "node"}[${i}]`;
      if (!ch || typeof ch !== "object" || Array.isArray(ch)) {
        out.push({
          level: "error",
          path: childPath,
          type: typeof type === "string" ? type : "",
          platform: "schema",
          support: "invalid",
          note: "child must be an element object with type",
        });
        return;
      }
      if (typeof (ch as WidgetNode).type !== "string") {
        out.push({
          level: "error",
          path: childPath,
          type: "",
          platform: "schema",
          support: "invalid",
          note: "child must have string type",
        });
        return;
      }
      walkShape(ch as WidgetNode, childPath, out);
    });
  }
  if (node.content && typeof node.content === "object" && (node.content as WidgetNode).type) {
    walkShape(node.content as WidgetNode, `${path}/content`, out);
  }
}

/**
 * Validate widget config against capabilities.json for one or more platforms.
 */
export function validateWidgetConfig(
  config: unknown,
  platforms: string[],
  caps: Capabilities,
): ConfigFinding[] {
  const findings: ConfigFinding[] = validateWidgetShape(config);
  if (!config || typeof config !== "object") {
    return findings;
  }
  const { $schema: _schema, ...body } = config as Record<string, unknown>;
  const sizes = ["small", "medium", "large"];
  for (const size of sizes) {
    if (!body[size]) continue;
    for (const platform of platforms) {
      walk(body[size] as WidgetNode, size, platform, caps, findings);
    }
  }
  return findings;
}

export interface PluginWidgetsFinding {
  level: "error" | "warn";
  msg: string;
}

/** Shallow validate plugins.widgets against known enum / required fields (plugin-config.v1). */
export function validatePluginWidgets(widgets: unknown): PluginWidgetsFinding[] {
  const out: PluginWidgetsFinding[] = [];
  if (!widgets || typeof widgets !== "object") {
    out.push({ level: "error", msg: "plugins.widgets missing or not an object" });
    return out;
  }
  const w = widgets as Record<string, unknown>;
  const transports = new Set(["appGroup", "userDefaults", "widgetContainer", "auto"]);
  if (w.transport != null && !transports.has(w.transport as string)) {
    out.push({ level: "error", msg: `transport=${String(w.transport)} invalid` });
  }
  if (!w.appGroup || typeof w.appGroup !== "string") {
    out.push({ level: "error", msg: "appGroup is required (string)" });
  } else if (!w.appGroup.startsWith("group.")) {
    out.push({ level: "warn", msg: `appGroup "${w.appGroup}" does not start with group.` });
  }
  if (w.extensionBundleId != null && typeof w.extensionBundleId !== "string") {
    out.push({ level: "error", msg: "extensionBundleId must be a string" });
  }
  return out;
}
