import { spawnSync } from "node:child_process";
import { join } from "node:path";
import { defineCommand } from "citty";
import { repoRoot } from "../utils/workspace.js";

const PLATFORMS = ["desktop", "android", "macos", "geometry", "events"] as const;
type Platform = (typeof PLATFORMS)[number];

function run(
  command: string,
  args: string[],
  opts: { cwd?: string; env?: NodeJS.ProcessEnv } = {},
): void {
  const result = spawnSync(command, args, {
    cwd: opts.cwd ?? repoRoot(),
    stdio: "inherit",
    env: { ...process.env, ...opts.env },
  });
  if (result.error) throw result.error;
  const code = result.status ?? 1;
  if (code !== 0) process.exit(code);
}

function caseEnv(caseName: string | undefined): NodeJS.ProcessEnv {
  return caseName ? { CASE: caseName } : {};
}

function runDesktop(update: boolean, caseName: string | undefined): void {
  const env: NodeJS.ProcessEnv = {
    ...caseEnv(caseName),
    ...(update ? { UPDATE_SNAPSHOTS: "1" } : {}),
  };
  run("pnpm", ["exec", "playwright", "test", "-c", "tests/desktop/playwright.config.ts"], {
    env,
  });
}

function runEvents(update: boolean, caseName: string | undefined): void {
  const env: NodeJS.ProcessEnv = {
    ...caseEnv(caseName),
    ...(update ? { UPDATE_SNAPSHOTS: "1" } : {}),
  };
  run(
    "pnpm",
    [
      "exec",
      "playwright",
      "test",
      "-c",
      "tests/desktop/playwright.config.ts",
      "tests/desktop/events.spec.ts",
    ],
    { env },
  );
}

function runAndroid(update: boolean, caseName: string | undefined): void {
  const gradleArgs = [":testDebugUnitTest"];
  if (update) gradleArgs.push("-PupdateSnapshots=true");
  run("./gradlew", gradleArgs, {
    cwd: join(repoRoot(), "android"),
    env: caseEnv(caseName),
  });
}

function runMacos(update: boolean, caseName: string | undefined): void {
  const env: NodeJS.ProcessEnv = {
    ...caseEnv(caseName),
    ...(update ? { GOLDEN_RECORD: "1" } : {}),
  };
  run("swift", ["test", "--filter", "MacRenderTests"], {
    cwd: join(repoRoot(), "swift"),
    env,
  });
}

function runGeometry(_update: boolean, caseName: string | undefined): void {
  // Geometry update path is `pnpm test:desktop:update` (see package.json).
  if (_update) {
    runDesktop(true, caseName);
    return;
  }
  run("node", ["tests/desktop/geometry-diff.mjs"], { env: caseEnv(caseName) });
}

export const testCommand = defineCommand({
  meta: {
    name: "test",
    description:
      "Run maintainer test suites (desktop / android / macos / geometry / events)",
  },
  args: {
    platform: {
      type: "string",
      description: "desktop | android | macos | geometry | events",
      default: "desktop",
    },
    case: {
      type: "string",
      description: "Optional CASE= filter / golden case id",
    },
    update: {
      type: "boolean",
      default: false,
      description: "Update snapshots / record goldens where supported",
    },
  },
  run({ args }) {
    const platform = String(args.platform || "desktop") as Platform;
    if (!PLATFORMS.includes(platform)) {
      console.error(
        `test: unknown --platform "${platform}". Expected: ${PLATFORMS.join(", ")}`,
      );
      process.exit(2);
    }

    const caseName = args.case ? String(args.case) : undefined;
    const update = Boolean(args.update);

    switch (platform) {
      case "desktop":
        runDesktop(update, caseName);
        break;
      case "android":
        runAndroid(update, caseName);
        break;
      case "macos":
        runMacos(update, caseName);
        break;
      case "geometry":
        runGeometry(update, caseName);
        break;
      case "events":
        runEvents(update, caseName);
        break;
    }
  },
});
