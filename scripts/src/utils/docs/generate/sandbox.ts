// @ts-nocheck
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative } from "node:path";
import {
  ALL_PLATFORMS,
  PUBLIC,
  ROOT,
} from "./shared.js";
export function copyShots(report, check) {
  const dest = join(PUBLIC, "shots");
  if (existsSync(dest)) rmSync(dest, { recursive: true, force: true });
  mkdirSync(dest, { recursive: true });
  for (const platform of ALL_PLATFORMS) {
    const src = join(ROOT, "tests/golden", platform);
    if (!existsSync(src)) continue;
    for (const file of readdirSync(src)) {
      if (!file.endsWith(".png")) continue;
      const to = join(dest, platform, file);
      mkdirSync(dirname(to), { recursive: true });
      copyFileSync(join(src, file), to);
      report.written.push(relative(ROOT, to));
    }
  }
  void check;
}

export function copySchemas(report, check) {
  const srcDir = join(ROOT, "schemas");
  const destDir = join(PUBLIC, "schemas");
  mkdirSync(destDir, { recursive: true });
  for (const name of ["widget-config.v1.json", "plugin-config.v1.json", "capabilities.json"]) {
    const from = join(srcDir, name);
    const to = join(destDir, name);
    if (!existsSync(from)) {
      report.stale.push(`missing schema ${name}`);
      continue;
    }
    const body = readFileSync(from, "utf8");
    if (check) {
      if (!existsSync(to) || readFileSync(to, "utf8") !== body) {
        report.stale.push(`docs/public/schemas/${name}`);
      }
    } else {
      writeFileSync(to, body, "utf8");
      report.written.push(`docs/public/schemas/${name}`);
    }
  }
}

