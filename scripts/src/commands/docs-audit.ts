import { defineCommand } from "citty";
import {
  auditCoverage,
  auditSchemaDocs,
  collectElementUsage,
  extractElementsFromSchema,
  loadSchema,
} from "../utils/docs/generate.js";

type SchemaElement = {
  name: string;
  description: string;
  rootDesc?: string;
  properties: Record<string, { description?: string }>;
};

export const docsAuditCommand = defineCommand({
  meta: {
    name: "docs-audit",
    description: "Assert schema element coverage, goldens, and property docs",
  },
  async run() {
    const usage = collectElementUsage();
    const elements = extractElementsFromSchema(loadSchema()) as SchemaElement[];
    const names = elements.map((e) => e.name);
    const errors = [...auditCoverage(names, usage), ...auditSchemaDocs(elements)];
    if (errors.length === 0) {
      console.log(`OK — ${names.length} elements, coverage + schema docs complete.`);
      return;
    }
    console.error("Docs audit failed:");
    for (const e of errors) console.error(`  - ${e}`);
    process.exit(1);
  },
});
