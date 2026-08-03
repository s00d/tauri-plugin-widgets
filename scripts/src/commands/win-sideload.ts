import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const winSideloadCommand = defineCommand({
  meta: { name: "win-sideload", description: "Run scripts/sh/win-sideload.sh" },
  args: {
    _: { type: "positional", description: "Extra args forwarded to the script", required: false },
  },
  run({ args }) {
    const extra = Array.isArray(args._) ? args._.map(String) : args._ ? [String(args._)] : [];
    runToolScript("scripts/sh/win-sideload.sh", extra);
  },
});
