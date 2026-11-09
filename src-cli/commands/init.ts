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
  /** Comma list (macos,ios,windows). Default: the host OS, plus iOS when gen/apple exists. */
  platforms?: string;
}

export function initUnifiedAction(opts: InitUnifiedOptions): void {
  const cwd = process.cwd();
  if (!readTauriConf(cwd)) {
    console.error("ERROR: tauri.conf.json not found — run from a Tauri app root");
    process.exit(1);
  }
  const has = (p: string) => existsSync(join(cwd, "src-tauri", p));
  const wanted = new Set(
    opts.platforms
      ? opts.platforms.split(",").map((p) => p.trim().toLowerCase())
      : [
          ...(process.platform === "darwin" ? ["macos"] : []),
          ...(process.platform === "win32" ? ["windows"] : []),
          ...(has("gen/apple") ? ["ios"] : []),
        ],
  );

  let ran = 0;
  if (wanted.has("macos")) {
    if (has("macos-widget") && !opts.force) {
      console.log("  macos-widget exists — skip init-macos (--force to recreate)");
    } else {
      console.log(`\n==> init-macos`);
      initMacosAction({ appGroup: opts.appGroup, dir: "src-tauri/macos-widget", force: opts.force });
      ran++;
    }
  }
  if (wanted.has("ios")) {
    console.log(`\n==> init-ios`);
    initIosAction({ appGroup: opts.appGroup, dir: "src-tauri/ios-widget", force: opts.force });
    ran++;
  }
  if (wanted.has("windows")) {
    if (has("windows-widget") && !opts.force) {
      console.log("  windows-widget exists — skip init-windows (--force to recreate)");
    } else {
      console.log(`\n==> init-windows`);
      initWindowsAction({ dir: "src-tauri/windows-widget", force: opts.force });
      ran++;
    }
  }

  if (!ran) {
    console.log("Nothing to init. Pass --platforms macos,ios,windows (iOS needs `tauri ios init` first).");
  } else {
    console.log(`\nDone — ran ${ran} init(s). Check the setup with: npx tauri-widgets doctor`);
  }
}

export const initUnified = defineCommand({
  meta: {
    name: "init",
    description: "Set up native widget extensions (macOS / iOS / Windows)",
  },
  args: {
    platforms: { type: "string", description: "macos,ios,windows (default: host OS + iOS when gen/apple exists)" },
    force: { type: "boolean", description: "Recreate existing widget folders / templates", default: false },
    "app-group": { type: "string", description: "App Group id (default group.<identifier>)" },
  },
  run({ args }) {
    initUnifiedAction({
      force: args.force,
      appGroup: args["app-group"],
      platforms: args.platforms,
    });
  },
});
