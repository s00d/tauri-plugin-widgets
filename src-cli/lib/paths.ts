import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

let cachedRoot: string | null = null;

/**
 * Locate the tauri-plugin-widgets-api package root by walking up from this
 * module's own location. Works both from `src-cli/lib/paths.ts` (ts-node/tsx)
 * and from the built `dist-cli/lib/paths.mjs` (two levels under the package root).
 */
export function pluginRoot(): string {
  if (cachedRoot) return cachedRoot;

  let dir = dirname(fileURLToPath(import.meta.url));
  for (let i = 0; i < 12; i++) {
    const pkgPath = join(dir, "package.json");
    if (existsSync(pkgPath)) {
      try {
        const pkg = JSON.parse(readFileSync(pkgPath, "utf-8")) as { name?: string };
        if (pkg?.name === "tauri-plugin-widgets-api") {
          cachedRoot = dir;
          return dir;
        }
      } catch {
        /* malformed package.json while walking up — keep walking */
      }
    }
    const parent = dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }

  throw new Error(
    `pluginRoot(): could not locate tauri-plugin-widgets-api package.json by walking up from ${fileURLToPath(import.meta.url)}`,
  );
}
