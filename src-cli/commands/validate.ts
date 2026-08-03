import { defineCommand } from "citty";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { loadCapabilities, validateWidgetConfig } from "../lib/validate.js";

const SCHEMA_URL = "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json";

export const validateCmd = defineCommand({
  meta: {
    name: "validate",
    description: "Validate widget config against schemas/capabilities.json",
  },
  args: {
    config: {
      type: "positional",
      description: "Path to widget config JSON",
      required: true,
    },
    platforms: {
      type: "string",
      description: "Comma list: ios,android,macos,desktop,windows",
      default: "ios,android",
    },
  },
  run({ args }) {
    const abs = resolve(args.config);
    if (!existsSync(abs)) {
      console.error(`ERROR: ${abs} not found`);
      process.exit(1);
    }
    let config: unknown;
    try {
      config = JSON.parse(readFileSync(abs, "utf-8"));
    } catch (e) {
      console.error(`ERROR: invalid JSON: ${(e as Error).message}`);
      process.exit(1);
    }
    const platforms = String(args.platforms || "ios,android")
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);
    const { path: capsPath, data: caps } = loadCapabilities();
    console.log(`capabilities  ${capsPath}`);
    console.log(`schema        widget-config shape (small|medium|large + types)`);
    const findings = validateWidgetConfig(config, platforms, caps);
    let errors = 0;
    if (!findings.length) {
      console.log(`✓ schema + capabilities OK for ${platforms.join(",")}`);
    }
    for (const f of findings) {
      const mark = f.level === "error" ? "✗" : "⚠";
      const note = f.note ? ` — ${f.note}` : "";
      console.log(`${mark} ${f.path || "(root)"}/${f.type} — ${f.platform}: ${f.support}${note}`);
      if (f.level === "error") errors++;
    }
    if (!(config as { $schema?: string })?.$schema) {
      console.log(`⚠ missing $schema — add: "${SCHEMA_URL}"`);
    } else {
      console.log(`✓ $schema present`);
    }
    process.exitCode = errors ? 1 : 0;
  },
});
