import { defineCommand } from "citty";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { pluginRoot } from "../lib/paths.js";
import { detectTauriIdentifier, ensurePluginsWidgets, readTauriConf } from "../lib/tauri-conf.js";
import {
  findApplePbxproj,
  findIosWidgetEntryFiles,
  findIosWidgetTargetNames,
  patchIosCodeSignEntitlements,
  renderIosWidgetSwift,
} from "../lib/apple.js";
import {
  mergeAppGroupIntoEntitlements,
  resolveIosEntitlementsRels,
  writeEntitlementsFile,
} from "../lib/entitlements.js";
import { copyTemplate, widgetTemplateReplacements } from "../lib/templates.js";

export interface InitIosOptions {
  appGroup?: string;
  dir: string;
  force: boolean;
}

export function initIosAction(opts: InitIosOptions): void {
  const cwd = process.cwd();
  const conf = readTauriConf(cwd);

  const confAppGroup = conf?.data?.plugins?.widgets?.appGroup;
  let appGroup = opts.appGroup || confAppGroup;
  const identifier = conf ? detectTauriIdentifier(conf) : null;

  if (!appGroup) {
    if (!conf) {
      console.error("ERROR: Could not find tauri.conf.json. Run from your Tauri project root, or pass --app-group explicitly.");
      process.exit(1);
    }
    if (!identifier) {
      console.error("ERROR: tauri.conf.json is missing 'identifier'. Set it, or pass --app-group explicitly.");
      process.exit(1);
    }
    appGroup = "group." + identifier;
    console.log(`  Auto-detected app-group: ${appGroup}`);
  } else if (!opts.appGroup && confAppGroup) {
    console.log(`  Using app-group from plugins.widgets: ${appGroup}`);
  }

  const targetDir = resolve(cwd, opts.dir);
  const root = pluginRoot();
  const templateDir = join(root, "templates", "ios-widget");
  const appleDir = join(cwd, "src-tauri", "gen", "apple");
  const bundleId = identifier || "com.example.app";
  const widgetBundleId = `${bundleId}.WidgetExtension`;

  console.log(`\nPreparing iOS widget template in ${opts.dir}...\n`);

  mkdirSync(targetDir, { recursive: true });

  const templatePath = join(templateDir, "MyWidget.swift");
  const localTemplatePath = join(targetDir, "MyWidget.swift");

  if (!existsSync(localTemplatePath) || opts.force) {
    copyTemplate(templatePath, localTemplatePath, widgetTemplateReplacements(appGroup));
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

  // Entitlements + CODE_SIGN_ENTITLEMENTS (when gen/apple exists)
  if (existsSync(appleDir)) {
    const { appEntRel, widgetEntRel } = resolveIosEntitlementsRels(cwd, appleDir, widgetTargets);
    const appEntPath = join(appleDir, appEntRel);
    const widgetEntPath = join(appleDir, widgetEntRel);
    if (existsSync(appEntPath) && !opts.force) {
      mergeAppGroupIntoEntitlements(appEntPath, appGroup);
      console.log(`  Merged app-group into ${appEntRel}`);
    } else {
      writeEntitlementsFile(appEntPath, { appGroup, sandbox: false });
      console.log(`  Wrote ${appEntRel}`);
    }
    if (existsSync(widgetEntPath) && !opts.force) {
      mergeAppGroupIntoEntitlements(widgetEntPath, appGroup);
      console.log(`  Merged app-group into ${widgetEntRel}`);
    } else {
      writeEntitlementsFile(widgetEntPath, { appGroup, sandbox: false });
      console.log(`  Wrote ${widgetEntRel}`);
    }

    const widgetTargetFilter = (name: string) =>
      /Widget/i.test(name) || widgetTargets.some((t) => t === name);

    for (const pbx of findApplePbxproj(cwd)) {
      const { patched } = patchIosCodeSignEntitlements(
        pbx,
        {
          "com.apple.product-type.application": appEntRel,
          "com.apple.product-type.app-extension": widgetEntRel,
        },
        { targetNameFilter: widgetTargetFilter },
      );
      if (patched) {
        console.log(`  Patched CODE_SIGN_ENTITLEMENTS in ${basename(dirname(pbx))} (${patched} configs)`);
      } else {
        console.log(`  CODE_SIGN_ENTITLEMENTS already points at entitlements (${basename(dirname(pbx))})`);
      }
    }
  } else {
    console.log("  gen/apple missing — run `pnpm tauri ios init` then re-run init-ios for entitlements");
  }

  if (conf) {
    const result = ensurePluginsWidgets(conf, {
      appGroup,
      transport: "appGroup",
      force: true,
      explicitAppGroup: Boolean(opts.appGroup),
    });
    if (result.wrote) {
      console.log("  Wrote plugins.widgets (transport=appGroup — required on iOS)");
    } else if (conf.data?.plugins?.widgets) {
      console.log(`  Kept existing plugins.widgets (transport=${conf.data.plugins.widgets.transport})`);
    }
  }

  const portalIds = `https://developer.apple.com/account/resources/identifiers/list`;
  console.log(`
Done! Next steps:

  1. Run: pnpm tauri ios init   (if gen/apple is missing)

  2. Open Xcode:
     open src-tauri/gen/apple/*.xcodeproj

  3. File → New → Target → Widget Extension  (skip if already created)

  4. Add TauriWidgets Swift Package:
     File → Add Package Dependencies → Add Local →
     select node_modules/tauri-plugin-widgets-api/swift/
     Add to Target: ${widgetTargets.length ? widgetTargets.join(", ") : "WidgetExtension (or WidgetExtensionExtension)"}

  5. Re-run after creating the Widget Extension target:
     npx tauri-widgets init-ios
     (syncs Swift + entitlements + CODE_SIGN_ENTITLEMENTS)

  6. Developer portal (browser — two checkboxes):
     ${portalIds}
     • Register App Group:  ${appGroup}
     • App ID ${bundleId}           → enable App Groups → tick ${appGroup}
     • App ID ${widgetBundleId}     → enable App Groups → tick ${appGroup}

  7. Run: pnpm tauri ios dev

  Note: Widgets on a physical iOS device require a paid Apple Developer account —
  free provisioning cannot grant App Groups. Simulator works without a paid seat.
`);
}

export const initIos = defineCommand({
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
    initIosAction({
      appGroup: args["app-group"],
      dir: args.dir,
      force: args.force,
    });
  },
});
