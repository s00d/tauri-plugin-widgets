import { defineCommand } from "citty";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { readTauriConf } from "../lib/tauri-conf.js";
import { initMacosAction } from "./init-macos.js";
import { initIosAction } from "./init-ios.js";
import { initWindowsAction } from "./init-windows.js";

export interface InitUnifiedOptions {
  force: boolean;
  appGroup?: string;
}

export function initUnifiedAction(opts: InitUnifiedOptions): void {
  const cwd = process.cwd();
  const conf = readTauriConf(cwd);
  if (!conf) {
    console.error("ERROR: tauri.conf.json not found — run from a Tauri app root");
    process.exit(1);
  }
  const bundle = conf.data?.bundle || {};
  const targets = new Set<string>(
    ([] as string[])
      .concat(bundle.active || [])
      .concat(Array.isArray(bundle.targets) ? bundle.targets : [])
      .map(String),
  );
  // Heuristics
  const wantMac =
    targets.has("macos") ||
    existsSync(join(cwd, "src-tauri", "macos-widget")) ||
    process.platform === "darwin";
  const wantIos =
    targets.has("ios") ||
    existsSync(join(cwd, "src-tauri", "gen", "apple")) ||
    existsSync(join(cwd, "src-tauri", "ios-widget"));
  const wantWin = targets.has("windows") || existsSync(join(cwd, "src-tauri", "windows-widget"));

  let ran = 0;
  if (wantMac && !existsSync(join(cwd, "src-tauri", "macos-widget"))) {
    console.log(`\n==> init-macos`);
    initMacosAction({
      appGroup: opts.appGroup,
      dir: "src-tauri/macos-widget",
      force: opts.force,
      applySigning: true,
    });
    ran++;
  } else if (wantMac) {
    console.log("  macos-widget exists — skip init-macos (use --force via init-macos directly)");
  }
  if (wantIos) {
    console.log(`\n==> init-ios`);
    initIosAction({
      appGroup: opts.appGroup,
      dir: "src-tauri/ios-widget",
      force: opts.force,
    });
    ran++;
  }
  if (wantWin && !existsSync(join(cwd, "src-tauri", "windows-widget"))) {
    console.log(`\n==> init-windows`);
    initWindowsAction({
      dir: "src-tauri/windows-widget",
      force: opts.force,
    });
    ran++;
  } else if (wantWin) {
    console.log("  windows-widget exists — skip init-windows");
  }

  if (!ran) {
    console.log("Nothing to init. Try: npx tauri-widgets init-macos | init-ios | init-windows");
  } else {
    console.log(`\nDone — ran ${ran} init(s). Next: npx tauri-widgets signing && npx tauri-widgets doctor`);
  }
}

export const initUnified = defineCommand({
  meta: {
    name: "init",
    description: "Detect platforms and run init-macos / init-ios / init-windows as needed",
  },
  args: {
    force: { type: "boolean", description: "Forward --force to platform inits", default: false },
    "app-group": { type: "string", description: "App Group id" },
  },
  run({ args }) {
    initUnifiedAction({
      force: args.force,
      appGroup: args["app-group"],
    });
  },
});
