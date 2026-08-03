import { defineCommand, runMain } from "citty";
import { initUnified } from "./commands/init.js";
import { initMacos } from "./commands/init-macos.js";
import { initIos } from "./commands/init-ios.js";
import { initWindows } from "./commands/init-windows.js";
import { signing } from "./commands/signing.js";
import { doctor } from "./commands/doctor.js";
import { previewCmd } from "./commands/preview.js";
import { validateCmd } from "./commands/validate.js";
import { traceCmd } from "./commands/trace.js";
import { cleanCmd } from "./commands/clean.js";
import { toolsCmd } from "./commands/tools.js";

const main = defineCommand({
  meta: {
    name: "tauri-widgets",
    version: "0.5.0",
    description:
      "CLI for tauri-plugin-widgets: consumer commands (init, doctor, preview, …) plus `tools` for maintainer stands.",
  },
  subCommands: {
    init: initUnified,
    "init-macos": initMacos,
    "init-ios": initIos,
    "init-windows": initWindows,
    signing,
    doctor,
    preview: previewCmd,
    validate: validateCmd,
    trace: traceCmd,
    clean: cleanCmd,
    tools: toolsCmd,
  },
});

runMain(main);
