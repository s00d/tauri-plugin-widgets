import { test, expect } from "@playwright/test";
import fs from "node:fs";
import path from "node:path";
import {
  SIZE_VIEWPORTS,
  UPDATE_SNAPSHOTS,
  assertNoLiteralNullText,
  collectTexts,
  dumpTree,
  eventPixelPath,
  hideChrome,
  installActionMap,
  listEventScenarios,
  normalizeTree,
  stubTauriSimple,
  waitForRender,
  widgetHtmlUrl,
} from "./event-helpers.js";

const scenarios = listEventScenarios();

for (const sc of scenarios) {
  test(`event ${sc.id} [${sc.size}]`, async ({ page }) => {
    const vp = SIZE_VIEWPORTS[sc.size] ?? SIZE_VIEWPORTS.small;
    await page.setViewportSize(vp);
    await stubTauriSimple(page, sc.before);
    await page.goto(widgetHtmlUrl(sc.size));
    await waitForRender(page);
    await installActionMap(page, { [sc.action]: sc.after });
    await hideChrome(page);

    const beforeTree = normalizeTree(await dumpTree(page));
    expect(beforeTree).not.toBeNull();
    assertNoLiteralNullText(beforeTree!);
    const beforeTexts = collectTexts(beforeTree!);
    for (const t of sc.assertBeforeTexts) {
      expect(beforeTexts, `before missing "${t}"`).toContain(t);
    }

    const beforePng = eventPixelPath(sc.id, "before");
    if (UPDATE_SNAPSHOTS) {
      fs.mkdirSync(path.dirname(beforePng), { recursive: true });
      await page.locator("#root").screenshot({ path: beforePng, omitBackground: true });
    }

    // Prefer click by visible text; fall back to action attribute scan via evaluate.
    const clicked = await page.evaluate((clickText) => {
      const root = document.getElementById("root");
      if (!root) return false;
      const all = [...root.querySelectorAll("*")];
      const el = all.find((n) => (n.textContent || "").trim() === clickText);
      if (el && el instanceof HTMLElement) {
        el.click();
        return true;
      }
      return false;
    }, sc.clickText);
    expect(clicked, `click target "${sc.clickText}"`).toBe(true);

    // Wait for re-render after config push
    await page.waitForFunction(
      (afterNeedle) => {
        const root = document.getElementById("root");
        if (!root) return false;
        const txt = root.innerText || "";
        return afterNeedle.every((t: string) => txt.includes(t));
      },
      sc.assertAfterTexts,
      { timeout: 5000 },
    );

    const afterTree = normalizeTree(await dumpTree(page));
    expect(afterTree).not.toBeNull();
    assertNoLiteralNullText(afterTree!);
    const afterTexts = collectTexts(afterTree!);
    for (const t of sc.assertAfterTexts) {
      expect(afterTexts, `after missing "${t}"`).toContain(t);
    }
    for (const t of sc.assertGoneAfter ?? []) {
      expect(afterTexts, `after still has "${t}"`).not.toContain(t);
    }

    const actions = await page.evaluate(
      () => (window as unknown as { __ACTIONS__: Array<{ action: string }> }).__ACTIONS__,
    );
    expect(actions.map((a) => a.action)).toContain(sc.action);

    const afterPng = eventPixelPath(sc.id, "after");
    if (UPDATE_SNAPSHOTS) {
      fs.mkdirSync(path.dirname(afterPng), { recursive: true });
      await page.locator("#root").screenshot({ path: afterPng, omitBackground: true });
    } else if (fs.existsSync(beforePng) && fs.existsSync(afterPng)) {
      // Soft invariant: before/after screenshots differ when action mutates UI.
      const b = fs.readFileSync(beforePng);
      const a = fs.readFileSync(afterPng);
      expect(Buffer.compare(b, a) !== 0, "before/after PNG identical").toBe(true);
    }
  });
}

test("event corpus non-empty", () => {
  expect(scenarios.length, "add fixtures under tests/fixtures/events/").toBeGreaterThan(0);
});
