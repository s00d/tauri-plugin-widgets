import { spawnSync } from "node:child_process";
import { defineCommand } from "citty";
import { runToolScript } from "../utils/run-tool-script.js";
import { repoRoot } from "../utils/workspace.js";

const PLATFORM_SCRIPTS: Record<string, string> = {
  ios: "scripts/sh/ios-up.sh",
  android: "scripts/sh/android-up.sh",
  linux: "scripts/sh/linux-up.sh",
  "linux-wl": "scripts/sh/linux-up-wl.sh",
  win: "scripts/sh/win-up.sh",
};

const ALL_PLATFORMS = Object.keys(PLATFORM_SCRIPTS);

function shellOk(command: string): boolean {
  const result = spawnSync("bash", ["-c", command], {
    cwd: repoRoot(),
    stdio: "ignore",
    env: process.env,
  });
  return (result.status ?? 1) === 0;
}

function status(): void {
  const rows: [string, boolean][] = [
    ["ios     ", shellOk("xcrun simctl list devices booted 2>/dev/null | grep -q Booted")],
    ["android ", shellOk("adb devices 2>/dev/null | grep -q emulator")],
    ["linux   ", shellOk("docker ps -q -f name=^/wshot$ | grep -q .")],
    ["linux-wl ", shellOk("docker ps -q -f name=^/wshot-wl$ | grep -q .")],
    ["windows ", shellOk("ssh -o ConnectTimeout=2 -o BatchMode=yes utm-win exit 2>/dev/null")],
    ["desktop ", shellOk("pgrep -f widget-probe >/dev/null 2>&1")],
  ];
  for (const [label, ok] of rows) {
    console.log(`${label}${ok ? "ok" : "DOWN"}`);
  }
}

function up(platforms: string[]): void {
  const selected = platforms.length > 0 ? platforms : ALL_PLATFORMS;
  for (const platform of selected) {
    const script = PLATFORM_SCRIPTS[platform];
    if (!script) {
      console.error(
        `hosts up: unknown platform "${platform}". Expected: ${ALL_PLATFORMS.join(", ")}`,
      );
      process.exit(2);
    }
    console.log(`hosts up: ${platform}`);
    runToolScript(script);
  }
}

function down(): void {
  const result = spawnSync("docker", ["rm", "-f", "wshot", "wshot-wl"], {
    cwd: repoRoot(),
    stdio: "inherit",
    env: process.env,
  });
  if (result.error) throw result.error;
  // Match Justfile `linux-down`: ignore missing containers.
}

export const hostsCommand = defineCommand({
  meta: {
    name: "hosts",
    description: "Bring visual-stand hosts up / show status / tear linux containers down",
  },
  args: {
    verb: {
      type: "positional",
      description: "up | status | down  (for up: optional platforms ios|android|linux|linux-wl|win)",
      required: true,
    },
  },
  run({ args, rawArgs }) {
    const verb = String(args.verb);
    // citty keeps all positionals on rawArgs; platforms follow the verb.
    const platforms = rawArgs.slice(1).map(String).filter(Boolean);

    if (verb === "up") {
      up(platforms);
      return;
    }
    if (verb === "status") {
      if (platforms.length > 0) {
        console.error("hosts status: unexpected extra arguments");
        process.exit(2);
      }
      status();
      return;
    }
    if (verb === "down") {
      if (platforms.length > 0) {
        console.error("hosts down: unexpected extra arguments");
        process.exit(2);
      }
      down();
      return;
    }

    console.error(`hosts: unknown verb "${verb}". Use: up | status | down`);
    process.exit(2);
  },
});
