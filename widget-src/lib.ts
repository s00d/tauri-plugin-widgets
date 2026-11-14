import type { ElNode, WidgetConfig } from "./types";
import { clearTimers, getSize, setRenderSession, type RenderSession, type SetHandler } from "./ctx";
import { isDark } from "./style";
import { renderEl } from "./render/element";
import { greedy } from "./render/layout";
import { nextChangeMs, resolveConfig, type ResolveContext } from "./resolve";

export {
  resolveConfig,
  nextChangeMs,
  mergeData,
  sourceRequest,
  type ResolveContext,
} from "./resolve";

export type WidgetSize = "small" | "medium" | "large";
export type WidgetTheme = "light" | "dark";

export type RenderWidgetOptions = {
  size?: WidgetSize | string;
  theme?: WidgetTheme | string;
  group?: string;
  widgetId?: string;
  chrome?: boolean;
  /** Root background when the layout sets none (default: the theme's surface). */
  surface?: string;
  onAction?: (action: string, payload?: string | null) => void;
  /** Tap on an element with `set`: write the patch into the placement's state and render
   *  again with the new `context.state` (the built-in window does it via `set_widget_state`).
   *  Without it `set` is ignored. */
  onSet?: SetHandler;
  /** Render-time context for `entries` / `{{…}}`. Defaults: now, `navigator.language`. */
  context?: Partial<ResolveContext>;
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

/** Last render per host: re-rendering a host stops only that host's old timers. */
const hostSessions = new WeakMap<HTMLElement, RenderSession>();
/** Content of the `data-transition` elements in the host's last render, in DOM order. */
const hostTransitions = new WeakMap<HTMLElement, string[]>();

const TRANSITION_FRAMES: Record<string, Keyframe[]> = {
  opacity: [{ opacity: 0 }, { opacity: 1 }],
  slide: [{ transform: "translateY(40%)", opacity: 0 }, { transform: "none", opacity: 1 }],
  numeric: [{ transform: "translateY(35%)", opacity: 0.2 }, { transform: "none", opacity: 1 }],
};

/** `transition` hint: animate marked elements whose content differs from the last render
 *  of this host (matched by order). The first render does not animate. */
function animateTransitions(host: HTMLElement): void {
  const els = Array.from(host.querySelectorAll<HTMLElement>("[data-transition]"));
  const now = els.map((e) => e.getAttribute("data-transition") + "|" + e.innerHTML);
  const before = hostTransitions.get(host);
  hostTransitions.set(host, now);
  if (!before) return;
  els.forEach((e, i) => {
    const frames = TRANSITION_FRAMES[e.getAttribute("data-transition") || ""];
    if (frames && before[i] !== undefined && before[i] !== now[i] && typeof e.animate === "function") {
      e.animate(frames, { duration: 300, easing: "ease-out" });
    }
  });
}

function paintConfig(
  host: HTMLElement,
  cfg: WidgetConfig | null | undefined,
  chrome: boolean,
  surfaceOverride?: string,
): void {
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
  const surface = surfaceOverride || (isDark() ? "#0f172a" : "#f8fafc");
  host.style.color = ink;

  const wrapper = document.createElement("div");
  wrapper.style.cssText = "width:100%;height:100%;overflow:hidden;position:relative";

  const content = renderEl(data);
  const rootTransition = (cfg as { transition?: unknown }).transition;
  if (typeof rootTransition === "string" && rootTransition !== "none" && !content.hasAttribute("data-transition")) {
    content.setAttribute("data-transition", rootTransition);
  }
  content.style.width = "100%";
  content.style.height = "100%";
  if (!content.style.background && !content.style.backgroundColor) {
    content.style.background = surface;
  }
  if (data.type === "hstack" && !greedy(data, "h", undefined)) {
    // WidgetKit pins a content-height root row to the top; only the backdrop spans the widget.
    content.style.height = "auto";
    wrapper.style.background = content.style.background;
    wrapper.style.borderRadius = content.style.borderRadius;
    content.style.background = "";
  }
  // Apple always rounds the widget (16 in WidgetChrome.swift when the root sets none);
  // `overflow:hidden` clips the root's background to it.
  wrapper.style.borderRadius = wrapper.style.borderRadius || content.style.borderRadius || "16px";
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
  animateTransitions(host);
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
  const previous = hostSessions.get(host);
  if (previous) clearTimers(previous);
  const session = setRenderSession({
    size,
    theme,
    group: opts?.group,
    widgetId: opts?.widgetId,
    onAction: opts?.onAction,
    onSet: opts?.onSet,
  });
  hostSessions.set(host, session);
  const ctx: ResolveContext = {
    nowMs: opts?.context?.nowMs ?? Date.now(),
    locale: opts?.context?.locale ?? (typeof navigator !== "undefined" ? navigator.language : "") ?? "",
    instance: opts?.context?.instance ?? null,
    data: opts?.context?.data,
    state: opts?.context?.state,
  };
  const resolved = config ? (resolveConfig(config, ctx) as WidgetConfig) : config;
  if (resolved && config && config.__nonce != null) resolved.__nonce = config.__nonce;
  // Resolve keeps only the families; the config-level hint applies to the root.
  if (resolved && config && config.transition) resolved.transition = config.transition;
  paintConfig(host, resolved, !!opts?.chrome, opts?.surface);
  // Switch to the next scheduled entry on time (only when the caller did not pin `nowMs`).
  const next = config && opts?.context?.nowMs === undefined ? nextChangeMs(config, ctx.nowMs) : null;
  if (next !== null) {
    session.timers.push(
      setTimeout(function () {
        if (hostSessions.get(host) === session) renderWidget(host, config, opts);
      }, Math.min(next - ctx.nowMs + 50, 2147483647)),
    );
  }
  return {
    destroy() {
      clearTimers(session);
      if (hostSessions.get(host) === session) hostSessions.delete(host);
      host.replaceChildren();
    },
  };
}
