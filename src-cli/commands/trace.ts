import { defineCommand } from "citty";
import { resolve } from "node:path";
import { detectTauriIdentifier, readTauriConf } from "../lib/tauri-conf.js";
import { runTraceFollow } from "../lib/trace.js";

export const traceCmd = defineCommand({
  meta: {
    name: "trace",
    description: "Read / follow widget_trace.json from the host app data dir",
  },
  args: {
    cwd: {
      type: "positional",
      description: "App project root (default: cwd)",
      required: false,
    },
    follow: { type: "boolean", description: "Follow new events", default: false },
    lines: { type: "string", description: "Tail N events", default: "40" },
  },
  async run({ args }) {
    const cwd = resolve(args.cwd || process.cwd());
    const conf = readTauriConf(cwd);
    const identifier = detectTauriIdentifier(conf);
    await runTraceFollow({
      cwd,
      identifier,
      follow: Boolean(args.follow),
      lines: Number(args.lines) || 40,
    });
  },
});
