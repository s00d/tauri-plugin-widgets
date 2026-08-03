// @ts-nocheck
import { existsSync } from "node:fs";
import { join } from "node:path";
import {
  ROOT,
  caseRequiredPlatforms,
  listCases,
} from "./shared.js";
export function auditSchemaDocs(elements) {
  const errors = [];
  const enumBlurb = elements[0]?.rootDesc || "A UI element that can be a layout container or a leaf widget.";
  for (const el of elements) {
    if (!el.description || !el.description.trim()) {
      errors.push(`element \`${el.name}\` missing variant description in schema`);
    } else if (el.description.trim() === enumBlurb.trim()) {
      errors.push(`element \`${el.name}\` still uses enum-level description`);
    }
    for (const [name, prop] of Object.entries(el.properties)) {
      if (name === "type") continue;
      if (!prop.description || !String(prop.description).trim()) {
        errors.push(`\`${el.name}.${name}\` missing description`);
      }
    }
  }
  return errors;
}

export function auditCoverage(elementNames, usage) {
  const errors = [];
  for (const name of elementNames) {
    if ((usage.get(name) ?? []).length === 0) {
      errors.push(`element \`${name}\` has no fixture coverage`);
    }
  }
  for (const name of listCases()) {
    for (const platform of caseRequiredPlatforms(name)) {
      const png = join(ROOT, "tests/golden", platform, `${name}.png`);
      if (!existsSync(png)) {
        errors.push(`case \`${name}\` missing golden for ${platform}`);
      }
    }
  }
  return errors;
}
