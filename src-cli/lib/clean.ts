import { existsSync, readdirSync, rmSync } from "node:fs";
import { join } from "node:path";
import { loadTraceFile } from "./trace.js";

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
  if (identifier && home) {
    bases.push(join(home, "Library", "Application Support", identifier, "widgets"));
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
    bases.push(join(home, "Library", "Group Containers", group));
  }

  for (const base of bases) {
    if (!existsSync(base)) continue;
    const files = ["widget_data.json", "widget_trace.json", "receipts.json", "render_receipt.json"];
    try {
      for (const f of readdirSync(base)) {
        if (files.includes(f) || /^config:/.test(f) || f.endsWith(".json")) {
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
