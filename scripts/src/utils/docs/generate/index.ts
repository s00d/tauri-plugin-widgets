// @ts-nocheck
import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { auditCoverage, auditSchemaDocs } from "./audit.js";
import { genCapabilityMatrix } from "./capability-matrix.js";
import { genElements } from "./elements.js";
import { genPermissions } from "./permissions.js";
import { copySandbox, copySchemas, copyShots } from "./sandbox.js";
import { genShowcase } from "./showcase.js";
import { DOCS, PUBLIC, collectElementUsage } from "./shared.js";
export async function runGenerate({ check = false, audit = true } = {}) {
  /** @type {Report} */
  const report = { written: [], stale: [] };
  mkdirSync(PUBLIC, { recursive: true });
  mkdirSync(join(DOCS, "guide", "_generated"), { recursive: true });

  const usage = collectElementUsage();
  copyShots(report, check);
  copySandbox(report, check);
  genShowcase(report, check);
  const elements = genElements(report, check, usage);
  genPermissions(report, check);
  genCapabilityMatrix(report, check);
  copySchemas(report, check);

  let auditErrors = [];
  if (audit) {
    auditErrors = [
      ...auditCoverage(
        elements.map((e) => e.name),
        usage,
      ),
      ...auditSchemaDocs(elements),
    ];
  }

  return { report, auditErrors, elementNames: elements.map((e) => e.name), usage };
}
