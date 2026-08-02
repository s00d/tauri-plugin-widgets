import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const androidUpCommand = defineCommand({
  meta: { name: "android-up", description: "Run tools/android-up.sh" },
  args: {
    _: { type: "positional", description: "Extra args", required: false },
  },
  run({ args }) {
    const extra = Array.isArray(args._) ? args._.map(String) : args._ ? [String(args._)] : [];
    runToolScript("tools/android-up.sh", extra);
  },
});
