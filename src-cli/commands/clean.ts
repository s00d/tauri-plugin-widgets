import { defineCommand } from "citty";
import { resolve } from "node:path";
import { detectTauriIdentifier, readTauriConf } from "../lib/tauri-conf.js";
import { runClean } from "../lib/clean.js";

export const cleanCmd = defineCommand({
  meta: {
    name: "clean",
    description: "Clear widget store / trace / App Group JSON and/or image-prefetch cache",
  },
  args: {
    cwd: {
      type: "positional",
      description: "App project root (default: cwd)",
      required: false,
    },
    group: {
      type: "string",
      description: "App Group id (macOS Group Containers cleanup)",
    },
    cache: {
      type: "boolean",
      description: "Only clear image-prefetch disk cache",
      default: false,
    },
  },
  run({ args }) {
    const cwd = resolve(args.cwd || process.cwd());
    const conf = readTauriConf(cwd);
    const identifier = detectTauriIdentifier(conf);
    const group = args.group || conf?.data?.plugins?.widgets?.appGroup || undefined;
    runClean({
      cwd,
      identifier,
      group,
      cacheOnly: Boolean(args.cache),
    });
  },
});
