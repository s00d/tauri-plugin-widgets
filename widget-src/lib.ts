import type { ElNode, WidgetConfig } from "./types";
import { clearTimers, getSize, setRenderSession } from "./ctx";
import { isDark } from "./style";
import { renderEl } from "./render/element";

export type WidgetSize = "small" | "medium" | "large";
export type WidgetTheme = "light" | "dark";

export type RenderWidgetOptions = {
  size?: WidgetSize | string;
  theme?: WidgetTheme | string;
  chrome?: boolean;
  onAction?: (action: string, payload?: string | null) => void;
};

export type RenderHandle = { destroy(): void };

const HOST_CSS = `
.widget-host{width:100%;height:100%;overflow:hidden;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;color:#e5e7eb;background:transparent;-webkit-user-select:none;user-select:none}
.widget-host .w-empty,.widget-host .w-err{display:flex;align-items:center;justify-content:center;height:100%;color:#6b7280;font-size:13px;text-align:center;padding:16px}
.widget-host .w-err{color:#f87171}
.widget-host #drag-handle{position:absolute;top:0;left:0;right:32px;height:28px;z-index:10;cursor:move}
.widget-host #close-btn{position:absolute;top:6px;right:6px;width:20px;height:20px;border:none;border-radius:50%;background:rgba(255,255,255,.12);color:#fff;font-size:11px;cursor:pointer;z-index:11;display:flex;align-items:center;justify-content:center;line-height:1;padding:0}
.widget-host #close-btn:hover{background:rgba(239,68,68,.8)}
`.trim();

function ensureHostCss(): void {
  if (typeof document === "undefined") return;
  if (document.getElementById("widget-host-css")) return;
  const style = document.createElement("style");
  style.id = "widget-host-css";
  style.textContent = HOST_CSS;
  document.head.appendChild(style);
}

function pickBranch(cfg: WidgetConfig, size: string): ElNode | undefined {
  const keyed = cfg as WidgetConfig & Record<string, ElNode | undefined>;
  if (size) return keyed[size] || cfg.small || cfg.medium || cfg.large;
  return cfg.small || cfg.medium || cfg.large;
}

function paintEmpty(host: HTMLElement, message: string): void {
  host.textContent = "";
  const empty = document.createElement("div");
  empty.className = "w-empty";
  const emptyInner = document.createElement("div");
  const icon = document.createElement("div");
  icon.style.cssText = "font-size:28px;margin-bottom:4px";
  icon.textContent = "\uD83D\uDCCC";
  const msg = document.createElement("div");
  msg.style.cssText = "font-size:14px;opacity:.7";
  msg.textContent = message;
  emptyInner.appendChild(icon);
  emptyInner.appendChild(msg);
  empty.appendChild(emptyInner);
  host.appendChild(empty);
}

function paintConfig(
  host: HTMLElement,
  cfg: WidgetConfig | null | undefined,
  chrome: boolean,
): void {
  clearTimers();
  host.replaceChildren();
  if (!cfg) {
    paintEmpty(host, "No widget config");
    return;
  }
  const size = getSize();
  const data = pickBranch(cfg, size);
  if (!data) {
    paintEmpty(host, 'No config for size "' + size + '"');
    return;
  }

  const ink = isDark() ? "#e5e7eb" : "#111827";
  const surface = isDark() ? "#0f172a" : "#f8fafc";
  host.style.color = ink;

  const wrapper = document.createElement("div");
  wrapper.style.cssText = "width:100%;height:100%;overflow:hidden;position:relative";

  const content = renderEl(data);
  content.style.width = "100%";
  content.style.height = "100%";
  if (!content.style.background && !content.style.backgroundColor) {
    content.style.background = surface;
  }
  wrapper.appendChild(content);

  if (chrome) {
    const drag = document.createElement("div");
    drag.id = "drag-handle";
    drag.setAttribute("data-tauri-drag-region", "");
    wrapper.appendChild(drag);

    const cls = document.createElement("button");
    cls.id = "close-btn";
    cls.innerHTML = "&#x2715;";
    cls.onclick = function () {
      try {
        const label = window.__TAURI_INTERNALS__?.metadata?.currentWindow?.label;
        if (!label) {
          window.close();
          return;
        }
        const internals = window.__TAURI_INTERNALS__;
        if (!internals) {
          window.close();
          return;
        }
        internals
          .invoke("plugin:widgets|close_widget_window", { label: label })
          .catch(function () {});
      } catch (_) {
        window.close();
      }
    };
    wrapper.appendChild(cls);
  }
  host.appendChild(wrapper);
}

export function renderWidget(
  host: HTMLElement,
  config: WidgetConfig | null | undefined,
  opts?: RenderWidgetOptions,
): RenderHandle {
  ensureHostCss();
  host.classList.add("widget-host");
  const size = opts?.size ?? "";
  const theme = (opts?.theme || "").toLowerCase();
  if (theme) host.setAttribute("data-theme", theme);
  else host.removeAttribute("data-theme");
  setRenderSession({
    size,
    theme,
    onAction: opts?.onAction,
  });
  paintConfig(host, config, !!opts?.chrome);
  return {
    destroy() {
      clearTimers();
      host.replaceChildren();
    },
  };
}
