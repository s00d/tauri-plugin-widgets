import { defineCommand } from "citty";
import { docsAuditCommand } from "./commands/docs-audit.js";
import { docsGenerateCommand } from "./commands/docs-generate.js";
import { triageCommand } from "./commands/triage.js";
import { auditSheetsCommand } from "./commands/audit-sheets.js";
import { iosUpCommand } from "./commands/ios-up.js";
import { androidUpCommand } from "./commands/android-up.js";
import { linuxUpCommand } from "./commands/linux-up.js";
import { linuxUpWlCommand } from "./commands/linux-up-wl.js";
import { winUpCommand } from "./commands/win-up.js";
import { buildExampleCommand } from "./commands/build-example.js";
import { shotLinuxCommand } from "./commands/shot-linux.js";
import { genAdaptiveSnapshotsCommand } from "./commands/gen-adaptive-snapshots.js";

export const main = defineCommand({
  meta: {
    name: "@tauri-plugin-widgets/scripts",
    description: [
      "Maintainer tooling for tauri-plugin-widgets.",
      "",
      "From the repository root:",
      "  pnpm -C scripts cli <command> [flags]",
      "",
      "Or use root aliases: pnpm docs:generate, pnpm triage, …",
    ].join("\n"),
  },
  subCommands: {
    "docs-generate": docsGenerateCommand,
    "docs-audit": docsAuditCommand,
    triage: triageCommand,
    "audit-sheets": auditSheetsCommand,
    "ios-up": iosUpCommand,
    "android-up": androidUpCommand,
    "linux-up": linuxUpCommand,
    "linux-up-wl": linuxUpWlCommand,
    "win-up": winUpCommand,
    "build-example": buildExampleCommand,
    "shot-linux": shotLinuxCommand,
    "gen-adaptive-snapshots": genAdaptiveSnapshotsCommand,
  },
});
