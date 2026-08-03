import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import type { TauriConf } from "./tauri-conf.js";
import { findApplePbxproj } from "./apple.js";

export interface EntitlementsOptions {
  appGroup: string;
  sandbox?: boolean;
}

export function entitlementsPlist({ appGroup, sandbox = false }: EntitlementsOptions): string {
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
    `        <string>${appGroup}</string>`,
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

export interface IosEntitlementsRels {
  appEntRel: string;
  widgetEntRel: string;
}

/**
 * Resolve iOS entitlements relative paths under gen/apple.
 * Prefer existing CODE_SIGN_ENTITLEMENTS / on-disk files; never invent "ExtensionExtensionExtension".
 */
export function resolveIosEntitlementsRels(
  cwd: string,
  appleDir: string,
  widgetTargets: string[],
): IosEntitlementsRels {
  let appEntRel: string | null = null;
  let widgetEntRel: string | null = null;

  for (const pbx of findApplePbxproj(cwd)) {
    const raw = readFileSync(pbx, "utf-8");
    const matches = [...raw.matchAll(/CODE_SIGN_ENTITLEMENTS = "?([^";]+\.entitlements)"?/g)].map(
      (m) => m[1].replace(/^"/, "").replace(/"$/, ""),
    );
    if (!appEntRel) {
      appEntRel = matches.find((p) => /_iOS\//.test(p) || /(^|\/)App\.entitlements$/.test(p)) || null;
    }
    if (!widgetEntRel) {
      widgetEntRel =
        matches.find((p) => /Widget|Extension/i.test(p) && !/_iOS\//.test(p)) || null;
    }
  }

  if (!appEntRel) {
    const iosDirs = readdirSync(appleDir, { withFileTypes: true }).filter(
      (e) => e.isDirectory() && e.name.endsWith("_iOS"),
    );
    if (iosDirs[0]) appEntRel = `${iosDirs[0].name}/${iosDirs[0].name}.entitlements`;
    else appEntRel = "App.entitlements";
  }

  if (!widgetEntRel) {
    // Prefer an existing *Extension*.entitlements on disk
    const onDisk: string[] = [];
    for (const e of readdirSync(appleDir, { withFileTypes: true })) {
      if (e.isFile() && /\.entitlements$/.test(e.name) && /Extension|Widget/i.test(e.name)) {
        onDisk.push(e.name);
      }
    }
    if (onDisk[0]) {
      widgetEntRel = onDisk[0];
    } else {
      const widgetName = widgetTargets[0] || "WidgetExtension";
      // Target is often already named WidgetExtensionExtension — do not append another "Extension"
      widgetEntRel = /Extension$/i.test(widgetName)
        ? `${widgetName}.entitlements`
        : `${widgetName}Extension.entitlements`;
    }
  }

  return { appEntRel, widgetEntRel };
}
