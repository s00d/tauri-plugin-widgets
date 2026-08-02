import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const shotLinuxCommand = defineCommand({
  meta: { name: "shot-linux", description: "Run tools/shot-linux.sh" },
  args: {
    _: { type: "positional", description: "Extra args", required: false },
  },
  run({ args }) {
    const extra = Array.isArray(args._) ? args._.map(String) : args._ ? [String(args._)] : [];
    runToolScript("tools/shot-linux.sh", extra);
  },
});
