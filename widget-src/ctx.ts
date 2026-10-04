function tauri(): NonNullable<Window["__TAURI_INTERNALS__"]> {
  const t = window.__TAURI_INTERNALS__;
  if (!t) throw new Error("Tauri internals unavailable");
  return t;
}

export function invoke(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<unknown> {
  return tauri().invoke(cmd, args || {});
}

export function listen(
  event: string,
  handler: (ev: { payload: unknown }) => void,
): Promise<unknown> {
  const id = tauri().transformCallback((e) => {
    handler(e);
  });
  const ch = {
    id,
    __TAURI_CHANNEL_MARKER__: true as const,
    toJSON() {
      return `__CHANNEL__:${id}`;
    },
  };
  return invoke("plugin:event|listen", {
    event,
    target: { kind: "Any" },
    handler: ch,
  });
}

const params =
  typeof window !== "undefined"
    ? new URLSearchParams(window.location.search)
    : new URLSearchParams();
export const GROUP = params.get("group") || "default";
export const SIZE = params.get("size") || "";
export const WIDGET_ID = params.get("widgetId") || "default";
export const THEME = (params.get("theme") || "").toLowerCase();
export const root =
  typeof document !== "undefined"
    ? (document.getElementById("root") as HTMLElement | null)
    : null;

type ActionHandler = (action: string, payload?: string | null) => void;

let sessionSize = SIZE;
let sessionTheme = THEME;
let actionHandler: ActionHandler | null = null;

export function setRenderSession(opts: {
  size?: string;
  theme?: string;
  onAction?: ActionHandler;
}): void {
  if (opts.size !== undefined) sessionSize = opts.size;
  if (opts.theme !== undefined) sessionTheme = opts.theme;
  if (opts.onAction !== undefined) actionHandler = opts.onAction;
}

export function getSize(): string {
  return sessionSize;
}

export function getTheme(): string {
  return sessionTheme;
}

export function emitAction(
  action: string,
  payload?: string | null,
): Promise<unknown> {
  if (actionHandler) {
    actionHandler(action, payload);
    return Promise.resolve();
  }
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return Promise.resolve();
  }
  return invoke("plugin:widgets|widget_action", {
    action,
    payload: payload == null ? null : payload,
    widgetId: WIDGET_ID,
    group: GROUP,
  }).catch(console.error);
}

export let timerIds: ReturnType<typeof setInterval>[] = [];

export function clearTimers(): void {
  timerIds.forEach((id) => {
    clearInterval(id);
  });
  timerIds = [];
}
