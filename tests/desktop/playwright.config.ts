// @ts-check
import { defineConfig } from "@playwright/test";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(root, "../..");
const outputDir = path.join(repo, "test-results");

export default defineConfig({
  testDir: root,
  timeout: 30_000,
  expect: { timeout: 10_000 },
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  outputDir,
  // CI: annotations + a JSON report; `flaky-report.mjs` lists tests that passed only on retry.
  reporter: process.env.CI
    ? [["github"], ["list"], ["json", { outputFile: path.join(outputDir, "results.json") }]]
    : "list",
  use: {
    viewport: { width: 170, height: 170 },
    colorScheme: "dark",
    // UTC like the Swift tests and the emulator (`-timezone Etc/UTC`):
    // `date-timer` (`2099-06-15T14:30:00Z`) shows `02:30 PM` on every port.
    locale: "en-US",
    timezoneId: "UTC",
  },
  projects: [{ name: "chromium", use: { browserName: "chromium" } }],
});
