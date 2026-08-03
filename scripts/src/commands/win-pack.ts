import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const winPackCommand = defineCommand({
  meta: { name: "win-pack", description: "Run scripts/sh/win-pack.sh" },
  args: {
    _: { type: "positional", description: "Extra args forwarded to the script", required: false },
  },
  run({ args }) {
    const extra = Array.isArray(args._) ? args._.map(String) : args._ ? [String(args._)] : [];
    runToolScript("scripts/sh/win-pack.sh", extra);
  },
});
