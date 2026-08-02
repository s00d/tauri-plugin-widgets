import { defineCommand } from "citty";
import { runGenerate } from "../utils/docs/generate.js";

export const docsGenerateCommand = defineCommand({
  meta: {
    name: "docs-generate",
    description: "Generate VitePress pages from schema, cases, goldens, permissions",
  },
  args: {
    check: {
      type: "boolean",
      default: false,
      description: "Fail when generated tracked markers would change",
    },
    "skip-audit": {
      type: "boolean",
      default: false,
      description: "Skip element/golden/schema-doc coverage asserts",
    },
  },
  async run({ args }) {
    const { report, auditErrors } = await runGenerate({
      check: args.check,
      audit: !args["skip-audit"],
    });

    if (args.check) {
      for (const f of report.stale) console.log(`  x ${f} is out of date`);
      console.log(
        report.stale.length === 0
          ? "All generated documentation is up to date."
          : "\nRun `pnpm docs:generate` and commit marker updates.",
      );
      if (report.stale.length > 0) process.exit(1);
    } else {
      console.log(`Wrote ${report.written.length} file(s).`);
    }

    if (auditErrors.length > 0) {
      console.error("\nDocs audit failed:");
      for (const e of auditErrors) console.error(`  - ${e}`);
      process.exit(1);
    }
  },
});
