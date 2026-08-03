import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const winSyncCommand = defineCommand({
  meta: { name: "win-sync", description: "Run scripts/win/sync.sh" },
  args: {
    _: { type: "positional", description: "Extra args forwarded to the script", required: false },
  },
  run({ args }) {
    const extra = Array.isArray(args._) ? args._.map(String) : args._ ? [String(args._)] : [];
    runToolScript("scripts/win/sync.sh", extra);
  },
});
