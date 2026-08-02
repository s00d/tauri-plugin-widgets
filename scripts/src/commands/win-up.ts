import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const winUpCommand = defineCommand({
  meta: { name: "win-up", description: "Run tools/win-up.sh" },
  args: {
    _: { type: "positional", description: "Extra args", required: false },
  },
  run({ args }) {
    const extra = Array.isArray(args._) ? args._.map(String) : args._ ? [String(args._)] : [];
    runToolScript("tools/win-up.sh", extra);
  },
});
