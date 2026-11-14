// Desktop live-activity island page (`widget.html?widgetId=activity.__island__`):
// compact pill around a notch, hover → expanded regions, taps report `activity.<id>`.
// The view mirrors Rust `island::View` for a 14" MacBook Pro notch (185 × 32).
import { test, expect, type Page } from "@playwright/test";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { REPO_ROOT, WIDGET_HTML } from "./helpers.js";

const activity = (id: string, extra: Record<string, unknown> = {}) => ({
  id,
  group: "g",
  widgetId: `activity.${id}`,
  url: null,
  keyline: "#34c759",
  regions: {
    compactLeading: { type: "image", systemName: "hammer", size: 15, color: "#34c759" },
    compactTrailing: { type: "text", content: "42%", fontSize: 13, fontWeight: "semibold", color: "#34c759" },
    minimal: { type: "image", systemName: "circle.fill", size: 15 },
    leading: { type: "text", content: "Build", fontSize: 14, fontWeight: "semibold" },
    trailing: { type: "text", content: "42%", fontSize: 14 },
    center: null,
    bottom: {
      type: "vstack",
      spacing: 6,
      padding: 12,
      children: [
        { type: "text", content: "Compiling tauri-plugin-widgets", fontSize: 13 },
        { type: "progress", value: 0.42, tint: "#34c759" },
        { type: "button", label: "Stop", action: "stop" },
      ],
    },
    ...extra,
  },
});

const VIEW = {
  expanded: false,
  flush: true,
  gap: 185,
  band: 32,
  compact: [329, 32],
  expandedSize: [400, 164],
  minimal: [] as number[],
  activities: [activity("build")],
};

async function stub(page: Page, view: unknown) {
  await page.addInitScript((v) => {
    const w = window as unknown as Record<string, unknown> & {
      __CALLS__: Array<{ cmd: string; args: Record<string, unknown> }>;
    };
    w.__CALLS__ = [];
    let current = JSON.parse(JSON.stringify(v));
    w.__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args: Record<string, unknown> = {}) => {
        w.__CALLS__.push({ cmd, args });
        if (cmd.endsWith("activity_island")) {
          if (typeof args.expanded === "boolean") current = { ...current, expanded: args.expanded };
          return Promise.resolve(current);
        }
        return Promise.resolve(null);
      },
      transformCallback: () => 1,
      metadata: { currentWindow: { label: "activity-_island" } },
    };
  }, view);
}

function islandUrl(): string {
  const q = new URLSearchParams({ group: "g", widgetId: "activity.__island__", size: "medium" });
  return `${pathToFileURL(WIDGET_HTML).href}?${q}`;
}

const shot = (name: string) => path.join(REPO_ROOT, "test-results", "island", name);

test("island: compact pill wraps the notch, hover expands, taps report activity id", async ({ page }) => {
  await page.setViewportSize({ width: 400, height: 164 });
  await stub(page, VIEW);
  await page.goto(islandUrl());
  const pill = page.locator(".isl");
  await expect(pill).toHaveCSS("width", "329px");
  await expect(pill).toHaveCSS("height", "32px");
  // Square top corners at the notch, keyline as an inset outline.
  await expect(pill).toHaveCSS("border-top-left-radius", "0px");
  expect(await pill.evaluate((e) => getComputedStyle(e).boxShadow)).toContain("inset");
  const pct = page.locator(".layer.c").getByText("42%");
  await expect(pct).toBeVisible();
  // Compact regions sit on the band's middle line, not pinned to its top.
  const tb = (await pct.boundingBox())!;
  expect(Math.abs(tb.y + tb.height / 2 - 16)).toBeLessThanOrEqual(2);
  await page.screenshot({ path: shot("compact.png"), omitBackground: true });

  await pill.hover();
  await expect(pill).toHaveClass(/open/);
  await expect(pill).toHaveCSS("width", "400px");
  await expect(pill).toHaveCSS("height", "164px");
  await expect(page.locator(".layer.x").getByText("Compiling tauri-plugin-widgets")).toBeVisible();
  await page.waitForTimeout(450);
  await page.screenshot({ path: shot("expanded.png"), omitBackground: true });

  await page.locator(".layer.x").getByText("Stop").click();
  const calls = await page.evaluate(() => (window as unknown as { __CALLS__: unknown[] }).__CALLS__);
  expect(calls).toContainEqual({ cmd: "plugin:widgets|activity_island", args: { expanded: true } });
  expect(calls).toContainEqual(
    expect.objectContaining({
      cmd: "plugin:widgets|widget_action",
      args: expect.objectContaining({ action: "stop", widgetId: "activity.build" }),
    }),
  );

  // Leaving the window collapses the pill, then gives the room back to the host.
  await page.mouse.move(399, 160);
  await page.dispatchEvent("html", "mouseleave");
  await expect(pill).not.toHaveClass(/open/);
  await expect
    .poll(async () => page.evaluate(() => (window as unknown as { __CALLS__: Array<{ args: { expanded?: boolean } }> }).__CALLS__.some((c) => c.args.expanded === false)))
    .toBe(true);
});

test("island: older activities are minimal circles; a click focuses them", async ({ page }) => {
  await page.setViewportSize({ width: 220 + 2 * 38, height: 32 });
  const plain = {
    ...VIEW,
    flush: false,
    gap: 36,
    compact: [180, 32],
    expandedSize: [400, 164],
    minimal: [180 / 2 + 6 + 16],
    activities: [activity("build"), activity("timer")],
  };
  await stub(page, plain);
  await page.goto(islandUrl());
  const pill = page.locator(".isl");
  // Floating pill: fully rounded.
  await expect(pill).toHaveCSS("border-top-left-radius", "16px");
  const mini = page.locator(".mini");
  await expect(mini).toHaveCount(1);
  const pb = (await pill.boundingBox())!;
  const mb = (await mini.boundingBox())!;
  expect(Math.round(mb.x - (pb.x + pb.width))).toBe(6);
  await page.screenshot({ path: shot("minimal.png"), omitBackground: true });
  await mini.click();
  const calls = await page.evaluate(() => (window as unknown as { __CALLS__: Array<{ args: unknown }> }).__CALLS__);
  expect(calls.map((c) => c.args)).toContainEqual({ focus: "timer" });
});
