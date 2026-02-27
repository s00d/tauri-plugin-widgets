#!/usr/bin/env node

import { defineCommand, runMain } from "citty";
import { readFileSync, writeFileSync, mkdirSync, copyFileSync, chmodSync, existsSync, readdirSync } from "node:fs";
import { resolve, dirname, join, basename } from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PLUGIN_ROOT = resolve(__dirname, "..");

function findTauriConf(cwd) {
  const candidates = [
    join(cwd, "src-tauri", "tauri.conf.json"),
    join(cwd, "tauri.conf.json"),
  ];
  for (const c of candidates) {
    if (existsSync(c)) return c;
  }
  return null;
}

function readTauriConf(cwd) {
  const confPath = findTauriConf(cwd);
  if (!confPath) return null;
  try {
    return { path: confPath, data: JSON.parse(readFileSync(confPath, "utf-8")) };
  } catch {
    return null;
  }
}

function detectTauriIdentifier(conf) {
  const id = String(conf?.data?.identifier || "").trim();
  return id || null;
}

function replaceAll(str, search, replacement) {
  return str.split(search).join(replacement);
}

function copyTemplate(src, dest, replacements) {
  let content = readFileSync(src, "utf-8");
  for (const [search, replacement] of Object.entries(replacements)) {
    content = replaceAll(content, search, replacement);
  }
  writeFileSync(dest, content, "utf-8");
}

function renderIosWidgetSwift(templatePath, destPath, appGroup) {
  let content = readFileSync(templatePath, "utf-8");
  content = replaceAll(content, "group.com.example.myapp", appGroup);

  // Xcode-generated widget targets usually include `<Name>Bundle.swift` with @main.
  // In that case this file should declare `struct <Name>: Widget` (without @main)
  // so Bundle can reference it.
  const widgetName = basename(destPath, ".swift");
  const bundlePath = join(dirname(destPath), `${widgetName}Bundle.swift`);
  if (existsSync(bundlePath)) {
    content = content.replace(/@main\s*\n\s*struct\s+MyWidget\s*:\s*Widget/, `struct ${widgetName}: Widget`);
  }

  return content;
}

function findIosWidgetEntryFiles(cwd) {
  const appleDir = resolve(cwd, "src-tauri", "gen", "apple");
  if (!existsSync(appleDir)) return [];

  const entries = readdirSync(appleDir, { withFileTypes: true });
  const result = [];

  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    const name = entry.name;

    if (
      name.startsWith(".") ||
      name.endsWith(".xcodeproj") ||
      name === "Externals" ||
      name === "Sources" ||
      name === "assets" ||
      name === "Assets.xcassets"
    ) {
      continue;
    }

    const dirPath = join(appleDir, name);
    const files = readdirSync(dirPath, { withFileTypes: true })
      .filter((f) => f.isFile() && f.name.endsWith(".swift"))
      .map((f) => join(dirPath, f.name));

    const candidates = files.filter((filePath) => {
      const fileName = filePath.split("/").pop() || "";
      if (fileName.endsWith("Bundle.swift")) return false;
      const content = readFileSync(filePath, "utf-8");
      return content.includes("import WidgetKit") && content.includes("WidgetConfiguration");
    });

    if (!candidates.length) continue;

    const preferred = candidates.find((p) => p.endsWith(`/${name}.swift`));
    result.push(preferred || candidates[0]);
  }

  return Array.from(new Set(result));
}

function findIosWidgetTargetNames(cwd) {
  const appleDir = resolve(cwd, "src-tauri", "gen", "apple");
  if (!existsSync(appleDir)) return [];

  const entries = readdirSync(appleDir, { withFileTypes: true });
  const targets = new Set();

  for (const entry of entries) {
    if (!entry.isDirectory() || !entry.name.endsWith(".xcodeproj")) continue;
    const pbxprojPath = join(appleDir, entry.name, "project.pbxproj");
    if (!existsSync(pbxprojPath)) continue;

    const pbx = readFileSync(pbxprojPath, "utf-8");
    const sectionStart = pbx.indexOf("/* Begin PBXNativeTarget section */");
    const sectionEnd = pbx.indexOf("/* End PBXNativeTarget section */");
    if (sectionStart === -1 || sectionEnd === -1 || sectionEnd <= sectionStart) continue;

    const section = pbx.slice(sectionStart, sectionEnd);
    const blockRe = /\bisa = PBXNativeTarget;[\s\S]*?};/g;
    let match;
    while ((match = blockRe.exec(section)) !== null) {
      const block = String(match[0] || "");
      if (!block.includes('productType = "com.apple.product-type.app-extension";')) continue;

      const nameMatch = block.match(/\bname = ([^;]+);/);
      const rawName = String(nameMatch?.[1] || "").trim();
      if (!rawName) continue;
      const cleanName = rawName.replace(/^"|"$/g, "");
      if (cleanName) targets.add(cleanName);
    }
  }

  return Array.from(targets);
}

// ─── init-macos ─────────────────────────────────────────────────────────────

const initMacos = defineCommand({
  meta: {
    name: "init-macos",
    description: "Initialize macOS widget extension in src-tauri/macos-widget/",
  },
  args: {
    "bundle-id": {
      type: "string",
      description: "Widget bundle identifier (e.g. com.example.myapp.widgetkit). Auto-detected from tauri.conf.json if omitted.",
    },
    "app-group": {
      type: "string",
      description: "App Group identifier (e.g. group.com.example.myapp). Auto-detected from tauri.conf.json if omitted.",
    },
    dir: {
      type: "string",
      description: "Target directory (default: src-tauri/macos-widget)",
      default: "src-tauri/macos-widget",
    },
    force: {
      type: "boolean",
      description: "Overwrite existing files",
      default: false,
    },
  },
  run({ args }) {
    const cwd = process.cwd();
    const conf = readTauriConf(cwd);

    let bundleId = args["bundle-id"];
    let appGroup = args["app-group"];

    if (!bundleId || !appGroup) {
      if (!conf) {
        console.error("ERROR: Could not find tauri.conf.json. Run from your Tauri project root, or pass --bundle-id and --app-group explicitly.");
        process.exit(1);
      }
      const identifier = detectTauriIdentifier(conf);
      if (!identifier) {
        console.error("ERROR: tauri.conf.json is missing 'identifier'. Set it, or pass --bundle-id and --app-group explicitly.");
        process.exit(1);
      }
      if (!bundleId) {
        bundleId = identifier + ".widgetkit";
        console.log(`  Auto-detected bundle-id: ${bundleId}`);
      }
      if (!appGroup) {
        appGroup = "group." + identifier;
        console.log(`  Auto-detected app-group: ${appGroup}`);
      }
    }

    const widgetDir = resolve(cwd, args.dir);
    const templateDir = join(PLUGIN_ROOT, "templates", "macos-widget");
    const swiftPackagePath = resolve(PLUGIN_ROOT, "swift");

    if (existsSync(widgetDir) && !args.force) {
      console.error(`ERROR: ${args.dir} already exists. Use --force to overwrite.`);
      process.exit(1);
    }

    console.log(`\nCreating macOS widget extension in ${args.dir}...\n`);

    mkdirSync(join(widgetDir, "Sources"), { recursive: true });

    copyTemplate(
      join(templateDir, "MyWidget.swift"),
      join(widgetDir, "Sources", "MyWidget.swift"),
      { "group.com.example.myapp": appGroup },
    );
    console.log("  Created Sources/MyWidget.swift");

    copyTemplate(
      join(templateDir, "Entitlements.plist"),
      join(widgetDir, "TauriWidgetExtension.entitlements"),
      { "group.com.example.myapp": appGroup },
    );
    console.log("  Created TauriWidgetExtension.entitlements");

    copyTemplate(
      join(templateDir, "App.entitlements"),
      join(widgetDir, "App.entitlements"),
      { "group.com.example.myapp": appGroup },
    );
    console.log("  Created App.entitlements");

    copyTemplate(
      join(templateDir, "project.yml"),
      join(widgetDir, "project.yml"),
      {
        "TAURI_WIDGETS_SWIFT_PATH": swiftPackagePath,
        "WIDGET_BUNDLE_ID": bundleId,
      },
    );
    console.log("  Created project.yml");

    copyFileSync(
      join(templateDir, "Info.plist"),
      join(widgetDir, "Info.plist"),
    );
    console.log("  Created Info.plist");

    copyFileSync(
      join(templateDir, "build-widget.sh"),
      join(widgetDir, "build-widget.sh"),
    );
    chmodSync(join(widgetDir, "build-widget.sh"), 0o755);
    console.log("  Created build-widget.sh");

    copyFileSync(
      join(templateDir, "embed-widget.sh"),
      join(widgetDir, "embed-widget.sh"),
    );
    chmodSync(join(widgetDir, "embed-widget.sh"), 0o755);
    console.log("  Created embed-widget.sh");

    writeFileSync(join(widgetDir, ".gitignore"), "build/\n*.xcodeproj\nxcuserdata/\n");
    console.log("  Created .gitignore");

    // Auto-patch tauri.conf.json
    if (conf) {
      let modified = false;
      const data = conf.data;

      if (!data.build) data.build = {};
      if (!data.build.beforeBundleCommand) {
        data.build.beforeBundleCommand = `./${args.dir}/build-widget.sh || true`;
        modified = true;
      }

      if (!data.bundle) data.bundle = {};
      if (data.bundle.targets !== "app" && !(Array.isArray(data.bundle.targets) && data.bundle.targets.includes("app") && data.bundle.targets.length === 1)) {
        data.bundle.targets = ["app"];
        modified = true;
        console.log(`  Set bundle.targets to ["app"] (DMG is rebuilt by embed-widget.sh)`);
      }

      if (modified) {
        writeFileSync(conf.path, JSON.stringify(data, null, 2) + "\n", "utf-8");
        console.log(`\n  Updated ${conf.path.replace(cwd + "/", "")}`);
      }
    }

    console.log(`
Done! Next steps:

  1. Install xcodegen if not already:
     brew install xcodegen

  2. Build the app (this also builds the .appex via beforeBundleCommand):
     pnpm tauri build

  3. Embed the widget extension into the .app bundle:
     ./${args.dir}/embed-widget.sh

  Or combine both in one command:
     pnpm tauri build && ./${args.dir}/embed-widget.sh

  Bundle ID:  ${bundleId}
  App Group:  ${appGroup}
`);
  },
});

// ─── init-ios ──────────────────────────────────────────────────────────────

const initIos = defineCommand({
  meta: {
    name: "init-ios",
    description: "Copy iOS widget extension template to your project",
  },
  args: {
    "app-group": {
      type: "string",
      description: "App Group identifier (e.g. group.com.example.myapp). Auto-detected from tauri.conf.json if omitted.",
    },
    dir: {
      type: "string",
      description: "Target directory for the widget Swift file (default: src-tauri/ios-widget)",
      default: "src-tauri/ios-widget",
    },
    force: {
      type: "boolean",
      description: "Overwrite existing files",
      default: false,
    },
  },
  run({ args }) {
    const cwd = process.cwd();
    const conf = readTauriConf(cwd);

    let appGroup = args["app-group"];

    if (!appGroup) {
      if (!conf) {
        console.error("ERROR: Could not find tauri.conf.json. Run from your Tauri project root, or pass --app-group explicitly.");
        process.exit(1);
      }
      const identifier = detectTauriIdentifier(conf);
      if (!identifier) {
        console.error("ERROR: tauri.conf.json is missing 'identifier'. Set it, or pass --app-group explicitly.");
        process.exit(1);
      }
      appGroup = "group." + identifier;
      console.log(`  Auto-detected app-group: ${appGroup}`);
    }

    const targetDir = resolve(cwd, args.dir);
    const templateDir = join(PLUGIN_ROOT, "templates", "ios-widget");

    console.log(`\nPreparing iOS widget template in ${args.dir}...\n`);

    mkdirSync(targetDir, { recursive: true });

    const templatePath = join(templateDir, "MyWidget.swift");
    const localTemplatePath = join(targetDir, "MyWidget.swift");

    if (!existsSync(localTemplatePath) || args.force) {
      copyTemplate(templatePath, localTemplatePath, { "group.com.example.myapp": appGroup });
      console.log("  Wrote ios-widget/MyWidget.swift");
    } else {
      console.log("  Kept existing ios-widget/MyWidget.swift (use --force to overwrite)");
    }

    const xcodeWidgetFiles = findIosWidgetEntryFiles(cwd);
    const widgetTargets = findIosWidgetTargetNames(cwd);
    if (xcodeWidgetFiles.length) {
      for (const filePath of xcodeWidgetFiles) {
        const rendered = renderIosWidgetSwift(templatePath, filePath, appGroup);
        writeFileSync(filePath, rendered, "utf-8");
        const pretty = filePath.replace(cwd + "/", "");
        console.log(`  Synced ${pretty}`);
      }
    } else {
      console.log("  No WidgetExtension swift file detected in src-tauri/gen/apple (create target in Xcode, then run init-ios again)");
    }

    console.log(`
Done! Next steps:

  1. Run: pnpm tauri ios init

  2. Open Xcode:
     open src-tauri/gen/apple/*.xcodeproj

  3. File → New → Target → Widget Extension

  4. Add TauriWidgets Swift Package:
     File → Add Package Dependencies → Add Local →
     select node_modules/tauri-plugin-widgets-api/swift/
     Add to Target: ${widgetTargets.length ? widgetTargets.join(", ") : "WidgetExtension (or WidgetExtensionExtension)"}

  5. Re-run this command after creating the Widget Extension target:
     npx tauri-plugin-widgets-api init-ios
     (it auto-syncs generated WidgetExtension.swift in src-tauri/gen/apple/*)

  6. Enable App Groups in BOTH targets with:
     ${appGroup}

  7. Run: pnpm tauri ios dev
`);
  },
});

// ─── Main ──────────────────────────────────────────────────────────────────

const main = defineCommand({
  meta: {
    name: "tauri-widgets",
    version: "0.3.0",
    description: "CLI for tauri-plugin-widgets — initialize native widget extensions",
  },
  subCommands: {
    "init-macos": initMacos,
    "init-ios": initIos,
  },
});

runMain(main);
