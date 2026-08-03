// @ts-nocheck
import { join } from "node:path";
import {
  PUBLIC,
  listCases,
  loadCase,
  loadFixture,
  writeBlock,
  writeOrCheck,
} from "./shared.js";
/** Group case names by preset base (strip .small/.medium/.large). */
export function groupCasesByPreset() {
  /** @type {Map<string, { name: string, case: any }[]>} */
  const groups = new Map();
  for (const name of listCases()) {
    const c = loadCase(name);
    const base = name.replace(/\.(small|medium|large)$/, "") || name;
    const list = groups.get(base) ?? [];
    list.push({ name, case: c });
    groups.set(base, list);
  }
  for (const list of groups.values()) {
    list.sort((a, b) => {
      const order = { small: 0, medium: 1, large: 2 };
      return (order[a.case.size] ?? 9) - (order[b.case.size] ?? 9);
    });
  }
  return groups;
}

export function genShowcase(report, check) {
  const groups = groupCasesByPreset();
  const parts = [];

  for (const [base, entries] of [...groups.entries()].sort((a, b) => a[0].localeCompare(b[0]))) {
    const sizes = entries.map((e) => e.case.size).join(", ");
    parts.push(`## ${base}\n`);
    parts.push(`Sizes: ${sizes}\n`);

    // Size tabs via ShotGrid for each case — show all size variants' desktop shot + platform tabs on primary
    for (const { name, case: c } of entries) {
      parts.push(`### ${c.size}\n`);
      parts.push(`<ShotGrid case="${name}" />\n`);
    }

    const primary = entries[0];
    let fixtureJson = "";
    try {
      fixtureJson = JSON.stringify(loadFixture(primary.case.fixture), null, 2);
    } catch (e) {
      fixtureJson = `/* ${e} */`;
    }

    writeOrCheck(
      report,
      join(PUBLIC, "gallery-data", `${primary.name}.json`),
      fixtureJson.endsWith("\n") ? fixtureJson : fixtureJson + "\n",
      check,
    );

    const b64 = Buffer.from(fixtureJson, "utf8").toString("base64");
    parts.push(`<details>\n<summary>Config JSON</summary>\n\n\`\`\`json\n${fixtureJson}\n\`\`\`\n\n</details>\n`);
    parts.push(
      `<Playground case="${primary.name}" size="${primary.case.size || "small"}" config-b64="${b64}" />\n`,
    );
  }

  writeBlock(report, "docs/showcase.md", "showcase", parts.join("\n"), check);
}
