import { defineCommand } from "citty";
import { auditSchemaDocs, extractElementsFromSchema, loadSchema } from "../utils/docs/generate.js";

type SchemaElement = {
  name: string;
  description: string;
  rootDesc?: string;
  properties: Record<string, { description?: string }>;
};

export const docsAuditCommand = defineCommand({
  meta: {
    name: "docs-audit",
    description: "Assert every schema element and property has a description",
  },
  async run() {
    const elements = extractElementsFromSchema(loadSchema()) as SchemaElement[];
    const names = elements.map((e) => e.name);
    const errors = auditSchemaDocs(elements);
    if (errors.length === 0) {
      console.log(`OK — ${names.length} elements, schema docs complete.`);
      return;
    }
    console.error("Docs audit failed:");
    for (const e of errors) console.error(`  - ${e}`);
    process.exit(1);
  },
});
