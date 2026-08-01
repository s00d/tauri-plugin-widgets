import fs from "node:fs";
import path from "node:path";
import { type Page } from "@playwright/test";
import {
  EXPECTED_PIXELS,
  FIXTURES_ROOT,
  SIZE_VIEWPORTS,
  UPDATE_SNAPSHOTS,
  assertNoLiteralNullText,
  normalizeTree,
  widgetHtmlUrl,
  type GeometryNode,
} from "./helpers.js";

export type EventScenario = {
  id: string;
  size: string;
  action: string;
  clickText: string;
  assertBeforeTexts: string[];
  assertAfterTexts: string[];
  assertGoneAfter?: string[];
  before: Record<string, unknown>;
  after: Record<string, unknown>;
};

export function listEventScenarios(): EventScenario[] {
  const dir = path.join(FIXTURES_ROOT, "events");
  if (!fs.existsSync(dir)) return [];
  return fs
    .readdirSync(dir)
    .filter((f) => f.endsWith(".json"))
    .sort()
    .map((f) => JSON.parse(fs.readFileSync(path.join(dir, f), "utf8")) as EventScenario);
}

export function eventPixelPath(id: string, phase: "before" | "after"): string {
  return path.join(EXPECTED_PIXELS, "desktop", "events", `${id}.${phase}.png`);
}

/**
 * Stub Tauri with live config + widget_action → widget-config-push.
 * Handlers map: action string → next WidgetConfig (or function).
 */
export async function stubTauriInteractive(
  page: Page,
  initialConfig: unknown,
  resolveAfterAction: (action: string, payload: string | null) => unknown | null,
) {
  await page.addInitScript(
    ({ cfg, _marker }) => {
      const win = window as unknown as {
        __FIXTURE__: unknown;
        __ACTIONS__: Array<{ action: string; payload: string | null }>;
        __PUSH_CONFIG__?: (next: unknown) => void;
        __TAURI_INTERNALS__: unknown;
      };
      win.__FIXTURE__ = cfg;
      win.__ACTIONS__ = [];
      const channelHandlers: Record<number, (e: unknown) => void> = {};
      let nextId = 1;
      const eventListeners: Array<(e: unknown) => void> = [];

      win.__PUSH_CONFIG__ = (next: unknown) => {
        win.__FIXTURE__ = next;
        for (const h of eventListeners) {
          h({
            payload: {
              config: next,
              widgetId: "w",
              group: "g",
            },
          });
        }
      };

      win.__TAURI_INTERNALS__ = {
        invoke: (cmd: string, args?: Record<string, unknown>) => {
          const c = String(cmd);
          if (c.includes("get_widget_config")) {
            return Promise.resolve(win.__FIXTURE__);
          }
          if (c.includes("listen")) {
            // Channel id registered via transformCallback just before invoke(listen)
            return Promise.resolve(1);
          }
          if (c.includes("widget_action")) {
            const action = String(args?.action ?? "");
            const payload = (args?.payload as string | null) ?? null;
            win.__ACTIONS__.push({ action, payload });
            // Host resolves next config via exposed bridge set by Playwright evaluate
            const bridge = (window as unknown as { __RESOLVE_ACTION__?: (a: string, p: string | null) => unknown }).__RESOLVE_ACTION__;
            if (bridge) {
              const next = bridge(action, payload);
              if (next != null && win.__PUSH_CONFIG__) win.__PUSH_CONFIG__(next);
            }
            return Promise.resolve(null);
          }
          return Promise.resolve(null);
        },
        transformCallback: (f: (e: unknown) => void) => {
          const id = nextId++;
          channelHandlers[id] = f;
          eventListeners.push(f);
          return id;
        },
        metadata: { currentWindow: { label: "test" } },
      };
      void _marker;
    },
    { cfg: initialConfig, _marker: 1 },
  );

  // Expose action resolver into the page after navigation starts — bind before goto via init
  await page.addInitScript((fnBody) => {
    // eslint-disable-next-line no-new-func
    (window as unknown as { __RESOLVE_ACTION__: (a: string, p: string | null) => unknown }).__RESOLVE_ACTION__ =
      new Function("action", "payload", fnBody) as (a: string, p: string | null) => unknown;
  }, `return (${serializeResolver(resolveAfterAction)})(action, payload);`);
}

function serializeResolver(
  fn: (action: string, payload: string | null) => unknown | null,
): string {
  // We can't serialize closures; scenarios pass map via page.evaluate after load instead.
  void fn;
  return `function(){ return null; }`;
}

/** Install action→config map after page is created (before interactions). */
export async function installActionMap(
  page: Page,
  map: Record<string, unknown>,
) {
  await page.evaluate((m) => {
    (window as unknown as { __RESOLVE_ACTION__: (a: string, _p: string | null) => unknown }).__RESOLVE_ACTION__ =
      (action) => (Object.prototype.hasOwnProperty.call(m, action) ? m[action] : null);
  }, map);
}

export async function stubTauriSimple(page: Page, fixture: unknown) {
  await page.addInitScript((cfg) => {
    const win = window as unknown as {
      __FIXTURE__: unknown;
      __ACTIONS__: Array<{ action: string; payload: string | null }>;
      __PUSH_CONFIG__?: (next: unknown) => void;
      __TAURI_INTERNALS__: unknown;
    };
    win.__FIXTURE__ = cfg;
    win.__ACTIONS__ = [];
    const eventListeners: Array<(e: unknown) => void> = [];

    win.__PUSH_CONFIG__ = (next: unknown) => {
      win.__FIXTURE__ = next;
      for (const h of eventListeners) {
        h({ payload: { config: next, widgetId: "w", group: "g" } });
      }
    };

    win.__TAURI_INTERNALS__ = {
      invoke: (cmd: string, args?: Record<string, unknown>) => {
        const c = String(cmd);
        if (c.includes("get_widget_config")) return Promise.resolve(win.__FIXTURE__);
        if (c.includes("listen")) return Promise.resolve(1);
        if (c.includes("widget_action")) {
          const action = String(args?.action ?? "");
          const payload = (args?.payload as string | null) ?? null;
          win.__ACTIONS__.push({ action, payload });
          const bridge = (window as unknown as { __RESOLVE_ACTION__?: (a: string, p: string | null) => unknown }).__RESOLVE_ACTION__;
          if (bridge) {
            const next = bridge(action, payload);
            if (next != null && win.__PUSH_CONFIG__) win.__PUSH_CONFIG__(next);
          }
          return Promise.resolve(null);
        }
        return Promise.resolve(null);
      },
      transformCallback: (f: (e: unknown) => void) => {
        eventListeners.push(f);
        return eventListeners.length;
      },
      metadata: { currentWindow: { label: "test" } },
    };
  }, fixture);
}

export async function dumpTree(page: Page): Promise<GeometryNode> {
  return page.evaluate(() => {
    const isChrome = (el: Element) => el.id === "drag-handle" || el.id === "close-btn";
    const walk = (el: Element): GeometryNode => {
      const r = el.getBoundingClientRect();
      const cs = getComputedStyle(el);
      const text =
        el.children.length === 0 ? (el.textContent || "").trim() || null : null;
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

export async function waitForRender(page: Page) {
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

export async function hideChrome(page: Page) {
  await page.addStyleTag({
    content: `#drag-handle,#close-btn{display:none!important}`,
  });
}

export function collectTexts(node: GeometryNode, out: string[] = []): string[] {
  if (node.text) out.push(node.text);
  for (const c of node.children ?? []) collectTexts(c, out);
  return out;
}

export { SIZE_VIEWPORTS, UPDATE_SNAPSHOTS, assertNoLiteralNullText, normalizeTree, widgetHtmlUrl };
