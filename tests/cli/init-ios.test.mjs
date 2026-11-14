// `tauri-widgets init-ios` on a fixture gen/apple/project.yml (what `tauri ios init` writes).
// xcodegen and plutil are replaced by stubs on PATH, so this runs on any Unix host.
//
// IOS_E2E=1 (macOS + xcodegen + Xcode): also generate the real Xcode project and build the
// WidgetExtension scheme for the iOS Simulator. Not run by default (minutes, needs Xcode).
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../..");
const CLI = join(ROOT, "dist-cli/cli.mjs");
const FIXTURE = join(ROOT, "tests/cli/fixtures/ios/project.yml");
const unix = process.platform !== "win32";

/** Temp Tauri project with the fixture spec; `stubs: false` keeps the real tools. */
function project({ conf = {}, stubs = true } = {}) {
  const dir = mkdtempSync(join(tmpdir(), "init-ios-"));
  const apple = join(dir, "src-tauri/gen/apple");
  mkdirSync(apple, { recursive: true });
  copyFileSync(FIXTURE, join(apple, "project.yml"));
  writeFileSync(
    join(dir, "src-tauri/tauri.conf.json"),
    JSON.stringify({ identifier: "com.example.demo", version: "1.2.3", ...conf }, null, 2),
  );
  const bin = join(dir, "bin");
  mkdirSync(bin);
  if (stubs) {
    // xcodegen: record the call. plutil: an app entitlements file that already has push.
    writeFileSync(join(bin, "xcodegen"), `#!/bin/sh\necho "$PWD $*" >> "${dir}/xcodegen.log"\n`);
    writeFileSync(join(bin, "plutil"), `#!/bin/sh\necho '{"aps-environment":"development"}'\n`);
    chmodSync(join(bin, "xcodegen"), 0o755);
    chmodSync(join(bin, "plutil"), 0o755);
  }
  const run = (...args) =>
    spawnSync(process.execPath, [CLI, "init-ios", ...args], {
      cwd: dir,
      encoding: "utf8",
      env: { ...process.env, PATH: `${bin}:${process.env.PATH}` },
    });
  const spec = () => readFileSync(join(apple, "project.yml"), "utf8");
  return { dir, apple, run, spec };
}

/** Lines of one top-level target block (`  name:` up to the next `  other:`). */
function targetBlock(spec, name) {
  const lines = spec.split("\n");
  const start = lines.indexOf(`  ${name}:`);
  assert.ok(start >= 0, `target ${name} missing`);
  const end = lines.findIndex((l, i) => i > start && /^ {2}[^\s#][^:]*:$/.test(l));
  return lines.slice(start, end < 0 ? undefined : end).join("\n");
}

describe("init-ios on a fixture project.yml", { skip: !unix && "needs sh stubs" }, () => {
  it("adds the extension target, package, embed, entitlements and live activities", () => {
    const p = project();
    const r = p.run();
    assert.equal(r.status, 0, r.stderr + r.stdout);
    const spec = p.spec();

    const ext = targetBlock(spec, "WidgetExtension");
    assert.match(ext, /type: app-extension/);
    assert.match(ext, /PRODUCT_BUNDLE_IDENTIFIER: com\.example\.demo\.WidgetExtension/);
    assert.match(ext, /DEVELOPMENT_TEAM: ABCDE12345/, "team copied from the app target");
    assert.match(ext, /TauriWidgetsAppGroup: group\.com\.example\.demo/);
    assert.match(ext, /- group\.com\.example\.demo/);
    assert.match(ext, /- path: \.\.\/\.\.\/ios-widget/);
    // Extension version must equal the app's (App Store check).
    assert.match(ext, /CFBundleShortVersionString: 0\.3\.0/);
    assert.match(ext, /CFBundleVersion: "0\.3\.0"/);
    assert.match(ext, /- package: TauriWidgets/);

    assert.match(spec, /^packages:\n {2}TauriWidgets:\n {4}path: \.\.\/\.\.\/\.\.\/node_modules\/tauri-plugin-widgets-api\/swift$/m);

    const app = targetBlock(spec, "demo_iOS");
    assert.match(app, / {4}dependencies:\n {6}- target: WidgetExtension\n {8}embed: true\n {6}- framework: libapp\.a/);
    assert.match(app, / {8}NSSupportsLiveActivities: true/);
    // Existing entitlement keys (from plutil) kept, the App Group added.
    const ents = app.match(/ {6}properties: (\{.*\})/);
    assert.ok(ents, "app entitlements properties");
    assert.deepEqual(JSON.parse(ents[1]), {
      "aps-environment": "development",
      "com.apple.security.application-groups": ["group.com.example.demo"],
    });
    // targetTemplates.app (also `type: application`) is untouched.
    assert.doesNotMatch(spec.slice(0, spec.indexOf("\ntargets:\n")), /WidgetExtension|NSSupportsLiveActivities/);

    assert.ok(existsSync(join(p.dir, "src-tauri/ios-widget/MyWidget.swift")));
    const swift = readFileSync(join(p.dir, "src-tauri/ios-widget/MyWidget.swift"), "utf8");
    assert.ok(!swift.includes("{{APP_GROUP}}"), "template placeholder replaced");
    const log = readFileSync(join(p.dir, "xcodegen.log"), "utf8");
    assert.match(log, /gen\/apple generate --spec project\.yml/);
  });

  it("is idempotent: a second run changes nothing", () => {
    const p = project();
    assert.equal(p.run().status, 0);
    const once = p.spec();
    const r = p.run();
    assert.equal(r.status, 0, r.stderr);
    assert.equal(p.spec(), once);
    assert.match(r.stdout, /already has the widget extension/);
    assert.equal(once.split("- target: WidgetExtension").length, 2);
  });

  it("keeps an edited MyWidget.swift unless --force", () => {
    const p = project();
    p.run();
    const file = join(p.dir, "src-tauri/ios-widget/MyWidget.swift");
    writeFileSync(file, "// mine\n");
    p.run();
    assert.equal(readFileSync(file, "utf8"), "// mine\n");
    p.run("--force");
    assert.notEqual(readFileSync(file, "utf8"), "// mine\n");
  });

  it("--app-group writes plugins.widgets.appGroup and uses it everywhere", () => {
    const p = project();
    assert.equal(p.run("--app-group", "group.custom.shared").status, 0);
    const conf = JSON.parse(readFileSync(join(p.dir, "src-tauri/tauri.conf.json"), "utf8"));
    assert.equal(conf.plugins.widgets.appGroup, "group.custom.shared");
    const spec = p.spec();
    assert.match(targetBlock(spec, "WidgetExtension"), /TauriWidgetsAppGroup: group\.custom\.shared/);
    assert.doesNotMatch(spec, /group\.com\.example\.demo/);
  });

  it("fails when the spec has no app target", () => {
    const p = project();
    writeFileSync(join(p.apple, "project.yml"), "name: x\ntargets:\n  lib:\n    type: framework\n");
    const r = p.run();
    assert.equal(r.status, 1);
    assert.match(r.stderr, /no app target/);
  });

  it("asks for `tauri ios init` when gen/apple is missing", () => {
    const p = project();
    rmSync(join(p.apple, "project.yml"));
    const r = p.run();
    assert.equal(r.status, 0);
    assert.match(r.stdout, /tauri ios init/);
  });
});

const e2e =
  process.env.IOS_E2E === "1" &&
  process.platform === "darwin" &&
  spawnSync("xcodegen", ["--version"]).status === 0;

describe("init-ios e2e: WidgetExtension builds for the iOS Simulator", { skip: !e2e && "set IOS_E2E=1 (macOS, xcodegen, Xcode)" }, () => {
  it("xcodebuild -scheme WidgetExtension", { timeout: 15 * 60_000 }, () => {
    const p = project({ stubs: false });
    // The package path init-ios writes: node_modules/tauri-plugin-widgets-api/swift.
    mkdirSync(join(p.dir, "node_modules"));
    symlinkSync(ROOT, join(p.dir, "node_modules/tauri-plugin-widgets-api"));
    // Source folders the app target lists (xcodegen refuses missing ones).
    for (const d of ["Sources", "demo_iOS", "assets"]) mkdirSync(join(p.apple, d), { recursive: true });
    mkdirSync(join(p.dir, "src-tauri/src")); // fileGroups: [../../src]
    // The scheme also builds the host app: drop its "Build Rust Code" phase (no Rust here)
    // plus storyboard / asset catalog / libapp.a, and give it a stub main.
    const shell = p.spec()
      .replace(/\n {4}preBuildScripts:\n[\s\S]*$/, "\n")
      .replace("      - path: LaunchScreen.storyboard\n", "")
      .replace("      - path: Assets.xcassets\n", "")
      .replace("      - framework: libapp.a\n        embed: false\n", "");
    writeFileSync(join(p.apple, "Sources/App.swift"), "@main struct App { static func main() {} }\n");
    writeFileSync(join(p.apple, "project.yml"), shell);
    const r = p.run();
    assert.equal(r.status, 0, r.stderr + r.stdout);
    const b = spawnSync(
      "xcodebuild",
      ["build", "-scheme", "WidgetExtension", "-destination", "generic/platform=iOS Simulator",
        "-derivedDataPath", join(p.dir, "DerivedData"), "CODE_SIGNING_ALLOWED=NO"],
      { cwd: p.apple, encoding: "utf8", maxBuffer: 256 * 1024 * 1024 },
    );
    const errors = b.stdout.split("\n").filter((l) => / error: /.test(l)).join("\n");
    assert.equal(b.status, 0, errors || b.stdout.slice(-4000) + b.stderr);
  });
});
