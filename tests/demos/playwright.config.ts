// Demo recordings for the docs (`pnpm docs:demos`): the real desktop renderer (widget.html)
// in headless Chromium with a stubbed Tauri host, recorded to video. Not part of the tests.
import { defineConfig } from "@playwright/test";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  testDir: root,
  timeout: 120_000,
  workers: 1,
  outputDir: path.resolve(root, "../../node_modules/.cache/demos"),
  reporter: "list",
  use: { colorScheme: "dark", locale: "en-US", timezoneId: "UTC", deviceScaleFactor: 2 },
  projects: [{ name: "chromium", use: { browserName: "chromium" } }],
});
