import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync, mkdtempSync, writeFileSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { validateWidgetConfig, loadCapabilities, validatePluginWidgets } from "../../dist-cli/lib/validate.mjs";
import { formatTraceEvent } from "../../dist-cli/lib/trace.mjs";
import { prefetchCacheDir } from "../../dist-cli/lib/clean.mjs";
import { startPreviewServer, loadConfig } from "../../dist-cli/lib/preview.mjs";
import { pluginRoot } from "../../dist-cli/lib/paths.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../..");
const CLI = join(ROOT, "dist-cli/cli.mjs");

function cli(...args) {
  return spawnSync(process.execPath, [CLI, ...args], {
    encoding: "utf8",
    cwd: ROOT,
  });
}

describe("consumer CLI surface", () => {
  it("lists consumer commands", () => {
    const r = cli("--help");
    assert.equal(r.status, 0);
    for (const cmd of ["init", "preview", "validate", "signing", "doctor", "trace", "clean"]) {
      assert.match(r.stdout, new RegExp(cmd));
    }
  });

  it("signing --help mentions apply and dev-cert", () => {
    const r = cli("signing", "--help");
    assert.equal(r.status, 0);
    assert.match(r.stdout, /apply/);
    assert.match(r.stdout, /dev-cert/);
  });

  it("pluginRoot resolves templates from dist-cli", () => {
    assert.ok(existsSync(join(pluginRoot(), "templates/macos-widget/project.yml")));
  });
});

describe("validate", () => {
  it("loads capabilities.json", () => {
    const { data } = loadCapabilities();
    assert.equal(data.version, 1);
    assert.ok(data.elements.vstack);
    assert.ok(data.elements.timer);
  });

  it("flags degraded canvas on android", () => {
    const { data } = loadCapabilities();
    const findings = validateWidgetConfig(
      {
        small: { type: "canvas", draws: [] },
      },
      ["android"],
      data,
    );
    assert.ok(findings.some((f) => f.type === "canvas" && f.support === "degraded"));
  });

  it("cli validate exits 0 on starter widget", () => {
    const r = cli("validate", join(ROOT, "templates/starter/widget.json"), "--platforms", "desktop,ios");
    assert.equal(r.status, 0, r.stdout + r.stderr);
    assert.match(r.stdout, /OK|✓/);
  });

  it("validatePluginWidgets requires appGroup", () => {
    const bad = validatePluginWidgets({ transport: "appGroup" });
    assert.ok(bad.some((f) => f.level === "error"));
    const good = validatePluginWidgets({ transport: "widgetContainer", appGroup: "group.com.x" });
    assert.equal(good.filter((f) => f.level === "error").length, 0);
  });
});

describe("preview", () => {
  it("serves config JSON", async () => {
    const dir = mkdtempSync(join(tmpdir(), "tpw-prev-"));
    const cfg = join(dir, "w.json");
    writeFileSync(
      cfg,
      JSON.stringify({
        $schema: "https://example.com/schema.json",
        small: { type: "text", content: "hi", color: "#fff" },
      }),
    );
    const stripped = loadConfig(cfg);
    assert.equal(stripped.$schema, undefined);
    assert.equal(stripped.small.type, "text");

    const srv = await startPreviewServer({
      configPath: cfg,
      size: "small",
      port: 41789,
      watchFile: false,
    });
    try {
      const res = await fetch("http://127.0.0.1:41789/api/config");
      assert.equal(res.status, 200);
      const j = await res.json();
      assert.equal(j.small.content, "hi");
      const html = await fetch(srv.url);
      assert.equal(html.status, 200);
      const body = await html.text();
      assert.match(body, /__TAURI_INTERNALS__/);
    } finally {
      srv.close();
    }
  });
});

describe("trace format", () => {
  it("formats reload throttle", () => {
    const line = formatTraceEvent({
      ts: Date.parse("2026-08-03T12:05:31Z"),
      kind: "reload",
      reason: { outcome: "throttled", remainingSecs: 840 },
    });
    assert.match(line, /throttled/);
    assert.match(line, /840/);
  });
});

describe("clean", () => {
  it("exposes prefetch cache path", () => {
    assert.match(prefetchCacheDir(), /tauri-plugin-widgets/);
    assert.match(prefetchCacheDir(), /image-prefetch/);
  });
});

describe("schemas published", () => {
  it("has plugin-config and capabilities", () => {
    assert.ok(existsSync(join(ROOT, "schemas/plugin-config.v1.json")));
    assert.ok(existsSync(join(ROOT, "schemas/capabilities.json")));
    const caps = JSON.parse(readFileSync(join(ROOT, "schemas/capabilities.json"), "utf8"));
    assert.equal(caps.version, 1);
  });
});
