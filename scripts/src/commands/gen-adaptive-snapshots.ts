import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const genAdaptiveSnapshotsCommand = defineCommand({
  meta: {
    name: "gen-adaptive-snapshots",
    description: "Run tools/gen-adaptive-snapshots.sh",
  },
  args: {
    _: { type: "positional", description: "Extra args", required: false },
  },
  run({ args }) {
    const extra = Array.isArray(args._) ? args._.map(String) : args._ ? [String(args._)] : [];
    runToolScript("tools/gen-adaptive-snapshots.sh", extra);
  },
});
