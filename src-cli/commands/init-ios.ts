import { defineCommand } from "citty";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { pluginRoot } from "../lib/paths.js";
import { detectTauriIdentifier, effectiveAppGroup, ensurePluginsWidgets, readTauriConf } from "../lib/tauri-conf.js";
import { copyTemplate, widgetTemplateReplacements } from "../lib/templates.js";

export interface InitIosOptions {
  appGroup?: string;
  dir: string;
  force: boolean;
}

/** Marks the block init-ios owns in gen/apple/project.yml. */
const MARKER = "# tauri-plugin-widgets: widget extension";
const TARGET = "WidgetExtension";

/**
 * Add the WidgetKit extension to the XcodeGen spec Tauri generates (gen/apple/project.yml)
 * and regenerate the Xcode project: no manual "New Target", "Add Package" or entitlements
 * clicks. The widget Swift code lives in src-tauri/ios-widget, outside gen/.
 */
export function initIosAction(opts: InitIosOptions): void {
  const cwd = process.cwd();
  const conf = readTauriConf(cwd);
  if (!conf) {
    console.error("ERROR: tauri.conf.json not found. Run from your Tauri project root.");
    process.exit(1);
  }
  const identifier = detectTauriIdentifier(conf);
  if (!identifier) {
    console.error("ERROR: tauri.conf.json is missing 'identifier'.");
    process.exit(1);
  }
  const appGroup = opts.appGroup || effectiveAppGroup(conf)!;
  if (opts.appGroup && opts.appGroup !== `group.${identifier}`) {
    ensurePluginsWidgets(conf, { appGroup, explicitAppGroup: true });
    console.log(`  Wrote plugins.widgets.appGroup = ${appGroup}`);
  }

  // 1. Widget code — written once, never overwritten without --force.
  const widgetDir = resolve(cwd, opts.dir);
  const widgetFile = join(widgetDir, "MyWidget.swift");
  mkdirSync(widgetDir, { recursive: true });
  if (!existsSync(widgetFile) || opts.force) {
    copyTemplate(
      join(pluginRoot(), "templates", "ios-widget", "MyWidget.swift"),
      widgetFile,
      widgetTemplateReplacements(appGroup),
    );
    console.log(`  Wrote ${opts.dir}/MyWidget.swift`);
  } else {
    console.log(`  Kept ${opts.dir}/MyWidget.swift`);
  }

  // 2. Xcode project
  const appleDir = join(cwd, "src-tauri", "gen", "apple");
  const specPath = join(appleDir, "project.yml");
  if (!existsSync(specPath)) {
    console.log("\n  gen/apple/project.yml missing: run `pnpm tauri ios init`, then init-ios again.");
    return;
  }
  let spec = readFileSync(specPath, "utf-8");
  // Only under `targets:` (targetTemplates also has `type: application`).
  const targetsAt = () => spec.indexOf("\ntargets:\n");
  const appTarget = spec.slice(targetsAt()).match(/^ {2}([^\s:#]+):\n {4}type: application$/m)?.[1];
  if (!appTarget) {
    console.error(`ERROR: no app target found in ${specPath}`);
    process.exit(1);
  }

  // Main app entitlements. XcodeGen rewrites the .entitlements file from project.yml on
  // every generate, so the group must live in `properties` there; existing keys from the
  // file (push, associated domains, …) are carried over. YAML accepts the JSON flow form.
  const appBlock = (() => {
    const start = spec.indexOf(`\n  ${appTarget}:\n`, targetsAt()) + 1;
    const next = spec.slice(start + 1).search(/\n {2}[^\s#][^\n]*:\n/);
    return spec.slice(start, next < 0 ? undefined : start + 1 + next + 1);
  })();
  const entsBlock = appBlock.match(/^( {4}entitlements:\n {6}path: (.+)\n)((?: {6}properties:.*\n(?: {8,}.*\n)*)?)/m);
  if (entsBlock && !entsBlock[3]) {
    const file = join(appleDir, entsBlock[2].trim());
    let props: Record<string, unknown> = { "com.apple.security.application-groups": [appGroup] };
    try {
      props = JSON.parse(execFileSync("plutil", ["-convert", "json", "-o", "-", file], { encoding: "utf-8" }));
    } catch {
      // no file / not a plist: the group alone
    }
    const key = "com.apple.security.application-groups";
    const groups = Array.isArray(props[key]) ? (props[key] as string[]) : [];
    if (!groups.includes(appGroup)) props[key] = [...groups, appGroup];
    spec = spec.replace(appBlock, appBlock.replace(entsBlock[0], `${entsBlock[1]}      properties: ${JSON.stringify(props)}\n`));
    writeFileSync(specPath, spec, "utf-8");
    console.log(`  App entitlements: ${appGroup} (in project.yml)`);
  } else if (entsBlock) {
    console.log("  App entitlements already have properties in project.yml — add the App Group there if missing");
  }

  // Live activities (`startActivity`): ActivityKit needs the flag in the APP's Info.plist.
  {
    // Re-read the block: the entitlements step above may have rewritten it.
    const start = spec.indexOf(`\n  ${appTarget}:\n`, targetsAt()) + 1;
    const next = spec.slice(start + 1).search(/\n {2}[^\s#][^\n]*:\n/);
    const block = spec.slice(start, next < 0 ? undefined : start + 1 + next + 1);
    const info = block.match(/^ {4}info:\n {6}path: .+\n {6}properties:\n/m);
    // Frequent updates: more than the default hourly budget of push / local updates.
    const missing = ["NSSupportsLiveActivities", "NSSupportsLiveActivitiesFrequentUpdates"].filter(
      (k) => !new RegExp(`^ {8}${k}:`, "m").test(block),
    );
    if (missing.length === 0) {
      // already there
    } else if (info && start > 0) {
      const patched = block.replace(info[0], `${info[0]}${missing.map((k) => `        ${k}: true\n`).join("")}`);
      spec = spec.replace(block, patched);
      writeFileSync(specPath, spec, "utf-8");
      console.log(`  App Info.plist: ${missing.join(", ")} (live activities)`);
    } else {
      console.log("  Add NSSupportsLiveActivities = YES (and NSSupportsLiveActivitiesFrequentUpdates) to the app's Info.plist to use startActivity()");
    }
  }

  // `tauri ios init` writes iOS 14.0; current Xcode (SDK minimum 15.0) refuses to build it.
  spec = spec.replace(/^(\s+deploymentTarget:\n\s+iOS: )["']?(\d+(?:\.\d+)?)["']?$/m, (line, head, v) => {
    if (Number.parseFloat(v) >= 15) return line;
    console.log(`  project deployment target iOS ${v} → 15.0 (Xcode minimum)`);
    return `${head}15.0`;
  });

  if (spec.includes(MARKER)) {
    console.log("  project.yml already has the widget extension");
  } else {
    const team =
      spec.match(/DEVELOPMENT_TEAM: (\S+)/)?.[1] ||
      String((conf.data.bundle as { iOS?: { developmentTeam?: string } } | undefined)?.iOS?.developmentTeam || "");
    const swiftPkg = relative(appleDir, join(cwd, "node_modules", "tauri-plugin-widgets-api", "swift"));
    // App Store rejects an extension whose version differs from its app: copy the app's.
    const appVersion = (key: string) => appBlock.match(new RegExp(`^ {8}${key}: (.+)$`, "m"))?.[1]?.trim();
    const shortVersion = appVersion("CFBundleShortVersionString") ?? JSON.stringify(String(conf.data.version ?? "1.0.0"));
    const bundleVersion = appVersion("CFBundleVersion") ?? shortVersion;
    const sources = relative(appleDir, widgetDir);

    const packages = `${MARKER} (package)\npackages:\n  TauriWidgets:\n    path: ${swiftPkg}\n`;
    spec = /^packages:/m.test(spec)
      ? spec.replace(/^packages:\n/m, `packages:\n  TauriWidgets:\n    path: ${swiftPkg}\n`)
      : spec.replace(/^targets:\n/m, `${packages}targets:\n`);

    const target = [
      `  ${MARKER}`,
      `  ${TARGET}:`,
      "    type: app-extension",
      "    platform: iOS",
      "    deploymentTarget: \"17.0\"",
      "    sources:",
      `      - path: ${sources}`,
      "    info:",
      `      path: ${TARGET}/Info.plist`,
      "      properties:",
      "        CFBundleDisplayName: Widgets",
      `        CFBundleShortVersionString: ${shortVersion}`,
      `        CFBundleVersion: ${bundleVersion}`,
      // Read by TauriInstanceQuery before any provider ran (Edit Widget choices).
      `        TauriWidgetsAppGroup: ${appGroup}`,
      "        NSExtension:",
      "          NSExtensionPointIdentifier: com.apple.widgetkit-extension",
      "    entitlements:",
      `      path: ${TARGET}/${TARGET}.entitlements`,
      "      properties:",
      "        com.apple.security.application-groups:",
      `          - ${appGroup}`,
      "    settings:",
      "      base:",
      `        PRODUCT_BUNDLE_IDENTIFIER: ${identifier}.${TARGET}`,
      "        CODE_SIGN_STYLE: Automatic",
      ...(team ? [`        DEVELOPMENT_TEAM: ${team}`] : []),
      "        SWIFT_VERSION: \"5.9\"",
      "        TARGETED_DEVICE_FAMILY: \"1,2\"",
      "        SKIP_INSTALL: YES",
      "    dependencies:",
      "      - package: TauriWidgets",
      "",
    ].join("\n");
    spec = spec.replace(/^targets:\n/m, `targets:\n${target}`);

    // Embed the extension in the app: first `dependencies:` list of the app target.
    const appStart = spec.indexOf(`\n  ${appTarget}:\n`, targetsAt());
    const depIdx = spec.indexOf("\n    dependencies:\n", appStart);
    if (appStart < 0 || depIdx < 0) {
      console.error(`ERROR: cannot find dependencies of ${appTarget} in project.yml`);
      process.exit(1);
    }
    const at = depIdx + "\n    dependencies:\n".length;
    spec = `${spec.slice(0, at)}      - target: ${TARGET}\n        embed: true\n${spec.slice(at)}`;

    writeFileSync(specPath, spec, "utf-8");
    console.log(`  Added ${TARGET} target + TauriWidgets package to project.yml`);
  }

  try {
    execFileSync("xcodegen", ["generate", "--spec", "project.yml"], { cwd: appleDir, stdio: "inherit" });
  } catch {
    console.error("\nERROR: xcodegen failed or is not installed (brew install xcodegen).");
    process.exit(1);
  }

  const handMade = readdirSync(appleDir).filter((n) => n !== TARGET && /Widget/i.test(n) && !n.includes("."));
  if (handMade.length) {
    console.log(`\n  Note: ${handMade.join(", ")} looks like a widget target made in Xcode earlier.`);
    console.log("  It is no longer in the project; move any custom code into", opts.dir, "and delete it.");
  }

  console.log(`
Done. Widget code: ${opts.dir}/MyWidget.swift (widgetId "default", App Group ${appGroup})

  Simulator: pnpm tauri ios dev
  Device:    needs a paid Apple Developer account (App Groups). Register the group once:
             https://developer.apple.com/account/resources/identifiers/list
             (Automatic signing in Xcode usually does it for you on the first device build.)
`);
}

export const initIos = defineCommand({
  meta: {
    name: "init-ios",
    description: "Add the WidgetKit extension target to src-tauri/gen/apple (XcodeGen)",
  },
  args: {
    "app-group": {
      type: "string",
      description: "App Group id. Default: plugins.widgets.appGroup, else group.<identifier>.",
    },
    dir: {
      type: "string",
      description: "Widget Swift sources (default: src-tauri/ios-widget)",
      default: "src-tauri/ios-widget",
    },
    force: {
      type: "boolean",
      description: "Overwrite MyWidget.swift with the template",
      default: false,
    },
  },
  run({ args }) {
    initIosAction({
      appGroup: args["app-group"],
      dir: args.dir,
      force: args.force,
    });
  },
});
