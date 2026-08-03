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
import { winSyncCommand } from "./commands/win-sync.js";
import { winBootstrapCommand } from "./commands/win-bootstrap.js";
import { winPackCommand } from "./commands/win-pack.js";
import { winSideloadCommand } from "./commands/win-sideload.js";
import { buildExampleCommand } from "./commands/build-example.js";
import { shotLinuxCommand } from "./commands/shot-linux.js";
import { genAdaptiveSnapshotsCommand } from "./commands/gen-adaptive-snapshots.js";
import { hostsCommand } from "./commands/hosts.js";
import { shotCommand } from "./commands/shot.js";
import { testCommand } from "./commands/test-runner.js";

export const main = defineCommand({
  meta: {
    name: "@tauri-plugin-widgets/scripts",
    description: [
      "Maintainer tooling for tauri-plugin-widgets.",
      "",
      "From the repository root:",
      "  pnpm -C scripts cli <command> [flags]",
      "",
      "Or use root aliases: pnpm docs:generate, pnpm triage, pnpm hosts, …",
      "",
      "Shell/PowerShell stands live under scripts/sh/ and scripts/win/.",
    ].join("\n"),
  },
  subCommands: {
    "docs-generate": docsGenerateCommand,
    "docs-audit": docsAuditCommand,
    triage: triageCommand,
    "audit-sheets": auditSheetsCommand,
    hosts: hostsCommand,
    shot: shotCommand,
    test: testCommand,
    "ios-up": iosUpCommand,
    "android-up": androidUpCommand,
    "linux-up": linuxUpCommand,
    "linux-up-wl": linuxUpWlCommand,
    "win-up": winUpCommand,
    "win-sync": winSyncCommand,
    "win-bootstrap": winBootstrapCommand,
    "win-pack": winPackCommand,
    "win-sideload": winSideloadCommand,
    "build-example": buildExampleCommand,
    "shot-linux": shotLinuxCommand,
    "gen-adaptive-snapshots": genAdaptiveSnapshotsCommand,
  },
});
