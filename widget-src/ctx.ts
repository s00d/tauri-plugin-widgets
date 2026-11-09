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

/** State of one `renderWidget` call. Rendering is synchronous, so renderers read the
 *  current session while building; click handlers and timers keep their own session,
 *  so several widgets on one page do not share handlers or kill each other's timers. */
export type RenderSession = {
  size: string;
  theme: string;
  group: string;
  widgetId: string;
  onAction: ActionHandler | null;
  timers: ReturnType<typeof setInterval>[];
};

let current: RenderSession = newSession({});

function newSession(opts: {
  size?: string;
  theme?: string;
  group?: string;
  widgetId?: string;
  onAction?: ActionHandler;
}): RenderSession {
  return {
    size: opts.size ?? "",
    theme: opts.theme ?? "",
    // "" = the app's default group (plugins.widgets.appGroup).
    group: opts.group ?? "",
    widgetId: opts.widgetId ?? "default",
    onAction: opts.onAction ?? null,
    timers: [],
  };
}

/** Start a render: everything built until the next call belongs to this session. */
export function setRenderSession(opts: {
  size?: string;
  theme?: string;
  group?: string;
  widgetId?: string;
  onAction?: ActionHandler;
}): RenderSession {
  current = newSession(opts);
  return current;
}

export function getSize(): string {
  return current.size;
}

export function getTheme(): string {
  return current.theme;
}

/** Action emitter bound to the session being rendered (call while rendering). */
export function boundEmit(): (action: string, payload?: string | null) => Promise<unknown> {
  const session = current;
  return (action, payload) => emitAction(session, action, payload);
}

function emitAction(
  session: RenderSession,
  action: string,
  payload?: string | null,
): Promise<unknown> {
  if (session.onAction) {
    session.onAction(action, payload);
    return Promise.resolve();
  }
  if (typeof window === "undefined" || !window.__TAURI_INTERNALS__) {
    return Promise.resolve();
  }
  return invoke("plugin:widgets|widget_action", {
    action,
    payload: payload == null ? null : payload,
    widgetId: session.widgetId,
    group: session.group || null,
  }).catch(console.error);
}

/** Keep a timer with the session being rendered. */
export function addTimer(id: ReturnType<typeof setInterval>): void {
  current.timers.push(id);
}

export function clearTimers(session: RenderSession = current): void {
  session.timers.forEach((id) => {
    clearInterval(id);
  });
  session.timers = [];
}
