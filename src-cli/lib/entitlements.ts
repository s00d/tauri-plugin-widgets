import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import type { TauriConf } from "./tauri-conf.js";

export interface EntitlementsOptions {
  appGroup: string;
  sandbox?: boolean;
}

function assertSafeAppGroup(appGroup: string): void {
  // XML 1.0 forbids most C0 controls in CharData; also reject markup delimiters.
  if (/[\x00-\x08\x0B\x0C\x0E-\x1F<>&"']/.test(appGroup)) {
    throw new Error(`appGroup contains XML-unsafe characters: ${appGroup}`);
  }
}

/** Escape text embedded in XML plist `<string>` nodes. */
export function xmlEscapePlist(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}

export function entitlementsPlist({ appGroup, sandbox = false }: EntitlementsOptions): string {
  assertSafeAppGroup(appGroup);
  const escaped = xmlEscapePlist(appGroup);
  const lines = [
    `<?xml version="1.0" encoding="UTF-8"?>`,
    `<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">`,
    `<plist version="1.0">`,
    `<dict>`,
  ];
  if (sandbox) {
    lines.push(`    <key>com.apple.security.app-sandbox</key>`, `    <true/>`);
  }
  lines.push(
    `    <key>com.apple.security.application-groups</key>`,
    `    <array>`,
    `        <string>${escaped}</string>`,
    `    </array>`,
    `</dict>`,
    `</plist>`,
    ``,
  );
  return lines.join("\n");
}

export function writeEntitlementsFile(dest: string, opts: EntitlementsOptions): void {
  mkdirSync(dirname(dest), { recursive: true });
  writeFileSync(dest, entitlementsPlist(opts), "utf-8");
}

export interface SyncMacosEntitlementsResult {
  wrote: boolean;
  reason?: string;
  appGroup?: string;
  dir?: string;
}

/** Regenerate macOS-widget entitlements from plugins.widgets.appGroup. */
export function syncMacosEntitlementsFromConf(
  cwd: string,
  conf: TauriConf | null,
): SyncMacosEntitlementsResult {
  const appGroup = conf?.data?.plugins?.widgets?.appGroup;
  if (!appGroup) return { wrote: false, reason: "no appGroup" };
  const widgetDir = join(cwd, "src-tauri", "macos-widget");
  if (!existsSync(widgetDir)) return { wrote: false, reason: "no macos-widget" };
  writeEntitlementsFile(join(widgetDir, "App.entitlements"), { appGroup, sandbox: false });
  writeEntitlementsFile(join(widgetDir, "TauriWidgetExtension.entitlements"), {
    appGroup,
    sandbox: true,
  });
  return { wrote: true, appGroup, dir: widgetDir };
}

export function findEntitlementsFiles(cwd: string): string[] {
  // Walk src-tauri once (covers macos-widget + gen/apple). Dedupe by real path.
  const root = join(cwd, "src-tauri");
  const seen = new Set<string>();
  const out: string[] = [];
  if (!existsSync(root)) return out;
  const walk = (dir: string, depth = 0) => {
    if (depth > 6) return;
    let entries: import("node:fs").Dirent[] = [];
    try {
      entries = readdirSync(dir, { withFileTypes: true });
    } catch {
      return;
    }
    for (const e of entries) {
      if (e.name === "node_modules" || e.name === "target" || e.name.startsWith(".")) continue;
      const p = join(dir, e.name);
      if (e.isDirectory()) {
        walk(p, depth + 1);
        continue;
      }
      if (!e.name.endsWith(".entitlements")) continue;
      let key = p;
      try {
        key = resolve(p);
      } catch {
        /* ignore */
      }
      if (seen.has(key)) continue;
      seen.add(key);
      out.push(p);
    }
  };
  walk(root);
  return out;
}

export function extractAppGroups(filePath: string): string[] {
  try {
    const raw = readFileSync(filePath, "utf-8");
    const groups: string[] = [];
    const re = /group\.[A-Za-z0-9._-]+/g;
    let m: RegExpExecArray | null;
    while ((m = re.exec(raw))) groups.push(m[0]);
    return [...new Set(groups)];
  } catch {
    return [];
  }
}

export interface AppGroupMentions {
  hits: string[];
}

/** Collect App Group ids from Info.plist / pbxproj / entitlements under src-tauri. */
export function findAppGroupMentions(cwd: string, appGroup: string | null | undefined): AppGroupMentions {
  if (!appGroup) return { hits: [] };
  const root = join(cwd, "src-tauri");
  const hits: string[] = [];
  const walk = (dir: string, depth = 0) => {
    if (depth > 7 || !existsSync(dir)) return;
    let entries: import("node:fs").Dirent[] = [];
    try {
      entries = readdirSync(dir, { withFileTypes: true });
    } catch {
      return;
    }
    for (const e of entries) {
      if (e.name === "node_modules" || e.name === "target" || e.name.startsWith(".")) continue;
      const p = join(dir, e.name);
      if (e.isDirectory()) {
        walk(p, depth + 1);
        continue;
      }
      const lower = e.name.toLowerCase();
      const interesting =
        lower.endsWith(".entitlements") ||
        lower.endsWith(".plist") ||
        lower.endsWith(".pbxproj") ||
        lower.endsWith(".swift") ||
        lower === "tauri.conf.json";
      if (!interesting) continue;
      try {
        const raw = readFileSync(p, "utf-8");
        if (raw.includes(appGroup)) {
          hits.push(p.startsWith(cwd) ? p.slice(cwd.length + 1) : p);
        }
      } catch {
        /* ignore */
      }
    }
  };
  walk(root);
  return { hits };
}
