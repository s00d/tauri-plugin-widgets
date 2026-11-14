#!/usr/bin/env node
// Lists Playwright tests that failed first and passed on retry (status "flaky") from the
// JSON reporter output. Writes to the GitHub step summary and warning annotations.
// Never fails the job: flaky tests are surfaced, the retry decides pass/fail.
import fs from "node:fs";

const file = process.argv[2] ?? "test-results/results.json";
if (!fs.existsSync(file)) {
  console.log(`no report at ${file}`);
  process.exit(0);
}
const report = JSON.parse(fs.readFileSync(file, "utf8"));
const flaky = [];
const walk = (suite, trail) => {
  const here = suite.title ? [...trail, suite.title] : trail;
  for (const spec of suite.specs ?? []) {
    for (const t of spec.tests ?? []) {
      if (t.status === "flaky") flaky.push({ title: [...here, spec.title].join(" › "), file: spec.file, line: spec.line });
    }
  }
  for (const s of suite.suites ?? []) walk(s, here);
};
for (const s of report.suites ?? []) walk(s, []);

const lines = flaky.length
  ? [`### Flaky Playwright tests (${flaky.length})`, "", ...flaky.map((f) => `- ${f.title} (${f.file}:${f.line})`)]
  : ["No flaky Playwright tests."];
console.log(lines.join("\n"));
for (const f of flaky) console.log(`::warning file=tests/desktop/${f.file},line=${f.line}::flaky (passed on retry): ${f.title}`);
if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, lines.join("\n") + "\n");
