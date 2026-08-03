import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";

export const buildExampleCommand = defineCommand({
  meta: {
    name: "build-example",
    description: "Run scripts/sh/build-example-artifacts.sh [platforms…]",
  },
  args: {
    _: {
      type: "positional",
      description: "Platforms (macos ios android linux windows)",
      required: false,
    },
  },
  run({ args, rawArgs }) {
    // citty keeps all positionals on rawArgs; platforms follow the verb.
    const fromRaw = rawArgs.slice(1).map(String).filter(Boolean);
    const fromArgs = Array.isArray(args._)
      ? args._.map(String)
      : args._
        ? [String(args._)]
        : [];
    const extra = fromRaw.length > 0 ? fromRaw : fromArgs;
    runToolScript("scripts/sh/build-example-artifacts.sh", extra);
  },
});
