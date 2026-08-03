import { existsSync, readdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { execSync } from "node:child_process";
import { loadTraceFile } from "./trace.js";

const SAFE_SEGMENT = /^[A-Za-z0-9._-]+$/;

export function validateSafeSegment(value: string, label: string): void {
  if (value === "." || value.includes("..") || value.includes("/") || value.includes("\\")) {
    throw new Error(`${label} contains unsafe path characters`);
  }
  if (!SAFE_SEGMENT.test(value)) {
    throw new Error(`${label} must match ${SAFE_SEGMENT}`);
  }
}

function resolveMacGroupContainer(group: string): string | null {
  if (process.platform !== "darwin") return null;
  try {
    const escaped = group.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
    const out = execSync(
      `swift -e 'import Foundation; print(FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: "${escaped}")?.path ?? "")'`,
      { encoding: "utf-8" },
    ).trim();
    return out || null;
  } catch {
    return null;
  }
}

function cacheBase(): string {
  if (process.platform === "darwin" && process.env.HOME) {
    return join(process.env.HOME, "Library", "Caches");
  }
  if (process.platform === "win32" && process.env.LOCALAPPDATA) {
    return process.env.LOCALAPPDATA;
  }
  return process.env.XDG_CACHE_HOME || join(process.env.HOME || "/tmp", ".cache");
}

export function prefetchCacheDir(): string {
  return join(cacheBase(), "tauri-plugin-widgets", "image-prefetch");
}

function rmDir(path: string, label: string): boolean {
  if (!existsSync(path)) {
    console.log(`  – ${label}: not present (${path})`);
    return false;
  }
  rmSync(path, { recursive: true, force: true });
  console.log(`  ✓ ${label}: removed ${path}`);
  return true;
}

export interface RunCleanOptions {
  cwd: string;
  identifier: string | null;
  group?: string;
  cacheOnly?: boolean;
}

/**
 * Clear widget store / receipts / trace and/or prefetch cache.
 */
export function runClean({ cwd, identifier, group, cacheOnly = false }: RunCleanOptions): void {
  console.log("clean");
  let n = 0;

  if (identifier) validateSafeSegment(identifier, "identifier");
  if (group) validateSafeSegment(group, "group");

  if (cacheOnly) {
    if (rmDir(prefetchCacheDir(), "image-prefetch cache")) n++;
    console.log(n ? `OK — cleared cache` : `OK — nothing to clear`);
    return;
  }

  const trace = loadTraceFile(cwd, identifier);
  if (trace) {
    rmSync(trace.path, { force: true });
    console.log(`  ✓ trace: removed ${trace.path}`);
    n++;
  } else {
    console.log(`  – trace: not found`);
  }

  const home = process.env.HOME || "";
  const bases: string[] = [];
  // Explicit data-dir overrides used by the Rust store / CLI.
  for (const envKey of ["TAURI_WIDGETS_DATA", "WIDGET_DATA_DIR"] as const) {
    const override = process.env[envKey]?.trim();
    if (override) {
      bases.push(override.endsWith(".json") ? dirname(override) : override);
    }
  }
  if (identifier && home) {
    bases.push(join(home, "Library", "Application Support", identifier, "widgets"));
  }
  if (process.env.LOCALAPPDATA) {
    bases.push(join(process.env.LOCALAPPDATA, "tauri-plugin-widgets"));
  }
  if (identifier && process.env.LOCALAPPDATA) {
    bases.push(join(process.env.LOCALAPPDATA, identifier, "widgets"));
  }
  if (identifier && home) {
    bases.push(
      join(process.env.XDG_DATA_HOME || join(home, ".local", "share"), identifier, "widgets"),
    );
  }
  if (group && home && process.platform === "darwin") {
    const resolved = resolveMacGroupContainer(group);
    bases.push(resolved || join(home, "Library", "Group Containers", group));
  }

  const storeFiles = new Set([
    "widget_data.json",
    "widget_trace.json",
    "widget_receipt.json",
    "widget_receipts.json",
    "receipts.json",
    "render_receipt.json",
  ]);

  for (const base of bases) {
    if (!existsSync(base)) continue;
    try {
      for (const f of readdirSync(base)) {
        // Only known widget store / cache / config keys — never wipe unrelated App Group JSON.
        if (storeFiles.has(f) || /^config:/.test(f)) {
          const p = join(base, f);
          rmSync(p, { force: true, recursive: true });
          console.log(`  ✓ store: removed ${p}`);
          n++;
        }
      }
    } catch (e) {
      console.log(`  ⚠ ${base}: ${(e as Error).message}`);
    }
  }

  console.log(n ? `OK — ${n} path(s) cleared` : `OK — nothing to clear`);
}
