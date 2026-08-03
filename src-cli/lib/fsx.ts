import { existsSync, readdirSync } from "node:fs";
import { join } from "node:path";

export function findBuiltApps(cwd: string): string[] {
  const roots = [
    join(cwd, "src-tauri", "target", "release", "bundle", "macos"),
    join(cwd, "src-tauri", "target", "debug", "bundle", "macos"),
  ];
  const apps: string[] = [];
  for (const root of roots) {
    if (!existsSync(root)) continue;
    const walk = (dir: string, depth = 0) => {
      if (depth > 4) return;
      let entries: import("node:fs").Dirent[] = [];
      try {
        entries = readdirSync(dir, { withFileTypes: true });
      } catch {
        return;
      }
      for (const e of entries) {
        const p = join(dir, e.name);
        if (e.isDirectory() && e.name.endsWith(".app")) apps.push(p);
        else if (e.isDirectory()) walk(p, depth + 1);
      }
    };
    walk(root);
  }
  return apps;
}
