import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const genAdaptiveSnapshotsCommand = defineCommand({
  meta: {
    name: "gen-adaptive-snapshots",
    description: "Run scripts/sh/gen-adaptive-snapshots.sh",
  },
  args: {
    _: { type: "positional", description: "Extra args forwarded to the script", required: false },
  },
  run({ args }) {
    const extra = Array.isArray(args._) ? args._.map(String) : args._ ? [String(args._)] : [];
    runToolScript("scripts/sh/gen-adaptive-snapshots.sh", extra);
  },
});
