import { test, expect, type Page } from "@playwright/test";
import fs from "node:fs";
import path from "node:path";
import {
  SIZE_VIEWPORTS,
  UPDATE_SNAPSHOTS,
  assertNoLiteralNullText,
  diffTrees,
  geometryPath,
  listFixtures,
  normalizeTree,
  pixelPath,
  widgetHtmlUrl,
  type GeometryNode,
} from "./helpers.js";

async function stubTauri(page: Page, fixture: unknown) {
  await page.addInitScript((cfg) => {
    // Freeze clock so timer fixtures produce stable geometry + pixels.
    const FIXED_NOW = Date.UTC(2026, 7, 1, 12, 0, 0);
    Date.now = () => FIXED_NOW;

    (window as unknown as { __FIXTURE__: unknown }).__FIXTURE__ = cfg;
    const listeners: Record<string, Array<(e: unknown) => void>> = {};
    (window as unknown as { __TAURI_INTERNALS__: unknown }).__TAURI_INTERNALS__ = {
      invoke: (cmd: string) => {
        if (String(cmd).includes("get_widget_config")) {
          return Promise.resolve((window as unknown as { __FIXTURE__: unknown }).__FIXTURE__);
        }
        if (String(cmd).includes("listen")) {
          return Promise.resolve(1);
        }
        return Promise.resolve(null);
      },
      transformCallback: (f: (e: unknown) => void) => {
        const id = Object.keys(listeners).length + 1;
        listeners[String(id)] = [f];
        return id;
      },
      metadata: { currentWindow: { label: "test" } },
    };
  }, fixture);
}

/** Dump layout tree from #root widget content (skip chrome). */
async function dumpTree(page: Page): Promise<GeometryNode> {
  return page.evaluate(() => {
    const isChrome = (el: Element) =>
      el.id === "drag-handle" || el.id === "close-btn";

    const walk = (el: Element): {
      kind: string;
      rect: [number, number, number, number];
      text: string | null;
      px: number | null;
      color: string | null;
      children: ReturnType<typeof walk>[];
    } => {
      const r = el.getBoundingClientRect();
      const cs = getComputedStyle(el);
      const text =
        el.children.length === 0
          ? (el.textContent || "").trim() || null
          : null;
      return {
        kind: el.tagName.toLowerCase(),
        rect: [r.x | 0, r.y | 0, r.width | 0, r.height | 0],
        text,
        px: text ? parseFloat(cs.fontSize) || null : null,
        color: text ? cs.color : null,
        children: [...el.children].filter((c) => !isChrome(c)).map(walk),
      };
    };
    const root = document.getElementById("root");
    if (!root) throw new Error("#root missing");
    // Prefer first content child that is not chrome (#drag-handle / #close-btn)
    const content =
      [...root.children].find(
        (c) => !isChrome(c) && !c.classList.contains("w-empty"),
      ) || [...root.children].find((c) => !isChrome(c));
    if (!content) throw new Error("no content under #root");
    return walk(content);
  });
}

async function waitForRender(page: Page) {
  await page.waitForFunction(() => {
    const root = document.getElementById("root");
    if (!root) return false;
    if (root.querySelector(".w-err")) return true;
    if (root.querySelector(".w-empty")) return false;
    return root.children.length > 0;
  });
  const err = await page.locator(".w-err").count();
  if (err) {
    const msg = await page.locator(".w-err").innerText();
    throw new Error(`widget render error: ${msg}`);
  }
}

const fixtures = listFixtures();

for (const fx of fixtures) {
  for (const size of fx.sizes) {
    test(`desktop geometry ${fx.id} [${size}]`, async ({ page }) => {
      const fixture = JSON.parse(fs.readFileSync(fx.absPath, "utf8"));
      const vp = SIZE_VIEWPORTS[size] ?? SIZE_VIEWPORTS.small;
      await page.setViewportSize(vp);
      await stubTauri(page, fixture);
      await page.goto(widgetHtmlUrl(size));
      await waitForRender(page);

      const raw = await dumpTree(page);
      const tree = normalizeTree(raw);
      expect(tree).not.toBeNull();
      assertNoLiteralNullText(tree!);

      // Literal "null" regression for bugs/null-fields
      if (fx.id.includes("null-fields")) {
        const html = await page.content();
        expect(html).not.toMatch(/>\s*null\s*</);
      }

      const expectedFile = geometryPath(fx.id, size, "desktop");
      const pngFile = pixelPath(fx.id, size, "desktop");

      // Hide desktop chrome so goldens are widget-only.
      await page.addStyleTag({
        content: `#drag-handle,#close-btn{display:none!important}`,
      });

      if (UPDATE_SNAPSHOTS) {
        fs.mkdirSync(path.dirname(expectedFile), { recursive: true });
        fs.writeFileSync(expectedFile, JSON.stringify(tree, null, 2) + "\n");
        fs.mkdirSync(path.dirname(pngFile), { recursive: true });
        await page.locator("#root").screenshot({ path: pngFile, omitBackground: true });
      } else if (fs.existsSync(expectedFile)) {
        const expected = JSON.parse(fs.readFileSync(expectedFile, "utf8")) as GeometryNode;
        const errs = diffTrees(tree!, expected, 2);
        expect(errs, errs.join("\n")).toEqual([]);
      } else {
        test.info().annotations.push({
          type: "note",
          description: `no baseline yet: ${expectedFile} (run pnpm test:desktop:update)`,
        });
      }
    });
  }
}
