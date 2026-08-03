import fs from "node:fs";
import { join } from "node:path";
import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";
import { repoRoot } from "../utils/workspace.js";

function readCaseSize(caseName: string): string {
  const path = join(repoRoot(), "tests/cases", `${caseName}.json`);
  if (!fs.existsSync(path)) {
    console.error(`shot: case not found: tests/cases/${caseName}.json`);
    process.exit(2);
  }
  try {
    const raw = JSON.parse(fs.readFileSync(path, "utf8")) as { size?: string };
    return raw.size || "small";
  } catch (e) {
    console.error(`shot: failed to parse ${path}:`, e instanceof Error ? e.message : e);
    process.exit(2);
  }
}

function printOtherPlatformHints(caseName: string, platform: string): void {
  console.log(`shot: platform "${platform}" is not auto-run from this CLI.`);
  console.log("Use one of:");
  switch (platform) {
    case "desktop":
      console.log(`  CASE=${caseName} GOLDEN_RECORD=1 pnpm test:visual:desktop`);
      console.log(`  just record-desktop ${caseName}`);
      break;
    case "android":
      console.log(`  just record-android ${caseName}`);
      console.log(`  just test-android-visual`);
      break;
    case "ios":
      console.log(`  just record-ios ${caseName}`);
      console.log(`  just test-ios-visual`);
      break;
    case "macos":
      console.log(`  just record-macos ${caseName}`);
      console.log(`  just test-macos-visual`);
      break;
    case "windows":
    case "win":
      console.log(`  just record-windows ${caseName}`);
      console.log(`  just test-windows-visual`);
      break;
    default:
      console.log(`  just shot-linux ${caseName}   # linux live PNG`);
      console.log(`  just record-desktop ${caseName}`);
      console.log(`  just record-android ${caseName}`);
      console.log(`  just record-ios ${caseName}`);
      console.log(`  just record-macos ${caseName}`);
      console.log(`  just record-windows ${caseName}`);
      break;
  }
}

export const shotCommand = defineCommand({
  meta: {
    name: "shot",
    description: "Capture a visual case (linux runs shot-linux; others print just/pnpm hints)",
  },
  args: {
    case: {
      type: "positional",
      description: "Case id under tests/cases/<case>.json",
      required: true,
    },
    platform: {
      type: "positional",
      description: "linux (default) | desktop | android | ios | macos | windows",
      required: false,
      default: "linux",
    },
  },
  run({ args }) {
    const caseName = String(args.case);
    const platform = String(args.platform || "linux");

    if (platform === "linux") {
      const size = readCaseSize(caseName);
      runToolScript("scripts/sh/shot-linux.sh", [caseName, size]);
      return;
    }

    printOtherPlatformHints(caseName, platform);
  },
});
