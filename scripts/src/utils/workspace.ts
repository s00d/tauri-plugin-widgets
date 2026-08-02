import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** Repository root (parent of `scripts/`). */
export function repoRoot(): string {
  const here = dirname(fileURLToPath(import.meta.url));
  return resolve(here, "../../..");
}
