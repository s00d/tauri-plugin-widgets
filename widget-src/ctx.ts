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

type ActionHandler = (action: string, payload?: string | null) => void;

let sessionSize = "";
let sessionTheme = "";
let sessionGroup = "default";
let sessionWidgetId = "default";
let actionHandler: ActionHandler | null = null;

export function setRenderSession(opts: {
  size?: string;
  theme?: string;
  group?: string;
  widgetId?: string;
  onAction?: ActionHandler;
}): void {
  if (opts.size !== undefined) sessionSize = opts.size;
  if (opts.theme !== undefined) sessionTheme = opts.theme;
  if (opts.group !== undefined) sessionGroup = opts.group;
  if (opts.widgetId !== undefined) sessionWidgetId = opts.widgetId;
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
    widgetId: sessionWidgetId,
    group: sessionGroup,
  }).catch(console.error);
}

export let timerIds: ReturnType<typeof setInterval>[] = [];

export function clearTimers(): void {
  timerIds.forEach((id) => {
    clearInterval(id);
  });
  timerIds = [];
}
