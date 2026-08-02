/**
 * `tauri-plugin-widgets` — JavaScript/TypeScript API.
 *
 * Build native widgets on Android (AppWidget), iOS/macOS (WidgetKit) and
 * desktop (frameless Tauri windows) from a single JSON configuration.
 *
 * Two API layers:
 *
 * 1. **Widget Config API** — send a declarative UI config, rendered natively.
 * 2. **Data API** — raw key-value storage shared with native widget extensions.
 *
 * @example
 * ```ts
 * import { setWidgetConfig, setItems, reloadAllTimelines } from "tauri-plugin-widgets-api";
 *
 * // Declarative UI
 * await setWidgetConfig({
 *   small: {
 *     type: "vstack", padding: 12, background: "#1a1a2e",
 *     children: [
 *       { type: "text", content: "72°", fontSize: 36, fontWeight: "bold", color: "#fff" },
 *       { type: "progress", value: 0.7, tint: "#4CAF50" },
 *     ],
 *   },
 * }, "group.com.example.myapp", "weather");
 *
 * // Or raw data for custom native widgets
 * await setItems("temperature", "72", "group.com.example.myapp");
 * await reloadAllTimelines();
 * ```
 *
 * @module
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

const PLUGIN_ID = "plugin:widgets";

// ─── Data API ────────────────────────────────────────────────────────────────

/**
 * Store a key-value pair in the widget data store.
 *
 * The storage backend depends on the platform:
 * - **Android** — `SharedPreferences` scoped by `group`.
 * - **iOS / macOS** — JSON file in App Group shared container.
 * - **Desktop** — JSON file in the app data directory (macOS: shared container when available).
 *
 * @param key   - Setting name (e.g. `"temperature"`).
 * @param value - Setting value. Use `JSON.stringify()` for complex data.
 * @param group - Widget group identifier:
 *                Android: SharedPreferences name;
 *                iOS/macOS: App Group ID (e.g. `"group.com.example.myapp"`);
 *                Desktop: arbitrary string used as filename.
 */
export async function setItems(
  key: string,
  value: string,
  group: string,
): Promise<boolean> {
  if (!key) throw new Error("setItems: 'key' must not be empty");
  if (!group) throw new Error("setItems: 'group' must not be empty");
  return await invoke<boolean>(`${PLUGIN_ID}|set_items`, { key, value, group });
}

/**
 * Read a previously stored value from the widget data store.
 *
 * @param key   - The key to look up.
 * @param group - The widget group identifier.
 * @returns The stored string value, or `null` if the key was never set.
 */
export async function getItems(
  key: string,
  group: string,
): Promise<string | null> {
  if (!key) throw new Error("getItems: 'key' must not be empty");
  if (!group) throw new Error("getItems: 'group' must not be empty");
  return await invoke<string | null>(`${PLUGIN_ID}|get_items`, { key, group });
}

// ─── Reload API ──────────────────────────────────────────────────────────────

/**
 * Reload all registered widget timelines.
 *
 * - **Android** — sends `ACTION_APPWIDGET_UPDATE` broadcast for every registered provider.
 * - **iOS / macOS** — calls `WidgetCenter.shared.reloadAllTimelines()`.
 * - **Desktop** — emits a `"widget-reload"` Tauri event that widget windows can listen for.
 */
export async function reloadAllTimelines(): Promise<boolean> {
  return await invoke<boolean>(`${PLUGIN_ID}|reload_all_timelines`);
}

/**
 * Reload timelines for a specific widget kind or class.
 *
 * @param ofKind - Widget class name (Android: fully-qualified class, e.g.
 *                 `"com.example.app.MyWidget"`; iOS: widget kind string from
 *                 your `WidgetConfiguration`).
 */
export async function reloadTimelines(ofKind: string): Promise<boolean> {
  if (!ofKind) throw new Error("reloadTimelines: 'ofKind' must not be empty");
  return await invoke<boolean>(`${PLUGIN_ID}|reload_timelines`, { ofKind });
}

// ─── Registration & Pinning ─────────────────────────────────────────────────

/**
 * Register widget provider class names or kind strings.
 *
 * Must be called before `reloadAllTimelines()` or `requestWidget()`.
 *
 * @param widgets - Array of provider identifiers:
 *   - **Android**: fully-qualified class names (e.g. `"com.example.app.MyWidget"`)
 *   - **iOS / macOS**: widget kind strings from your `WidgetConfiguration`
 */
export async function setRegisterWidget(
  widgets: string[],
): Promise<boolean> {
  if (!widgets?.length) {
    throw new Error("setRegisterWidget: 'widgets' must be a non-empty array");
  }
  return await invoke<boolean>(`${PLUGIN_ID}|set_register_widget`, { widgets });
}

/**
 * Request the OS to pin (add) a widget to the home screen.
 *
 * - **Android** (API 26+) — calls `AppWidgetManager.requestPinAppWidget`.
 * - **iOS / macOS / Desktop** — no-op (users add widgets manually).
 */
export async function requestWidget(): Promise<boolean> {
  return await invoke<boolean>(`${PLUGIN_ID}|request_widget`);
}

// ─── Widget Window API (desktop only) ───────────────────────────────────────

/** Configuration for creating a desktop widget window. */
export interface WidgetWindowConfig {
  /** Unique window label (e.g. `"cpu-widget"`). Used to reference the window later. */
  label: string;
  /**
   * Frontend route to load.  When omitted (or empty) the plugin
   * serves its built-in widget renderer automatically — no extra
   * files needed.  Set to a path (e.g. `"/widget.html"`) only if
   * you want to use your own custom renderer.
   */
  url?: string;
  /** Window width in logical pixels. */
  width: number;
  /** Window height in logical pixels. */
  height: number;
  /** X position on screen (pixels from left). */
  x?: number;
  /** Y position on screen (pixels from top). */
  y?: number;
  /** Keep above all other windows. Default: `false`. */
  alwaysOnTop?: boolean;
  /** Hide from taskbar / dock. Default: `true`. */
  skipTaskbar?: boolean;
  /**
   * Widget group identifier.  Required when using the built-in
   * renderer — tells it which config to load via `getWidgetConfig`.
   * Ignored when a custom `url` is provided.
   */
  group?: string;
  /**
   * Widget identity within the group. Required for the built-in renderer.
   */
  widgetId?: string;
  /**
   * Size family the built-in renderer should display:
   * `"small"`, `"medium"`, or `"large"`.  Defaults to `"small"`.
   */
  size?: "small" | "medium" | "large";
}

/**
 * Create a frameless, transparent desktop widget window.
 *
 * When no `url` is specified the plugin serves its built-in renderer
 * automatically — it reads the config via `getWidgetConfig(group)` and
 * renders it using vanilla HTML/CSS/SVG.  The window has a drag handle
 * at the top and a close button that appears on hover.
 *
 * You may provide your own `url` (e.g. `"/my-widget.html"`) if you
 * need full control over the rendering.
 *
 * **Desktop only.** Returns an error on mobile.
 *
 * @example
 * ```ts
 * // Built-in renderer (recommended):
 * await createWidgetWindow({
 *   label: "weather",
 *   width: 280,
 *   height: 200,
 *   group: "group.com.example.myapp",
 *   widgetId: "weather",
 *   size: "small",
 * });
 *
 * // Custom renderer:
 * await createWidgetWindow({
 *   label: "weather",
 *   url: "/my-widget.html",
 *   width: 280,
 *   height: 200,
 * });
 * ```
 */
export async function createWidgetWindow(
  config: WidgetWindowConfig,
): Promise<boolean> {
  if (!config.label) throw new Error("createWidgetWindow: 'label' is required");
  if (!config.url) {
    if (!config.group) {
      throw new Error("createWidgetWindow: 'group' is required for the built-in renderer");
    }
    if (!config.widgetId) {
      throw new Error("createWidgetWindow: 'widgetId' is required for the built-in renderer");
    }
  }
  return await invoke<boolean>(`${PLUGIN_ID}|create_widget_window`, { config });
}

/**
 * Close a previously created desktop widget window.
 *
 * @param label - The window label used when creating it.
 *
 * **Desktop only.**
 */
export async function closeWidgetWindow(label: string): Promise<boolean> {
  if (!label) throw new Error("closeWidgetWindow: 'label' is required");
  return await invoke<boolean>(`${PLUGIN_ID}|close_widget_window`, { label });
}

// ─── Widget Config API ──────────────────────────────────────────────────────

/** IR types generated from Rust `src/models.rs` (SoT). Do not hand-edit. */
export * from "./generated/widget-types";
import type { WidgetConfig } from "./generated/widget-types";

/**
 * Why a config write was skipped.
 */
export type SkipReason = {
  reason: "unchanged";
  hash: number;
};

/**
 * Result of a native WidgetKit / AppWidget reload attempt.
 */
export type ReloadOutcome =
  | { outcome: "ok" }
  | { outcome: "throttled"; remainingSecs: number }
  | { outcome: "skipped"; why: string }
  | { outcome: "failed"; error: string };

/**
 * Full result of {@link setWidgetConfig}. Never treat success as “reload happened”.
 */
export interface ApplyOutcome {
  /** `true` when store bytes changed and were written. */
  written: boolean;
  reload: ReloadOutcome;
  /** Transport names that received the map (empty when not written). */
  transports: string[];
  skip?: SkipReason;
}

/**
 * Send a declarative UI configuration to native widgets.
 *
 * The JSON config is stored in the widget data store and read by the native
 * widget extension, which renders it using platform-native components:
 * - **iOS / macOS** — SwiftUI views (`VStack`, `Text`, `Gauge`, etc.)
 * - **Android** — `RemoteViews` (`LinearLayout`, `TextView`, etc.)
 * - **Desktop** — HTML/CSS in the widget window
 *
 * Returns {@link ApplyOutcome}: check `written` and `reload.outcome` — a
 * successful invoke no longer means a native reload was performed.
 *
 * @param config - The widget UI configuration.
 * @param group  - Widget group identifier (same as `setItems` group).
 *
 * @example
 * ```ts
 * const outcome = await setWidgetConfig({
 *   small: {
 *     type: "vstack",
 *     padding: 12,
 *     background: "#1a1a2e",
 *     children: [
 *       { type: "text", content: "72°", fontSize: 36, fontWeight: "bold", color: "#fff" },
 *       { type: "progress", value: 0.7, tint: "#4CAF50" },
 *     ],
 *   },
 * }, "group.com.example.myapp", "weather");
 * if (outcome.reload.outcome === "throttled") {
 *   console.warn("reload delayed", outcome.reload.remainingSecs);
 * }
 * ```
 */
export async function setWidgetConfig(
  config: WidgetConfig,
  group: string,
  widgetId: string,
  /** Skip native widget reload (WidgetKit / AppWidgetManager).
   *  Desktop widget windows are always updated instantly via eval push. */
  skipReload = false,
): Promise<ApplyOutcome> {
  if (!group) throw new Error("setWidgetConfig: 'group' must not be empty");
  if (!widgetId) throw new Error("setWidgetConfig: 'widgetId' must not be empty");
  return await invoke<ApplyOutcome>(`${PLUGIN_ID}|set_widget_config`, {
    config, group, widgetId, skipReload,
  });
}

/**
 * Read the current widget UI configuration from the data store.
 *
 * @param group - Widget group identifier.
 * @param widgetId - Widget identity within the group.
 * @returns The current `WidgetConfig`, or `null` if none has been set.
 */
export async function getWidgetConfig(
  group: string,
  widgetId: string,
): Promise<WidgetConfig | null> {
  if (!group) throw new Error("getWidgetConfig: 'group' must not be empty");
  if (!widgetId) throw new Error("getWidgetConfig: 'widgetId' must not be empty");
  return await invoke<WidgetConfig | null>(`${PLUGIN_ID}|get_widget_config`, {
    group,
    widgetId,
  });
}

/** Element the renderer skipped, with reason. */
export interface SkippedElement {
  type: string;
  reason: string;
}

/** Cross-platform render receipt (diagnostics — not a critical path). */
export interface WidgetRenderReceipt {
  widgetId: string;
  group: string;
  instance: string;
  nonce: number;
  size?: string;
  theme?: string;
  schema?: number;
  /** prefs | state | appgroup | defaults | container | push | pull */
  source: string;
  /** reload | timeline | action | added | resize | snapshot */
  trigger?: string;
  rendered?: string[];
  skipped?: SkippedElement[];
  ts: number;
}

/** One host journal entry (`WIDGET_DEBUG` / debug builds). */
export type TraceEvent =
  | {
      kind: "configSet";
      widgetId: string;
      nonce: number;
      bytes: number;
      changed: boolean;
      skip?: { reason: "unchanged"; hash: number } | { reason: "noInstances" } | {
        reason: "transportUnavailable";
        name: string;
      };
    }
  | {
      kind: "write";
      transport: string;
      ok: boolean;
      durationMs: number;
      error?: string;
    }
  | { kind: "reload"; performed: boolean; reason: ReloadOutcome }
  | { kind: "poll"; count: number }
  | {
      kind: "render";
      instance: string;
      nonce: number;
      source: string;
      trigger: string;
      lagMs: number;
      skipped: SkippedElement[];
    };

export interface TraceEntry {
  ts: number;
  kind: TraceEvent["kind"];
  [key: string]: unknown;
}

export interface WidgetTrace {
  enabled: boolean;
  events: TraceEntry[];
  receipts: WidgetRenderReceipt[];
}

/**
 * Report that a renderer painted a config (desktop widget.html / tests).
 * Native widgets write receipts themselves; this is for the host webview path.
 */
export async function reportReceipt(receipt: WidgetRenderReceipt): Promise<boolean> {
  return await invoke<boolean>(`${PLUGIN_ID}|report_receipt`, { receipt });
}

/**
 * Live widget instances that recently rendered for `group`.
 */
export async function getWidgetDiagnostics(
  group: string,
): Promise<WidgetRenderReceipt[]> {
  if (!group) throw new Error("getWidgetDiagnostics: 'group' must not be empty");
  return await invoke<WidgetRenderReceipt[]>(`${PLUGIN_ID}|get_widget_diagnostics`, {
    group,
  });
}

/**
 * Host delivery journal + receipt history for `group`.
 * Active in debug builds or when `WIDGET_DEBUG=1`.
 */
export async function getWidgetTrace(
  group: string,
  opts?: { since?: number },
): Promise<WidgetTrace> {
  if (!group) throw new Error("getWidgetTrace: 'group' must not be empty");
  return await invoke<WidgetTrace>(`${PLUGIN_ID}|get_widget_trace`, {
    group,
    sinceMs: opts?.since ?? null,
  });
}

/** Force-flush the in-memory journal to disk (desktop). */
export async function flushWidgetTrace(): Promise<boolean> {
  return await invoke<boolean>(`${PLUGIN_ID}|flush_widget_trace`);
}

// ─── Widget Action API ──────────────────────────────────────────────────────

/** Payload delivered by the `widget-action` event. */
export interface WidgetActionPayload {
  /** The action identifier from the button config. */
  action: string;
  /** Optional extra data (e.g. serialized JSON from the widget). */
  payload?: string;
  /** Unix epoch ms when the action was enqueued / emitted. */
  ts?: number;
  /** Widget identity that emitted the action. */
  widgetId?: string;
  /** App group / prefs namespace. */
  group?: string;
}

/**
 * Trigger a `widget-action` event via the Rust plugin.
 *
 * The Rust command broadcasts the event through `app.emit()`, so every
 * window that called `onWidgetAction()` (or `listen("widget-action", …)`)
 * will receive it — including the main application window.
 *
 * @param action  - Action identifier (matches `ButtonElement.action`).
 * @param payload - Optional extra string payload.
 */
export async function widgetAction(
  action: string,
  payload?: string,
  opts?: { widgetId?: string; group?: string },
): Promise<boolean> {
  if (!action) throw new Error("widgetAction: 'action' must not be empty");
  return await invoke<boolean>(`${PLUGIN_ID}|widget_action`, {
    action,
    payload,
    widgetId: opts?.widgetId,
    group: opts?.group,
  });
}

/**
 * Listen for `widget-action` events emitted by widget buttons.
 *
 * @param callback - Called with the action payload each time a widget button
 *                   with an `action` field is tapped.
 * @returns A function to stop listening.
 *
 * @example
 * ```ts
 * import { onWidgetAction } from "tauri-plugin-widgets-api";
 *
 * const unlisten = await onWidgetAction((data) => {
 *   console.log("Widget action:", data.action, data.payload);
 *   if (data.action === "refresh_data") {
 *     fetchLatestData();
 *   }
 * });
 * ```
 */
export async function onWidgetAction(
  callback: (data: WidgetActionPayload) => void,
): Promise<UnlistenFn> {
  return await listen<WidgetActionPayload>("widget-action", (event) => {
    callback(event.payload);
  });
}

/** Poll pending widget actions queued by native side (Android fallback path). */
export async function pollPendingWidgetActions(
  group: string,
): Promise<WidgetActionPayload[]> {
  let res: unknown;
  try {
    res = await invoke<unknown>(`${PLUGIN_ID}|poll_pending_actions`, { group });
  } catch (e) {
    const msg = String(e ?? "");
    // iOS/macOS builds may not expose this Android-specific command path.
    if (msg.includes("No command pollPendingActions") || msg.includes("pollPendingActions")) {
      return [];
    }
    throw e;
  }
  const raw = Array.isArray(res)
    ? res
    : ((res as { results?: unknown } | null | undefined)?.results ?? null);
  if (!Array.isArray(raw)) return [];
  const parsed: Array<WidgetActionPayload | null> = raw.map((item) => {
    if (!item || typeof item !== "object") return null;
    const obj = item as {
      action?: unknown;
      payload?: unknown;
      ts?: unknown;
      widgetId?: unknown;
      group?: unknown;
    };
    if (typeof obj.action !== "string" || obj.action.length === 0) return null;
    return {
      action: obj.action,
      ...(typeof obj.payload === "string" ? { payload: obj.payload } : {}),
      ...(typeof obj.ts === "number" ? { ts: obj.ts } : {}),
      ...(typeof obj.widgetId === "string" ? { widgetId: obj.widgetId } : {}),
      ...(typeof obj.group === "string" ? { group: obj.group } : {}),
    };
  });
  return parsed.filter((v): v is WidgetActionPayload => v !== null);
}

// ─── Widget Updater ─────────────────────────────────────────────────────────

/**
 * Handlers registered by `startWidgetUpdater` instances, keyed by group then
 * widgetId. Polling a group's pending-action queue drains it entirely, so a
 * single shared dispatcher is used to route every envelope to the matching
 * sibling widget instead of losing actions to whichever updater happened to
 * poll first.
 */
const pendingActionHandlers = new Map<string, Map<string, (data: WidgetActionPayload) => void>>();
/** In-flight poll per group, so concurrent ticks share a single drain. */
const pendingActionPolls = new Map<string, Promise<void>>();

function pollAndDispatchPendingActions(group: string): Promise<void> {
  const inFlight = pendingActionPolls.get(group);
  if (inFlight) return inFlight;
  const promise = (async () => {
    let pending: WidgetActionPayload[];
    try {
      pending = await pollPendingWidgetActions(group);
    } catch (e) {
      console.debug("[widget-updater] pollPendingWidgetActions failed:", e);
      return;
    }
    const handlers = pendingActionHandlers.get(group);
    if (!handlers || handlers.size === 0) return;
    for (const item of pending) {
      for (const [widgetId, handler] of handlers) {
        if (item.widgetId && item.widgetId !== widgetId) continue;
        if (item.group && item.group !== group) continue;
        handler(item);
      }
    }
  })();
  pendingActionPolls.set(group, promise);
  return promise.finally(() => {
    if (pendingActionPolls.get(group) === promise) pendingActionPolls.delete(group);
  });
}

/**
 * Start a periodic widget updater that calls `builder` on a fixed interval,
 * sends the returned config to the native widget, and optionally reloads
 * widget timelines.
 *
 * This is a convenience wrapper around `setInterval` + `setWidgetConfig` +
 * `reloadAllTimelines` so you don't have to write boilerplate timers.
 *
 * **Important (iOS/macOS):** Apple enforces a daily widget reload budget
 * (~40-70 reloads/day). The Rust backend throttles `reloadAllTimelines()`
 * to at most once every 15 minutes. For frequent data updates, set
 * `reload: false` (the default) — desktop webview widgets update instantly
 * via events without counting against the budget.
 *
 * @param builder    - Called every `intervalMs`. Return the new `WidgetConfig`.
 *                     May be async.
 * @param group      - App Group / SharedPreferences group identifier.
 * @param options    - Optional settings.
 * @param options.intervalMs  - Update interval in milliseconds (default `1000`).
 *                              For native mobile widgets, consider using a
 *                              longer interval (60000+) to avoid wasted work.
 * @param options.immediate   - If `true`, run the builder immediately before
 *                              the first interval tick (default `true`).
 * @param options.reload      - If `true`, call `reloadAllTimelines()` after
 *                              each update so native widgets refresh
 *                              (default `false`). Set to `true` for native
 *                              widgets; leave `false` for desktop-only.
 *                              Note: backend throttles this to once per 15 min
 *                              on iOS/macOS to respect the budget.
 * @returns A `stop()` function. Call it to clear the timer.
 *
 * @example
 * ```ts
 * import { startWidgetUpdater } from "tauri-plugin-widgets-api";
 *
 * const stop = await startWidgetUpdater(
 *   () => ({
 *     small: {
 *       type: "vstack", padding: 12, background: "#1a1a2e",
 *       children: [
 *         { type: "text", content: new Date().toLocaleTimeString(),
 *           fontSize: 24, fontWeight: "bold", color: "#fff" },
 *       ],
 *     },
 *   }),
 *   "group.com.example.myapp",
 *   "clock",
 *   {
 *     intervalMs: 60_000,
 *     reload: true,
 *     onAction: (action, payload) => {
 *       console.log("Widget action:", action, payload);
 *     },
 *   },
 * );
 *
 * // later — stop updating and action listener:
 * stop();
 * ```
 */
export async function startWidgetUpdater(
  builder: () => WidgetConfig | Promise<WidgetConfig>,
  group: string,
  widgetId: string,
  options?: {
    /** Tick interval in ms.  Use `0` for a one-shot call (no periodic updates). Default: `1000`. */
    intervalMs?: number;
    /** Call `builder` immediately before starting the interval. Default: `true`. */
    immediate?: boolean;
    /** Also call `reloadAllTimelines()` after each tick. Default: `false`.
     *  Backend throttles to once per 15 min on iOS/macOS. */
    reload?: boolean;
    /** Subscribe to `widget-action` events for the lifetime of this updater.
     *  The listener is automatically removed when the returned stop function is called. */
    onAction?: (action: string, payload?: string) => void;
  },
): Promise<() => void> {
  if (!widgetId) throw new Error("startWidgetUpdater: 'widgetId' must not be empty");
  const intervalMs = options?.intervalMs ?? 5000;
  const immediate = options?.immediate ?? true;
  const reload = options?.reload ?? false;

  let running = false;
  let cancelled = false;

  if (options?.onAction) {
    const handler = options.onAction;
    let handlers = pendingActionHandlers.get(group);
    if (!handlers) {
      handlers = new Map();
      pendingActionHandlers.set(group, handlers);
    }
    handlers.set(widgetId, (item) => handler(item.action, item.payload));
  }

  async function tick() {
    if (cancelled || running) return;
    running = true;
    try {
      // Drain native pending_actions (iOS WidgetKit / Android fallback) into handlers.
      // Only poll when an action handler is registered — otherwise draining
      // would silently discard actions nobody is listening for.
      if (options?.onAction) {
        await pollAndDispatchPendingActions(group);
        if (cancelled) return;
      }

      const config = await builder();
      if (cancelled) return;
      // skipReload: true — the native reload-on-write is suppressed here so
      // the explicit reloadAllTimelines() call below stays the single source
      // of truth for `options.reload`, matching the documented behavior.
      await setWidgetConfig(config, group, widgetId, true);
      if (cancelled) return;
      if (reload) {
        await reloadAllTimelines();
      }
    } catch (e) {
      if (!cancelled) console.error("[widget-updater] tick failed:", e);
    } finally {
      running = false;
    }
  }

  if (immediate) {
    await tick();
  }

  const id = !cancelled && intervalMs > 0 ? setInterval(tick, intervalMs) : null;

  let actionUnsub: (() => void) | null = null;
  if (!cancelled && options?.onAction) {
    const handler = options.onAction;
    actionUnsub = await onWidgetAction((data) => {
      if (cancelled) return;
      if (data.widgetId && data.widgetId !== widgetId) return;
      if (data.group && data.group !== group) return;
      handler(data.action, data.payload);
    });
  }

  return () => {
    cancelled = true;
    if (id !== null) clearInterval(id);
    if (actionUnsub) { actionUnsub(); actionUnsub = null; }
    const handlers = pendingActionHandlers.get(group);
    if (handlers) {
      handlers.delete(widgetId);
      if (handlers.size === 0) pendingActionHandlers.delete(group);
    }
  };
}
