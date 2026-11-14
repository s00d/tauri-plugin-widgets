import type { ElNode, SkippedElement, WidgetConfig } from "./types";
import { invoke, listen, clearTimers } from "./ctx";
import { renderWidget } from "./lib";
import { mergeData, sourceRequest, MAX_SOURCE_BYTES, type ResolveContext } from "./resolve";
import { mountIsland, type IslandView } from "./island";

const params = new URLSearchParams(window.location.search);
// "" = the app's default group (plugins.widgets.appGroup).
const GROUP = params.get("group") || "";
const SIZE = params.get("size") || "";
const WIDGET_ID = params.get("widgetId") || "default";
const THEME = (params.get("theme") || "").toLowerCase();
const INSTANCE = params.get("instance") || "";
const SCOPED_ID = INSTANCE ? WIDGET_ID + "#" + INSTANCE : WIDGET_ID;

/** Last raw config and data layers; any of them changing re-renders. */
let lastConfig: WidgetConfig | null = null;
let hostData: unknown = undefined;
let fetchedData: unknown = undefined;
let sourceTimer: ReturnType<typeof setTimeout> | null = null;
let sourceKey = "";
/** Whether the shown config / data are the placement's own (not widget-wide). */
let lastConfigScoped = false;
let hostDataScoped = false;
/** Stored local state: `state:{widgetId}`, then `state:{widgetId}#{instance}`. */
let stateLayers: unknown[] = [];

function getRoot(): HTMLElement | null {
  return document.getElementById("root");
}

function collectTrace(node: unknown, rendered: string[], skipped: SkippedElement[]) {
  if (!node || typeof node !== "object") return;
  const n = node as ElNode;
  var t = n.type;
  if (typeof t === "string" && t) {
    rendered.push(t);
    if (t === "canvas" && !(window as unknown as { Path2D?: unknown }).Path2D) {
      skipped.push({ type: t, reason: "unsupported" });
    }
  }
  var kids = n.children;
  if (Array.isArray(kids))
    kids.forEach(function (c: unknown) {
      collectTrace(c, rendered, skipped);
    });
  if (Array.isArray(n.items))
    n.items.forEach(function (it: ElNode) {
      if (it && it.children) collectTrace(it, rendered, skipped);
    });
}

function reportReceipt(cfg: WidgetConfig | null | undefined, source: string, trigger?: string, nonceHint?: unknown) {
  try {
    var label =
      (window.__TAURI_INTERNALS__ &&
        window.__TAURI_INTERNALS__.metadata &&
        window.__TAURI_INTERNALS__.metadata.currentWindow &&
        window.__TAURI_INTERNALS__.metadata.currentWindow.label) ||
      "desktop";
    var rendered: string[] = [],
      skipped: SkippedElement[] = [];
    var keyed = cfg as (WidgetConfig & Record<string, ElNode | undefined>) | null | undefined;
    var branch = SIZE
      ? keyed && (keyed[SIZE] || keyed.small || keyed.medium || keyed.large)
      : keyed && (keyed.small || keyed.medium || keyed.large);
    collectTrace(branch, rendered, skipped);
    var nonce = 0;
    if (nonceHint != null) nonce = Number(nonceHint) || 0;
    else if (cfg && cfg.__nonce != null) nonce = Number(cfg.__nonce) || 0;
    var trig = trigger || (source === "push" ? "reload" : "timeline");
    invoke("plugin:widgets|report_receipt", {
      receipt: {
        widgetId: WIDGET_ID,
        group: GROUP,
        // Same id the host lists for this window (instance, or the label by default).
        instance: INSTANCE || label,
        nonce: nonce,
        size: SIZE || "small",
        theme: document.documentElement.getAttribute("data-theme") || undefined,
        schema: 1,
        source: source || "pull",
        trigger: trig,
        rendered: rendered,
        skipped: skipped,
        ts: Date.now(),
      },
    }).catch(function () {});
  } catch (_) {}
}

function context(cfg: WidgetConfig | null | undefined): Partial<ResolveContext> {
  return {
    locale: navigator.language || "",
    instance: INSTANCE || null,
    data: mergeData([(cfg as { data?: unknown } | null | undefined)?.data, hostData, fetchedData]),
    state: mergeData([cfg?.state, ...stateLayers]),
  };
}

function loadState(): Promise<void> {
  const keys = INSTANCE ? ["state:" + WIDGET_ID, "state:" + SCOPED_ID] : ["state:" + WIDGET_ID];
  return Promise.all(keys.map(getItem)).then(function (vals) {
    stateLayers = vals.map(parseJson);
  });
}

/** `set` tap: the host writes the placement's state (same operators as the native
 *  renderers) and emits `widget-state-changed`, which redraws every window showing it.
 *  No `skipReload`: a macOS WidgetKit widget with the same id reloads too. */
function writeState(patch: Record<string, unknown>): Promise<unknown> {
  return invoke("plugin:widgets|set_widget_state", {
    widgetId: WIDGET_ID,
    instance: INSTANCE || null,
    patch,
    group: GROUP || null,
  });
}

function render(cfg: WidgetConfig | null | undefined, source?: string, nonceHint?: unknown) {
  const root = getRoot();
  if (!root) return;
  lastConfig = cfg ?? null;
  scheduleSource(cfg);
  renderWidget(root, cfg ?? null, {
    size: SIZE,
    theme: THEME,
    group: GROUP,
    widgetId: WIDGET_ID,
    chrome: true,
    context: context(cfg),
    onSet: writeState,
  });
  if (!cfg) return;
  reportReceipt(
    cfg,
    source || "pull",
    source === "push" ? "reload" : "timeline",
    nonceHint,
  );
}

function getItem(key: string): Promise<string | null> {
  return (invoke("plugin:widgets|get_items", { key, group: GROUP || null }) as Promise<string | null>).catch(
    function () {
      return null;
    },
  );
}

function parseJson(raw: string | null): unknown {
  if (!raw) return undefined;
  try {
    return JSON.parse(raw);
  } catch (_) {
    return undefined;
  }
}

/** The placement's own value first, then the widget's (`store::lookup_keys`). */
function firstItem(prefix: string): Promise<{ value: unknown; scoped: boolean }> {
  const keys = INSTANCE ? [prefix + SCOPED_ID, prefix + WIDGET_ID] : [prefix + WIDGET_ID];
  return Promise.all(keys.map(getItem)).then(function (vals) {
    for (let i = 0; i < vals.length; i++) {
      const parsed = parseJson(vals[i]);
      if (parsed !== undefined) return { value: parsed, scoped: !!INSTANCE && i === 0 };
    }
    return { value: undefined, scoped: false };
  });
}

/** Fetch `source` now and then every `refreshMinutes`; the response is cached in the
 *  shared store (`fetched:{id}`) so other windows and the next launch start from it. */
function scheduleSource(cfg: WidgetConfig | null | undefined) {
  // Like the native renderers: only {{locale}} / {{instance}} in the request (no data, so a
  // response can never change its own request and refetch in a loop).
  const req = cfg
    ? sourceRequest(cfg, { nowMs: Date.now(), locale: navigator.language || "", instance: INSTANCE || null })
    : null;
  const key = req ? JSON.stringify(req) : "";
  if (key === sourceKey) return;
  sourceKey = key;
  if (sourceTimer) clearTimeout(sourceTimer);
  sourceTimer = null;
  if (!req) return;
  const run = function () {
    fetch(req.url, { method: req.method, headers: req.headers, body: req.body ?? undefined })
      .then(function (r) {
        if (!r.ok) throw new Error("HTTP " + r.status);
        return r.text();
      })
      .then(function (text) {
        if (sourceKey !== key) return; // source changed while this request was in flight
        if (new TextEncoder().encode(text).length > MAX_SOURCE_BYTES) throw new Error("source response too large");
        const json = JSON.parse(text);
        fetchedData = json;
        invoke("plugin:widgets|set_items", {
          key: "fetched:" + SCOPED_ID,
          value: text,
          group: GROUP || null,
        }).catch(function () {});
        render(lastConfig, "pull");
      })
      .catch(function (e) {
        console.warn("widget source fetch failed:", e);
      })
      .finally(function () {
        if (sourceKey === key) sourceTimer = setTimeout(run, req.refreshMinutes * 60000);
      });
  };
  run();
}

function loadConfig() {
  if (!getRoot()) return;
  const readConfig = function (instance: string | null) {
    return invoke("plugin:widgets|get_widget_config", {
      group: GROUP || null,
      widgetId: WIDGET_ID,
      instance,
    });
  };
  Promise.all([
    (INSTANCE ? readConfig(INSTANCE) : Promise.resolve(null)).then(function (own) {
      lastConfigScoped = !!own;
      return own || readConfig(null);
    }),
    invoke("plugin:widgets|get_items", {
      key: "__meta_nonce__",
      group: GROUP || null,
    }).catch(function () {
      return null;
    }),
  ])
    .then(function (res) {
      return Promise.all([firstItem("data:"), firstItem("fetched:"), loadState()]).then(function (layers) {
        hostData = layers[0].value;
        hostDataScoped = layers[0].scoped;
        if (fetchedData === undefined) fetchedData = layers[1].value;
        return res;
      });
    })
    .then(function (res) {
      var cfg = res[0] as WidgetConfig | null;
      var nonce = res[1];
      if (cfg && nonce != null) cfg.__nonce = Number(nonce) || 0;
      render(cfg, "pull", nonce);
    })
    .catch(function (e) {
      clearTimers();
      const root = getRoot();
      if (!root) return;
      root.textContent = "";
      var err = document.createElement("div");
      err.className = "w-err";
      err.textContent = String(e);
      root.appendChild(err);
    });
}

/** Live-activity island window (Rust `island::WIDGET_ID`). */
function initIsland(root: HTMLElement) {
  const ask = (args: Record<string, unknown>) =>
    (invoke("plugin:widgets|activity_island", args) as Promise<IslandView | null>).catch(function (e) {
      console.warn("activity_island:", e);
      return null;
    });
  const update = mountIsland(root, {
    setExpanded: (expanded) => ask({ expanded }),
    focus: (id) => {
      ask({ focus: id }).then(update);
    },
  });
  listen("activity-island", function (ev) {
    update(ev.payload as IslandView | null);
  });
  ask({}).then(update);
}

function init() {
  if (!window.__TAURI_INTERNALS__) {
    setTimeout(init, 50);
    return;
  }
  const root = getRoot();
  if (WIDGET_ID === "activity.__island__" && root) {
    initIsland(root);
    return;
  }
  listen("widget-config-push", function (ev) {
    type ConfigPush = {
      config?: WidgetConfig;
      widgetId?: string;
      group?: string;
    };
    const raw = ev.payload;
    if (raw && typeof raw === "object" && "config" in raw) {
      const p = raw as ConfigPush;
      if (!p.config) return;
      // An instance window takes its own config, or the widget-wide one while it has none.
      if (p.widgetId && p.widgetId !== WIDGET_ID && p.widgetId !== SCOPED_ID) return;
      if (INSTANCE && p.widgetId === WIDGET_ID && lastConfigScoped) return;
      if (GROUP && p.group && p.group !== GROUP) return;
      lastConfigScoped = p.widgetId === SCOPED_ID && !!INSTANCE;
      render(p.config, "push");
    } else {
      render(raw as WidgetConfig | null, "push");
    }
  });
  listen("widget-data-push", function (ev) {
    const p = ev.payload as { widgetId?: string; instance?: string | null; group?: string; data?: unknown };
    if (!p || p.widgetId !== WIDGET_ID) return;
    if (GROUP && p.group && p.group !== GROUP) return;
    if (p.instance && p.instance !== INSTANCE) return;
    // Widget-wide data does not replace a placement's own.
    if (!p.instance && INSTANCE && hostDataScoped) return;
    if (p.instance && p.data === null) {
      // Own data cleared: fall back to the widget-wide data in the store.
      hostDataScoped = false;
      firstItem("data:").then(function (d) {
        hostData = d.value;
        hostDataScoped = d.scoped;
        render(lastConfig, "push");
      });
      return;
    }
    hostDataScoped = !!p.instance;
    hostData = p.data === null ? undefined : p.data;
    render(lastConfig, "push");
  });
  listen("widget-state-changed", function (ev) {
    const p = ev.payload as { widgetId?: string; instance?: string | null; group?: string };
    if (!p || p.widgetId !== WIDGET_ID) return;
    if (GROUP && p.group && p.group !== GROUP) return;
    if (p.instance && p.instance !== INSTANCE) return;
    loadState().then(function () {
      render(lastConfig, "push");
    });
  });
  listen("widget-update", function () {
    loadConfig();
  });
  listen("widget-reload", function () {
    loadConfig();
  });
  loadConfig();
}

if (document.readyState === "complete" || document.readyState === "interactive")
  init();
else document.addEventListener("DOMContentLoaded", init);
