import type { ElNode, SkippedElement, WidgetConfig } from "./types";
import {
  invoke,
  listen,
  GROUP,
  SIZE,
  WIDGET_ID,
  root,
  clearTimers,
} from "./ctx";
import { isDark } from "./style";
import { renderEl } from "./render/element";

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
    var branch = SIZE
      ? cfg && ((cfg[SIZE] as ElNode | undefined) || cfg.small || cfg.medium || cfg.large)
      : cfg && (cfg.small || cfg.medium || cfg.large);
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
  clearTimers();
  if (!cfg) {
    root.innerHTML =
      '<div class="w-empty"><div><div style="font-size:28px;margin-bottom:4px">&#x1F4CC;</div><div style="font-size:14px;opacity:.7">No widget config</div></div></div>';
    return;
  }
  var data;
  if (SIZE) data = (cfg[SIZE] as ElNode | undefined) || cfg.small || cfg.medium || cfg.large;
  else data = cfg.small || cfg.medium || cfg.large;
  if (!data) {
    root.innerHTML =
      '<div class="w-empty"><div><div style="font-size:28px;margin-bottom:4px">&#x1F4CC;</div><div style="font-size:14px;opacity:.7">No config for size &quot;' +
      SIZE +
      '&quot;</div></div></div>';
    return;
  }

  root.innerHTML = "";
  // Readable defaults when fixtures leave color/background null (avoids black-on-black
  // and WebKit leftover Loading purple under transparent surfaces).
  var ink = isDark() ? "#e5e7eb" : "#111827";
  var surface = isDark() ? "#0f172a" : "#f8fafc";
  document.documentElement.style.color = ink;
  document.body.style.color = ink;
  var wrapper = document.createElement("div");
  wrapper.style.cssText =
    "width:100%;height:100%;overflow:hidden;position:relative";

  var content = renderEl(data as ElNode);
  content.style.width = "100%";
  content.style.height = "100%";
  if (!content.style.background && !content.style.backgroundColor) {
    content.style.background = surface;
  }
  wrapper.appendChild(content);

  var drag = document.createElement("div");
  drag.id = "drag-handle";
  drag.setAttribute("data-tauri-drag-region", "");
  wrapper.appendChild(drag);

  var cls = document.createElement("button");
  cls.id = "close-btn";
  cls.innerHTML = "&#x2715;";
  cls.onclick = function () {
    try {
      var label = window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label;
      if (!label) { window.close(); return; }
      invoke("plugin:widgets|close_widget_window", { label: label }).catch(
        function () {}
      );
    } catch (_) {
      window.close();
    }
  };
  wrapper.appendChild(cls);
  root.appendChild(wrapper);
  reportReceipt(
    cfg,
    source || "pull",
    source === "push" ? "reload" : "timeline",
    nonceHint
  );
}

function loadConfig() {
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
      root.innerHTML = '<div class="w-err">' + String(e) + "</div>";
    });
}

function init() {
  if (!window.__TAURI_INTERNALS__) {
    setTimeout(init, 50);
    return;
  }
  // Listeners BEFORE loadConfig — otherwise a push between the two is dropped.
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
