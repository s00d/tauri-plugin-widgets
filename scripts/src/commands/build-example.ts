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
    // citty keeps all positionals on rawArgs; drop the subcommand name when present.
    const start = rawArgs[0] === "build-example" ? 1 : 0;
    const fromRaw = rawArgs.slice(start).map(String).filter(Boolean);
    const fromArgs = Array.isArray(args._)
      ? args._.map(String)
      : args._
        ? [String(args._)]
        : [];
    const extra = fromRaw.length > 0 ? fromRaw : fromArgs;
    runToolScript("scripts/sh/build-example-artifacts.sh", extra);
  },
});
