import { test, expect, type Page } from "@playwright/test";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {
  CASE_FILTER,
  FIXED_NOW_MS,
  GOLDEN_RECORD,
  GOLDEN_RECORD_ALL,
  goldenPath,
  loadCases,
  outPaths,
} from "./cases.js";
import {
  SIZE_VIEWPORTS,
  UPDATE_SNAPSHOTS,
  assertNoLiteralNullText,
  diffTrees,
  geometryPath,
  normalizeTree,
  widgetHtmlUrl,
  type GeometryNode,
} from "./helpers.js";

async function stubTauri(page: Page, fixture: unknown, theme: string) {
  await page.addInitScript((fixedNow) => {
    Date.now = () => fixedNow;
  }, FIXED_NOW_MS);

  await page.addInitScript((th) => {
    document.documentElement.dataset.theme = th;
    document.documentElement.style.colorScheme = th === "light" ? "light" : "dark";
  }, theme);

  await page.addInitScript((cfg) => {
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

async function dumpTree(page: Page): Promise<GeometryNode> {
  return page.evaluate(() => {
    const isChrome = (el: Element) => el.id === "drag-handle" || el.id === "close-btn";
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
      const text = el.children.length === 0 ? (el.textContent || "").trim() || null : null;
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
    const content =
      [...root.children].find((c) => !isChrome(c) && !c.classList.contains("w-empty")) ||
      [...root.children].find((c) => !isChrome(c));
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

function isUniformPng(buf: Buffer): boolean {
  // Cheap check: all non-header bytes identical in a sample — rely on hash stability mostly.
  // Prefer comparing PNG buffers via hash; uniform detection via sharp would be heavier.
  // Sample every 64th byte after signature.
  if (buf.length < 100) return true;
  const first = buf[64];
  for (let i = 64; i < buf.length; i += 97) {
    if (buf[i] !== first) return false;
  }
  return false;
}

/** Poll #root screenshots until two consecutive hashes match and buffer is non-trivial. */
async function awaitStable(page: Page, deadlineMs = 15_000): Promise<Buffer> {
  const loc = page.locator("#root");
  const t0 = Date.now();
  let prev: string | null = null;
  let last: Buffer | null = null;
  while (Date.now() - t0 < deadlineMs) {
    const buf = await loc.screenshot({ omitBackground: true });
    last = buf;
    const h = crypto.createHash("sha256").update(buf).digest("hex");
    if (h === prev && buf.length > 200 && !isUniformPng(buf)) {
      return buf;
    }
    prev = h;
    await page.waitForTimeout(250);
  }
  throw new Error(`widget #root did not stabilize within ${deadlineMs}ms`);
}

function mismatchFraction(a: Buffer, b: Buffer): number {
  // Structural: if lengths differ a lot, fail. Byte-level on PNG compressed data is
  // unstable; decode via playwright's raw is better — use exact buffer equality first,
  // then soft byte compare on compressed streams is useless.
  // For pixel compare we use sharp when available; else require identical buffers.
  return a.equals(b) ? 0 : 1;
}

async function pixelMismatch(a: Buffer, b: Buffer): Promise<{ frac: number; diff?: Buffer }> {
  try {
    const sharp = (await import("sharp")).default;
    const ia = sharp(a).ensureAlpha().raw();
    const ib = sharp(b).ensureAlpha().raw();
    const [ma, mb] = await Promise.all([ia.metadata(), ib.metadata()]);
    if (ma.width !== mb.width || ma.height !== mb.height) {
      return { frac: 1 };
    }
    const [pa, pb] = await Promise.all([ia.toBuffer(), ib.toBuffer()]);
    const n = pa.length / 4;
    let bad = 0;
    const diff = Buffer.alloc(pa.length);
    for (let i = 0; i < pa.length; i += 4) {
      const dr = Math.abs(pa[i] - pb[i]);
      const dg = Math.abs(pa[i + 1] - pb[i + 1]);
      const db = Math.abs(pa[i + 2] - pb[i + 2]);
      const da = Math.abs(pa[i + 3] - pb[i + 3]);
      const hit = Math.max(dr, dg, db, da) > 8;
      if (hit) bad++;
      diff[i] = hit ? 255 : 20;
      diff[i + 1] = hit ? 0 : 20;
      diff[i + 2] = hit ? 255 : 20;
      diff[i + 3] = 255;
    }
    const diffPng = await sharp(diff, {
      raw: { width: ma.width!, height: ma.height!, channels: 4 },
    })
      .png()
      .toBuffer();
    return { frac: bad / n, diff: diffPng };
  } catch {
    return { frac: mismatchFraction(a, b) };
  }
}

const cases = loadCases(CASE_FILTER);
if (GOLDEN_RECORD && !CASE_FILTER && !GOLDEN_RECORD_ALL) {
  throw new Error("GOLDEN_RECORD=1 requires CASE=<name> (one case at a time)");
}

for (const c of cases) {
  test(`visual ${c.name}`, async ({ page }) => {
    const fixture = JSON.parse(fs.readFileSync(c.fixturePath, "utf8"));
    const vp = SIZE_VIEWPORTS[c.size] ?? SIZE_VIEWPORTS.small;
    await page.setViewportSize(vp);
    await stubTauri(page, fixture, c.theme);
    await page.goto(widgetHtmlUrl(c.size, c.theme));
    await waitForRender(page);

    const raw = await dumpTree(page);
    const tree = normalizeTree(raw);
    expect(tree).not.toBeNull();
    assertNoLiteralNullText(tree!);

    if (c.name.includes("null-fields")) {
      const html = await page.content();
      expect(html).not.toMatch(/>\s*null\s*</);
    }

    const expectedGeom = geometryPath(c.fixture, c.size, "desktop");
    await page.addStyleTag({
      content: `#drag-handle,#close-btn{display:none!important}`,
    });

    // Geometry Level-1: still updatable via UPDATE_SNAPSHOTS (bulk ok for geometry).
    // Skip compare when recording pixels — glyph/text changes fail text-leaf asserts first.
    if (UPDATE_SNAPSHOTS) {
      fs.mkdirSync(path.dirname(expectedGeom), { recursive: true });
      fs.writeFileSync(expectedGeom, JSON.stringify(tree, null, 2) + "\n");
    } else if (!GOLDEN_RECORD && fs.existsSync(expectedGeom)) {
      const expected = JSON.parse(fs.readFileSync(expectedGeom, "utf8")) as GeometryNode;
      const errs = diffTrees(tree!, expected, 2);
      expect(errs, errs.join("\n")).toEqual([]);
    }

    const png = await awaitStable(page);
    const golden = goldenPath(c.name);
    const outs = outPaths(c.name);

    if (GOLDEN_RECORD) {
      fs.mkdirSync(path.dirname(golden), { recursive: true });
      fs.writeFileSync(golden, png);
      return;
    }

    if (!fs.existsSync(golden)) {
      test.info().annotations.push({
        type: "note",
        description: `no golden yet: ${golden} (GOLDEN_RECORD=1 CASE=${c.name})`,
      });
      return;
    }

    const expectedPng = fs.readFileSync(golden);
    const { frac, diff } = await pixelMismatch(expectedPng, png);
    if (frac > 0.02) {
      fs.mkdirSync(outs.dir, { recursive: true });
      fs.writeFileSync(outs.actual, png);
      if (diff) fs.writeFileSync(outs.diff, diff);
      expect(frac, `pixel mismatch ${c.name}: ${(frac * 100).toFixed(2)}% → ${outs.dir}`).toBeLessThanOrEqual(
        0.02,
      );
    }
  });
}

// Keep discovery smoke if cases empty (misconfigured checkout).
test("cases corpus non-empty", () => {
  expect(loadCases().length, "tests/cases/*.json missing").toBeGreaterThan(0);
});
