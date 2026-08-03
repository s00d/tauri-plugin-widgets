import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export interface PluginWidgetsConfig {
  appGroup?: string;
  transport?: string;
  extensionBundleId?: string;
  [key: string]: unknown;
}

export interface TauriConfData {
  identifier?: string;
  productName?: string;
  build?: {
    beforeBundleCommand?: string;
    [key: string]: unknown;
  };
  bundle?: {
    active?: string[];
    targets?: string[] | "all";
    macOS?: {
      entitlements?: string;
      files?: Record<string, string>;
      [key: string]: unknown;
    };
    [key: string]: unknown;
  };
  plugins?: {
    widgets?: PluginWidgetsConfig;
    [key: string]: unknown;
  };
  [key: string]: unknown;
}

export interface TauriConf {
  path: string;
  data: TauriConfData;
}

export function findTauriConf(cwd: string): string | null {
  const candidates = [join(cwd, "src-tauri", "tauri.conf.json"), join(cwd, "tauri.conf.json")];
  for (const c of candidates) {
    if (existsSync(c)) return c;
  }
  return null;
}

export function readTauriConf(cwd: string): TauriConf | null {
  const confPath = findTauriConf(cwd);
  if (!confPath) return null;
  try {
    return { path: confPath, data: JSON.parse(readFileSync(confPath, "utf-8")) as TauriConfData };
  } catch {
    return null;
  }
}

export function detectTauriIdentifier(conf: TauriConf | null): string | null {
  const id = String(conf?.data?.identifier || "").trim();
  return id || null;
}

export function writeTauriConf(conf: TauriConf): void {
  writeFileSync(conf.path, JSON.stringify(conf.data, null, 2) + "\n", "utf-8");
}

export interface EnsurePluginsWidgetsOptions {
  appGroup?: string;
  transport?: string;
  extensionBundleId?: string;
  force?: boolean;
}

export interface EnsurePluginsWidgetsResult {
  wrote: boolean;
  transport?: string;
  skipped: boolean;
}

/**
 * Ensure plugins.widgets exists with appGroup + transport (+ optional extensionBundleId).
 */
export function ensurePluginsWidgets(
  conf: TauriConf | null,
  { appGroup, transport, extensionBundleId, force = false }: EnsurePluginsWidgetsOptions = {},
): EnsurePluginsWidgetsResult {
  if (!conf?.data || !conf?.path) return { wrote: false, skipped: true };
  if (!appGroup) return { wrote: false, skipped: true };
  const data = conf.data;
  if (!data.plugins) data.plugins = {};
  if (!data.plugins.widgets) data.plugins.widgets = {};
  const w = data.plugins.widgets;
  let changed = false;

  if (!w.appGroup || force) {
    if (w.appGroup !== appGroup) {
      w.appGroup = appGroup;
      changed = true;
    }
  }
  if (!w.transport || force) {
    if (w.transport !== transport) {
      w.transport = transport;
      changed = true;
    }
  }
  if (extensionBundleId && (!w.extensionBundleId || force)) {
    if (w.extensionBundleId !== extensionBundleId) {
      w.extensionBundleId = extensionBundleId;
      changed = true;
    }
  }

  if (changed) {
    writeTauriConf(conf);
  }
  return { wrote: changed, transport: w.transport, skipped: false };
}
