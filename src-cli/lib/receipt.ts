import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import type { Report } from "./report.js";

/** Expected receipt schema across Rust / Swift / Kotlin (keep in sync). */
const EXPECTED_RECEIPT_SCHEMA = 1;

interface SchemaCheck {
  label: string;
  path: string;
  re: RegExp;
}

/** Cross-check the render-receipt schema constant across Rust / Swift / Kotlin sources. */
export function checkReceiptSchemaSkew(pluginRoot: string, report: Report): void {
  const checks: SchemaCheck[] = [
    {
      label: "src/receipt.rs",
      path: join(pluginRoot, "src", "receipt.rs"),
      re: /fn default_schema\(\)\s*->\s*u32\s*\{\s*(\d+)/,
    },
    {
      label: "WidgetDataStore.swift",
      path: join(pluginRoot, "swift", "Sources", "TauriWidgets", "WidgetDataStore.swift"),
      re: /schema:\s*UInt32\s*=\s*(\d+)/,
    },
    {
      label: "TauriGlanceWidget.kt",
      path: join(pluginRoot, "android", "src", "main", "java", "TauriGlanceWidget.kt"),
      re: /\.put\("schema",\s*(\d+)\)/,
    },
  ];
  for (const c of checks) {
    if (!existsSync(c.path)) {
      report.warn(`${c.label}: file missing (skip schema check)`);
      continue;
    }
    const raw = readFileSync(c.path, "utf-8");
    const m = raw.match(c.re);
    if (!m) {
      report.warn(`${c.label}: could not parse schema default`);
      continue;
    }
    const n = Number(m[1]);
    if (n === EXPECTED_RECEIPT_SCHEMA) report.ok(`${c.label} schema=${n}`);
    else report.bad(`${c.label} schema=${n} (expected ${EXPECTED_RECEIPT_SCHEMA})`);
  }
}
