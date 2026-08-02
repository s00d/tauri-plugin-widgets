import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const linuxUpWlCommand = defineCommand({
  meta: { name: "linux-up-wl", description: "Run tools/linux-up-wl.sh" },
  args: {
    _: { type: "positional", description: "Extra args", required: false },
  },
  run({ args }) {
    const extra = Array.isArray(args._) ? args._.map(String) : args._ ? [String(args._)] : [];
    runToolScript("tools/linux-up-wl.sh", extra);
  },
});
