import type { ElNode, SkippedElement, WidgetConfig } from "./types";
import {
  invoke,
  listen,
  GROUP,
  SIZE,
  THEME,
  WIDGET_ID,
  root,
  clearTimers,
} from "./ctx";
import { renderWidget } from "./lib";

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
        instance: label,
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

function render(cfg: WidgetConfig | null | undefined, source?: string, nonceHint?: unknown) {
  if (!root) return;
  if (!cfg) {
    renderWidget(root, null, { size: SIZE, theme: THEME, chrome: true });
    return;
  }
  renderWidget(root, cfg, { size: SIZE, theme: THEME, chrome: true });
  reportReceipt(
    cfg,
    source || "pull",
    source === "push" ? "reload" : "timeline",
    nonceHint,
  );
}

function loadConfig() {
  if (!root) return;
  Promise.all([
    invoke("plugin:widgets|get_widget_config", {
      group: GROUP,
      widgetId: WIDGET_ID,
    }),
    invoke("plugin:widgets|get_items", {
      key: "__meta_nonce__",
      group: GROUP,
    }).catch(function () {
      return null;
    }),
  ])
    .then(function (res) {
      var cfg = res[0] as WidgetConfig | null;
      var nonce = res[1];
      if (cfg && nonce != null) cfg.__nonce = Number(nonce) || 0;
      render(cfg, "pull", nonce);
    })
    .catch(function (e) {
      clearTimers();
      if (!root) return;
      root.textContent = "";
      var err = document.createElement("div");
      err.className = "w-err";
      err.textContent = String(e);
      root.appendChild(err);
    });
}

function init() {
  if (!window.__TAURI_INTERNALS__) {
    setTimeout(init, 50);
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
      if (p.widgetId && p.widgetId !== WIDGET_ID) return;
      if (p.group && p.group !== GROUP) return;
      render(p.config, "push");
    } else {
      render(raw as WidgetConfig | null, "push");
    }
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
