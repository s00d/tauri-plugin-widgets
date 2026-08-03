import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const linuxUpWlCommand = defineCommand({
  meta: { name: "linux-up-wl", description: "Run scripts/sh/linux-up-wl.sh" },
  args: {
    _: { type: "positional", description: "Extra args forwarded to the script", required: false },
  },
  run({ args }) {
    const extra = Array.isArray(args._) ? args._.map(String) : args._ ? [String(args._)] : [];
    runToolScript("scripts/sh/linux-up-wl.sh", extra);
  },
});
