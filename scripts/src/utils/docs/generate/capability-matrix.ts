// @ts-nocheck
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { DOCS, writeBlock } from "./shared.js";
export function genCapabilityMatrix(report, check) {
  const generated = join(DOCS, "guide", "_generated", "capability-matrix.md");
  // Prefer cargo-owned file if present; else keep marker empty-ish
  let body = "_Run `cargo test --lib capabilities::write_docs::capability_matrix_doc_matches` to refresh._";
  if (existsSync(generated)) {
    body = readFileSync(generated, "utf8")
      .replace(/^# .*\n+/, "")
      .replace(/^Table is authored[^\n]*\n+/, "")
      .replace(/^Generated from[^\n]*\n+/, "")
      .trim();
  }
  writeBlock(report, "docs/guide/tiers.md", "capability-matrix", body, check);
}
