import { spawnSync } from "node:child_process";
import { join } from "node:path";
import { repoRoot } from "../utils/workspace.js";

/**
 * Run a repo-root bash script (usually under `scripts/sh/` or `scripts/win/`)
 * with inherited stdio. Extra args after the script name are forwarded.
 */
export function runToolScript(scriptRel: string, args: string[] = []): void {
  const root = repoRoot();
  const script = join(root, scriptRel);
  const result = spawnSync("bash", [script, ...args], {
    cwd: root,
    stdio: "inherit",
    env: process.env,
  });
  if (result.error) throw result.error;
  const code = result.status ?? 1;
  if (code !== 0) process.exit(code);
}
