// Records the docs demos from the real desktop renderer (widget.html) — the same code the
// app runs — with a stubbed Tauri host. Output: node_modules/.cache/demos/<test>/video.webm;
// scripts/sh/record-demos.sh converts each to docs/public/media/<name>.mp4 + .jpg poster.
// A drawn cursor shows hovers and clicks (headless video has no pointer).
import { test, type Browser, type Page } from "@playwright/test";
import { pathToFileURL } from "node:url";
import fs from "node:fs";
import path from "node:path";
import { ISLAND_DEMOS, launcherConfig } from "../../examples/tauri-plugin-widgets-example/src/islandDemos";
import type { ActivityContent, WidgetElement } from "../../guest-js";

const WIDGET_HTML = path.resolve(import.meta.dirname, "../../widget.html");

const OUT = path.resolve(import.meta.dirname, "../../node_modules/.cache/demos");

/** Page zoom: Playwright records CSS pixels (deviceScaleFactor does not reach the video), so
 *  the page is drawn at 2x CSS zoom in a 2x window for sharp text. */
const Z = 2;

/** Records `body` on a `w`x`h` (CSS px) scene to OUT/<name>.webm, at Z x resolution. */
async function record(browser: Browser, name: string, w: number, h: number, body: (page: Page) => Promise<void>) {
  const context = await browser.newContext({
    viewport: { width: w * Z, height: h * Z },
    recordVideo: { dir: OUT, size: { width: w * Z, height: h * Z } },
  });
  const page = await context.newPage();
  await body(page);
  await context.close();
  await page.video()!.saveAs(path.join(OUT, `${name}.webm`));
}

/** Stub host, as the Rust plugin answers the renderer: configs per widgetId, a key-value
 *  store (`get_items`), `set_widget_state` with the native operators, and named events the
 *  test can emit (`window.__emit`) like the host does. */
async function stubHost(page: Page, configs: Record<string, unknown>, items: Record<string, unknown> = {}) {
  await page.addInitScript(
    ([cfgs, initial]) => {
      const w = window as unknown as Record<string, any>;
      const cbs = new Map<number, (e: unknown) => void>();
      const byEvent = new Map<string, number[]>();
      let next = 1;
      w.__ITEMS__ = Object.fromEntries(Object.entries(initial).map(([k, v]) => [k, JSON.stringify(v)]));
      w.__emit = (event: string, payload: unknown) =>
        (byEvent.get(event) || []).forEach((id) => cbs.get(id)?.({ event, payload }));
      // Island: the last pushed view; `expanded` follows the page's own requests.
      w.__VIEW__ = null;
      w.__EXPANDED__ = false;
      w.__push = (view: Record<string, unknown> | null) => {
        w.__VIEW__ = view;
        w.__emit("activity-island", view && { ...view, expanded: w.__EXPANDED__ });
      };
      const q = new URLSearchParams(location.search);
      const widgetId = q.get("widgetId") || "";
      const apply = (old: unknown, v: unknown) =>
        v === "$toggle" ? !old : v === "$inc" ? Number(old || 0) + 1 : v === "$dec" ? Number(old || 0) - 1 : v;
      w.__TAURI_INTERNALS__ = {
        invoke: (cmd: string, args: Record<string, any> = {}) => {
          if (cmd === "plugin:event|listen") {
            byEvent.set(args.event, [...(byEvent.get(args.event) || []), args.handler]);
            return Promise.resolve(args.handler);
          }
          if (cmd.endsWith("activity_island")) {
            if (typeof args.expanded === "boolean") w.__EXPANDED__ = args.expanded;
            const v = w.__VIEW__;
            return Promise.resolve(v && { ...v, expanded: w.__EXPANDED__ });
          }
          if (cmd.includes("get_widget_config")) return Promise.resolve(cfgs[widgetId] ?? null);
          if (cmd.endsWith("get_items")) return Promise.resolve(w.__ITEMS__[args.key] ?? null);
          if (cmd.endsWith("set_widget_state")) {
            const key = `state:${args.widgetId}`;
            const st = JSON.parse(w.__ITEMS__[key] || "{}");
            const base = (cfgs[args.widgetId] as any)?.state || {};
            for (const [k, v] of Object.entries(args.patch || {})) {
              if (v === null) delete st[k];
              else st[k] = apply(k in st ? st[k] : base[k], v);
            }
            w.__ITEMS__[key] = JSON.stringify(st);
            setTimeout(() => w.__emit("widget-state-changed", { widgetId: args.widgetId, instance: null, state: st }), 0);
            return Promise.resolve(st);
          }
          if (cmd.endsWith("widget_action")) {
            w.__ACTIONS__ = [...(w.__ACTIONS__ || []), args];
            return Promise.resolve(null);
          }
          return Promise.resolve(null);
        },
        transformCallback: (cb: (e: unknown) => void) => {
          const id = next++;
          cbs.set(id, cb);
          return id;
        },
        metadata: { currentWindow: { label: widgetId } },
      };
    },
    [configs, items] as const,
  );
  // Desktop wallpaper behind transparent widget windows, and a drawn cursor.
  await page.addInitScript(() => {
    addEventListener("DOMContentLoaded", () => {
      const s = document.createElement("style");
      if (window !== window.top) {
        // Widget windows of a desk scene: drawn at 2x inside 2x iframes.
        s.textContent = "html { zoom: 2; background: transparent !important; }";
        document.head.appendChild(s);
        return;
      }
      // Island scenes load widget.html itself as the top page: zoom it; a desk page is not zoomed.
      const zoom = location.pathname.endsWith("widget.html") ? 2 : 1;
      (window as any).__Z__ = zoom;
      s.textContent = `
        html, body { min-height: 100%; height: 100%; }
        html { zoom: ${zoom}; background: radial-gradient(120% 140% at 50% 0%, #2b3a67 0%, #141a2e 55%, #0b0d17 100%) !important; }
        #demo-cursor { position: fixed; z-index: 99999; width: 18px; height: 18px; margin: -2px 0 0 -2px;
          pointer-events: none; transition: transform .08s; filter: drop-shadow(0 1px 2px #0008); }
        #demo-cursor.down { transform: scale(.82); }`;
      document.head.appendChild(s);
      const c = document.createElement("div");
      c.id = "demo-cursor";
      c.innerHTML = `<svg viewBox="0 0 18 18" width="18" height="18"><path d="M2 1l12 9-5 1 3 6-2 1-3-6-4 4z" fill="#fff" stroke="#000" stroke-width="1"/></svg>`;
      c.style.left = "-40px";
      document.body.appendChild(c);
      // Positioned by the test (moveTo): mouse events over iframes never reach this document.
    });
  });
}

const url = (widgetId: string, size = "medium") =>
  `${pathToFileURL(WIDGET_HTML).href}?${new URLSearchParams({ group: "g", widgetId, size })}`;

/** Rust `island::regions` for one activity (given regions win, defaults from icon / progress). */
function regions(c: ActivityContent) {
  const tint = c.keylineTint ?? c.tint ?? "#34c759";
  const icon: WidgetElement = { type: "image", systemName: c.icon ?? "circle.fill", size: 15, color: tint } as WidgetElement;
  const pct = c.progress != null ? `${Math.round(c.progress * 100)}%` : "";
  return {
    compactLeading: c.compactLeading ?? icon,
    compactTrailing: c.compactTrailing ?? { type: "text", content: pct, fontSize: 13, fontWeight: "semibold", color: tint },
    minimal: c.minimal ?? icon,
    leading: c.islandLeading ?? null,
    trailing: c.islandTrailing ?? null,
    center: c.islandCenter ?? null,
    bottom: c.layout ?? { type: "text", content: c.title ?? "", fontSize: 14, padding: 12 },
  };
}

/** A notched-Mac island view (Rust `island::View`) showing `c`. */
const view = (c: ActivityContent, expandedSize: [number, number] = [400, 164]) => ({
  expanded: false,
  flush: true,
  gap: 185,
  band: 32,
  compact: [329, 32],
  expandedSize,
  minimal: [],
  activities: [{ id: c.id, group: "g", widgetId: `activity.${c.id}`, url: null, keyline: c.keylineTint ?? c.tint ?? null, regions: regions(c) }],
});

const lastPos = new WeakMap<Page, [number, number]>();

/** Moves the mouse and the drawn cursor smoothly to scene point x, y (CSS px, unzoomed). */
async function moveTo(page: Page, x: number, y: number, steps = 18) {
  const [x0, y0] = lastPos.get(page) ?? [x, y];
  for (let i = 1; i <= steps; i++) {
    const k = steps === 1 ? 1 : 0.5 - 0.5 * Math.cos((i / steps) * Math.PI);
    const px = (x0 + (x - x0) * k) * Z;
    const py = (y0 + (y - y0) * k) * Z;
    await page.mouse.move(px, py);
    await page.evaluate(([px, py]) => {
      const c = document.getElementById("demo-cursor");
      const z = (window as any).__Z__ || 1;
      if (c) Object.assign(c.style, { left: `${px / z}px`, top: `${py / z}px` });
    }, [px, py]);
  }
  lastPos.set(page, [x, y]);
}

/** Press and release at the current position, shown on the drawn cursor. */
async function click(page: Page) {
  await page.evaluate(() => document.getElementById("demo-cursor")?.classList.add("down"));
  await page.mouse.down();
  await page.waitForTimeout(90);
  await page.mouse.up();
  await page.evaluate(() => document.getElementById("demo-cursor")?.classList.remove("down"));
}

/** Centre of `selector` in scene CSS px (boundingBox is in window px). */
async function centreOf(page: Page, selector: string) {
  const b = (await page.locator(selector).first().boundingBox())!;
  return [(b.x + b.width / 2) / Z, (b.y + b.height / 2) / Z] as const;
}

/** Plays a demo's frames at its own tick rate for `ms`. */
async function play(page: Page, demo: (typeof ISLAND_DEMOS)[number], ms: number, t0: number) {
  const end = Date.now() + ms;
  while (Date.now() < end) {
    await page.evaluate((v) => (window as any).__push(v), view(demo.frame(Date.now() - t0)));
    await page.waitForTimeout(Math.max(demo.tickMs, 40));
  }
}

test("island-states", async ({ browser }) => {
  await record(browser, "island-states", 560, 230, async (page) => {
    await stubHost(page, {});
    const build = ISLAND_DEMOS.find((d) => d.name === "Download")!;
    await page.goto(url("activity.__island__"));
    const t0 = Date.now();
    await moveTo(page, 470, 210, 1);
    await play(page, build, 1600, t0);
    const [x, y] = await centreOf(page, ".isl");
    await moveTo(page, x, y);
    await play(page, build, 3200, t0);
    await moveTo(page, 470, 215);
    await page.dispatchEvent("html", "mouseleave");
    await play(page, build, 1600, t0);
  });
});

test("island-demos", async ({ browser }) => {
  await record(browser, "island-demos", 560, 250, async (page) => {
    await stubHost(page, {});
    await page.goto(url("activity.__island__"));
    await moveTo(page, 480, 230, 1);
    for (const name of ["Download", "Monitor", "Eyes", "Music", "Timer"]) {
      const demo = ISLAND_DEMOS.find((d) => d.name === name)!;
      const t0 = Date.now();
      await play(page, demo, 1300, t0);
      const [x, y] = await centreOf(page, ".isl");
      await moveTo(page, x, y, 12);
      await play(page, demo, 2200, t0);
      await moveTo(page, 480, 235, 12);
      await page.dispatchEvent("html", "mouseleave");
      await play(page, demo, 700, t0);
    }
  });
});

test("island-launcher", async ({ browser }) => {
  await record(browser, "island-launcher", 560, 340, async (page) => {
    await stubHost(page, { launcher: launcherConfig() });
    const demo = ISLAND_DEMOS.find((d) => d.name === "Launcher")!;
    await page.goto(url("activity.__island__"));
    await moveTo(page, 470, 320, 1);
    await play(page, demo, 1200, Date.now());
    const [x, y] = await centreOf(page, ".isl");
    await moveTo(page, x, y);
    await play(page, demo, 900, Date.now());
    // The host opens the launcher window under the island on the button's action.
    const [bx, by] = await centreOf(page, "text=Open apps");
    await moveTo(page, bx, by, 10);
    await click(page);
    await page.evaluate((src) => {
      const f = document.createElement("iframe");
      f.src = src;
      f.style.cssText = "position:fixed;left:50%;top:184px;width:338px;height:150px;margin-left:-169px;border:0;" +
        "background:transparent;opacity:0;transform:translateY(-8px);transition:opacity .25s,transform .25s;";
      f.allowTransparency = true as never;
      document.body.appendChild(f);
      f.onload = () => requestAnimationFrame(() => { f.style.opacity = "1"; f.style.transform = "none"; });
    }, url("launcher"));
    await play(page, demo, 1500, Date.now());
    await moveTo(page, 280 - 60, 250, 14);
    await page.waitForTimeout(1500);
  });
});

// ── Widget scenes: widget windows (widget.html iframes) on the wallpaper ────────────────

type Win = { widgetId: string; size: "small" | "medium"; x: number; y: number; instance?: string };
const SIZE = { small: [170, 170], medium: [360, 170] } as const;

/** Opens a desktop scene with one iframe per widget window; returns their frames. */
async function desk(page: Page, wins: Win[]) {
  const frames = wins
    .map((w) => {
      const [fw, fh] = SIZE[w.size];
      const q = new URLSearchParams({ group: "g", widgetId: w.widgetId, size: w.size, ...(w.instance ? { instance: w.instance } : {}) });
      return `<iframe data-id="${w.widgetId}${w.instance ? "#" + w.instance : ""}" src="${pathToFileURL(WIDGET_HTML).href}?${q}"
        style="position:fixed;left:${w.x * Z}px;top:${w.y * Z}px;width:${fw * Z}px;height:${fh * Z}px;border:0;background:transparent;
        filter:drop-shadow(0 10px 24px #0008)" allowtransparency="true"></iframe>`;
    })
    .join("");
  // A file:// page: Chromium does not load file:// iframes into about:blank (setContent).
  const file = path.join(OUT, `desk-${wins.map((w) => w.widgetId + (w.instance || "")).join("-")}.html`);
  fs.mkdirSync(OUT, { recursive: true });
  fs.writeFileSync(file, `<!doctype html><html><body style="margin:0">${frames}</body></html>`);
  await page.goto(pathToFileURL(file).href);
  await page.waitForTimeout(600);
  return wins.map((w) => page.frames().find((f) => f.url().includes(`widgetId=${w.widgetId}`) && (!w.instance || f.url().includes(`instance=${w.instance}`)))!);
}

/** Clicks the element `selector` inside a widget frame with the drawn cursor. */
async function tap(page: Page, frame: import("@playwright/test").Frame, selector: string) {
  const b = (await frame.locator(selector).first().boundingBox())!;
  await moveTo(page, (b.x + b.width / 2) / Z, (b.y + b.height / 2) / Z, 14);
  await page.waitForTimeout(150);
  await click(page);
  await page.waitForTimeout(650);
}

const toggle = (key: string, label: string) => ({
  type: "toggle", label, isOn: false, tint: "#22c55e", color: "#e2e8f0",
  bind: { isOn: `state.${key}` }, set: { [key]: "$toggle" },
});

test("interactive-state", async ({ browser }) => {
  const tasks = {
    state: { milk: false, ship: true, call: false, cups: 2 },
    medium: {
      type: "hstack", padding: 16, spacing: 18, background: "#111827",
      children: [
        { type: "vstack", spacing: 8, alignment: "leading", flex: 1, children: [
          { type: "text", content: "Today", fontSize: 16, fontWeight: "bold", color: "#f8fafc" },
          toggle("milk", "Buy milk"), toggle("ship", "Ship 0.8"), toggle("call", "Call mom"),
        ] },
        { type: "vstack", spacing: 4, alignment: "center", children: [
          { type: "text", content: "💧", fontSize: 24 },
          { type: "text", content: "{{state.cups}}", fontSize: 32, fontWeight: "bold", color: "#38bdf8", transition: "numeric" },
          { type: "text", content: "cups", fontSize: 11, color: "#94a3b8" },
          { type: "button", label: "+1", set: { cups: "$inc" }, tint: "#0ea5e9", backgroundColor: "#0c4a6e", color: "#e0f2fe" },
        ] },
      ],
    },
  };
  await record(browser, "interactive-state", 560, 260, async (page) => {
    await stubHost(page, { tasks });
    const [f] = await desk(page, [{ widgetId: "tasks", size: "medium", x: 100, y: 45 }]);
    await moveTo(page, 520, 240, 1);
    await page.waitForTimeout(700);
    await tap(page, f, "text=Buy milk");
    await tap(page, f, "text=Call mom");
    await tap(page, f, "text=+1");
    await tap(page, f, "text=+1");
    await tap(page, f, "text=Ship 0.8");
    await moveTo(page, 520, 240, 16);
    await page.waitForTimeout(900);
  });
});

test("timelines", async ({ browser }) => {
  const now = Date.now() + 900;
  const slot = (emoji: string, greet: string, time: string, bg: string) => ({
    type: "hstack", padding: 18, spacing: 16, background: bg,
    children: [
      { type: "text", content: emoji, fontSize: 46 },
      { type: "vstack", spacing: 4, alignment: "leading", flex: 1, children: [
        { type: "text", content: greet, fontSize: 20, fontWeight: "bold", color: "#fff" },
        { type: "text", content: time, fontSize: 13, color: "#ffffffb3" },
      ] },
    ],
  });
  const day = {
    transition: "slide",
    medium: slot("🌅", "Good morning", "07:00 · entries[0]", "#7c2d12"),
    entries: [
      { at: now + 1800, medium: slot("☀️", "Lunch break", "13:00 · entries[1]", "#a16207") },
      { at: now + 3600, medium: slot("🌇", "Good evening", "19:00 · entries[2]", "#7e22ce") },
      { at: now + 5400, medium: slot("🌙", "Good night", "23:00 · entries[3]", "#1e1b4b") },
    ],
  };
  await record(browser, "timelines", 560, 260, async (page) => {
    await stubHost(page, { day });
    await desk(page, [{ widgetId: "day", size: "medium", x: 100, y: 45 }]);
    await page.waitForTimeout(8000);
  });
});

test("live-data", async ({ browser }) => {
  const dash = {
    medium: {
      type: "hstack", padding: 16, spacing: 16, background: "#0f172a",
      children: [
        { type: "vstack", spacing: 6, alignment: "leading", flex: 1, children: [
          { type: "text", content: "CPU", fontSize: 12, color: "#94a3b8" },
          { type: "text", content: "{{data.cpu}}%", fontSize: 30, fontWeight: "bold", color: "#22d3ee", transition: "numeric" },
          { type: "progress", value: 0, tint: "#22d3ee", bind: { value: "data.cpuFrac" } },
        ] },
        { type: "vstack", spacing: 6, alignment: "leading", flex: 1, children: [
          { type: "text", content: "Downloads", fontSize: 12, color: "#94a3b8" },
          { type: "text", content: "{{data.mb}} MB/s", fontSize: 30, fontWeight: "bold", color: "#a78bfa", transition: "numeric" },
          { type: "progress", value: 0, tint: "#a78bfa", bind: { value: "data.mbFrac" } },
        ] },
      ],
    },
  };
  await record(browser, "live-data", 560, 260, async (page) => {
    await stubHost(page, { dash }, { "data:dash": { cpu: 18, cpuFrac: 0.18, mb: 42, mbFrac: 0.42 } });
    const [f] = await desk(page, [{ widgetId: "dash", size: "medium", x: 100, y: 45 }]);
    for (let i = 0; i < 14; i++) {
      const cpu = Math.round(30 + 25 * Math.sin(i / 1.6) + 10 * Math.sin(i * 1.3));
      const mb = Math.round(60 + 30 * Math.sin(i / 2.3 + 1));
      await f.evaluate((data) => (window as any).__emit("widget-data-push", { widgetId: "dash", instance: null, data }),
        { cpu, cpuFrac: cpu / 100, mb, mbFrac: mb / 100 });
      await page.waitForTimeout(650);
    }
  });
});

test("instances", async ({ browser }) => {
  const weather = {
    small: {
      type: "vstack", padding: 14, spacing: 6, alignment: "leading", background: "#1e3a5f",
      children: [
        { type: "text", content: "{{data.city}}", fontSize: 14, fontWeight: "semibold", color: "#e0f2fe" },
        { type: "text", content: "{{data.temp}}°", fontSize: 44, fontWeight: "bold", color: "#fff", transition: "numeric" },
        { type: "text", content: "{{data.sky}}", fontSize: 12, color: "#bae6fd" },
      ],
    },
  };
  await record(browser, "instances", 560, 260, async (page) => {
    await stubHost(page, { weather }, {
      "data:weather#london": { city: "London", temp: 14, sky: "☁️ Cloudy" },
      "data:weather#newyork": { city: "New York", temp: 22, sky: "☀️ Sunny" },
    });
    const [lon, ny] = await desk(page, [
      { widgetId: "weather", instance: "london", size: "small", x: 90, y: 45 },
      { widgetId: "weather", instance: "newyork", size: "small", x: 300, y: 45 },
    ]);
    for (let i = 0; i < 6; i++) {
      await page.waitForTimeout(900);
      await lon.evaluate((d) => (window as any).__emit("widget-data-push", { widgetId: "weather", instance: "london", data: d }),
        { city: "London", temp: 14 + (i % 3), sky: i % 2 ? "🌧 Rain" : "☁️ Cloudy" });
      await ny.evaluate((d) => (window as any).__emit("widget-data-push", { widgetId: "weather", instance: "newyork", data: d }),
        { city: "New York", temp: 22 + ((i + 1) % 3), sky: "☀️ Sunny" });
    }
    await page.waitForTimeout(900);
  });
});
