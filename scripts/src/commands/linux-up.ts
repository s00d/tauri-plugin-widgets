import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const linuxUpCommand = defineCommand({
  meta: { name: "linux-up", description: "Run tools/linux-up.sh" },
  args: {
    _: { type: "positional", description: "Extra args", required: false },
  },
  run({ args }) {
    const extra = Array.isArray(args._) ? args._.map(String) : args._ ? [String(args._)] : [];
    runToolScript("tools/linux-up.sh", extra);
  },
});
