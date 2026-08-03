import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, writeFileSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { runDoctor } from "../../dist-cli/commands/doctor.mjs";
import { pluginRoot } from "../../dist-cli/lib/paths.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../..");
const EXAMPLE = join(ROOT, "examples/tauri-plugin-widgets-example");

describe("runDoctor", () => {
  it("flags missing tauri.conf without printing", async () => {
    const dir = mkdtempSync(join(tmpdir(), "tpw-doctor-"));
    const r = await runDoctor(dir);
    assert.ok(r.failures >= 1);
    assert.ok(
      r.items.some((i) => i.level === "bad" && /tauri\.conf\.json/.test(i.msg)),
      JSON.stringify(r.items.filter((i) => i.level === "bad"), null, 2),
    );
  });

  it("flags empty appGroup when conf exists", async () => {
    const dir = mkdtempSync(join(tmpdir(), "tpw-doctor-ag-"));
    const st = join(dir, "src-tauri");
    mkdirSync(st, { recursive: true });
    writeFileSync(
      join(st, "tauri.conf.json"),
      JSON.stringify({
        identifier: "com.test.empty-ag",
        plugins: { widgets: { transport: "auto", appGroup: "" } },
      }),
    );
    const r = await runDoctor(dir);
    assert.ok(
      r.items.some((i) => i.level === "bad" && /appGroup is empty/.test(i.msg)),
      JSON.stringify(r.items.filter((i) => i.section === "transport"), null, 2),
    );
  });

  it("returns structured items for the example app", async () => {
    assert.ok(existsSync(EXAMPLE));
    const r = await runDoctor(EXAMPLE);
    assert.ok(r.items.length > 0);
    assert.ok(r.items.some((i) => i.level === "ok"));
    // Must not throw / print — failures is a number we can assert on
    assert.equal(typeof r.failures, "number");
    assert.equal(typeof r.print, "function");
  });

  it("pluginRoot from doctor module tree finds package templates", () => {
    assert.ok(existsSync(join(pluginRoot(), "templates/macos-widget/project.yml")));
  });
});
