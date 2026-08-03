import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const androidUpCommand = defineCommand({
  meta: { name: "android-up", description: "Run scripts/sh/android-up.sh" },
  args: {
    _: { type: "positional", description: "Extra args forwarded to the script", required: false },
  },
  run({ args }) {
    const extra = Array.isArray(args._) ? args._.map(String) : args._ ? [String(args._)] : [];
    runToolScript("scripts/sh/android-up.sh", extra);
  },
});
