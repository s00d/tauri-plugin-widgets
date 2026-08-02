import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const buildExampleCommand = defineCommand({
  meta: {
    name: "build-example",
    description: "Run tools/build-example-artifacts.sh [platforms…]",
  },
  args: {
    _: {
      type: "positional",
      description: "Platforms (e.g. macos ios android)",
      required: false,
    },
  },
  run({ args }) {
    const extra = Array.isArray(args._) ? args._.map(String) : args._ ? [String(args._)] : [];
    runToolScript("tools/build-example-artifacts.sh", extra);
  },
});
