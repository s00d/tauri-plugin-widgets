import fs from "node:fs";
import path from "node:path";
import { FIXTURES_ROOT, REPO_ROOT } from "./helpers.js";

export const CASES_ROOT = path.join(REPO_ROOT, "tests/cases");
export const GOLDEN_ROOT = path.join(REPO_ROOT, "tests/golden");
export const OUT_ROOT = path.join(REPO_ROOT, "out");

export type VisualCase = {
  name: string;
  fixture: string;
  size: "small" | "medium" | "large" | string;
  theme: "light" | "dark" | string;
  locale: string;
  fixturePath: string;
  /** Native macOS WidgetKit canvas (not iPhone / not desktop WKWebView). */
  macos?: { width: number; height: number };
};

/** darwin → desktop; linux → linux; win32 → windows */
export function goldenPlatform(): "desktop" | "linux" | "windows" {
  switch (process.platform) {
    case "linux":
      return "linux";
    case "win32":
      return "windows";
    default:
      return "desktop";
  }
}

export function goldenPath(caseName: string, platform = goldenPlatform()): string {
  return path.join(GOLDEN_ROOT, platform, `${caseName}.png`);
}

export function outPaths(caseName: string, platform = goldenPlatform()) {
  const dir = path.join(OUT_ROOT, platform);
  return {
    dir,
    actual: path.join(dir, `${caseName}.actual.png`),
    diff: path.join(dir, `${caseName}.diff.png`),
  };
}

export function loadCases(filter?: string | null): VisualCase[] {
  if (!fs.existsSync(CASES_ROOT)) return [];
  const names = fs
    .readdirSync(CASES_ROOT)
    .filter((f) => f.endsWith(".json"))
    .map((f) => f.replace(/\.json$/, ""))
    .sort();
  const out: VisualCase[] = [];
  for (const name of names) {
    if (filter && name !== filter) continue;
    const raw = JSON.parse(fs.readFileSync(path.join(CASES_ROOT, `${name}.json`), "utf8")) as {
      fixture: string;
      size: string;
      theme?: string;
      locale?: string;
      macos?: { width: number; height: number };
    };
    const fixturePath = path.join(FIXTURES_ROOT, `${raw.fixture}.json`);
    if (!fs.existsSync(fixturePath)) {
      throw new Error(`case ${name}: fixture missing at ${fixturePath}`);
    }
    out.push({
      name,
      fixture: raw.fixture,
      size: raw.size,
      theme: raw.theme ?? "dark",
      locale: raw.locale ?? "en_US",
      fixturePath,
      macos: raw.macos,
    });
  }
  return out;
}

export const CASE_FILTER = process.env.CASE?.trim() || null;
export const GOLDEN_RECORD =
  process.env.GOLDEN_RECORD === "1" || process.env.GOLDEN_RECORD === "true";
/** Migration-only bulk record; day-to-day requires CASE= with GOLDEN_RECORD. */
export const GOLDEN_RECORD_ALL = process.env.GOLDEN_RECORD_ALL === "1";

/** Fixed wall-clock for date/timer fixtures (UTC). */
export const FIXED_NOW_MS = Date.UTC(2026, 7, 1, 12, 0, 0);
