import { defineCommand } from "citty";
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

function pluginRoot(): string {
  // dist-cli/commands/tools.mjs → repo root is ../..
  const here = dirname(fileURLToPath(import.meta.url));
  const candidates = [
    join(here, "../.."),
    join(here, "../../.."),
  ];
  for (const c of candidates) {
    if (existsSync(join(c, "scripts", "package.json"))) return c;
  }
  return join(here, "../..");
}

/**
 * Proxy to maintainer tooling (`pnpm -C scripts cli …`).
 * Keeps one consumer entrypoint (`tauri-widgets`) while stands stay in scripts/.
 */
export const toolsCmd = defineCommand({
  meta: {
    name: "tools",
    description:
      "Maintainer stands (docs-generate, ios-up, android-up, triage, …). Forwards to `pnpm -C scripts cli`.",
  },
  args: {
    // citty passes leftover argv via `--` style; we use raw process.argv
  },
  async run() {
    const root = pluginRoot();
    // Everything after `tools` (or `tools --`)
    const argv = process.argv;
    const idx = argv.findIndex((a) => a === "tools");
    const forward = idx >= 0 ? argv.slice(idx + 1).filter((a) => a !== "--") : [];
    if (forward.length === 0) {
      console.log(`Usage: tauri-widgets tools <command> [args…]

Examples:
  tauri-widgets tools docs-generate --check
  tauri-widgets tools triage
  tauri-widgets tools ios-up

Equivalent to: pnpm -C scripts cli <command> [args…]
Repo: ${root}`);
      return;
    }
    const result = spawnSync(
      "pnpm",
      ["-C", join(root, "scripts"), "cli", ...forward],
      { stdio: "inherit", shell: process.platform === "win32" },
    );
    process.exit(result.status ?? 1);
  },
});
