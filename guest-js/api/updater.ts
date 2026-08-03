import type { WidgetConfig } from "../generated/widget-types";
import { setWidgetConfig } from "./config";
import { reloadAllTimelines } from "./reload";
import {
  onWidgetAction,
  pollPendingWidgetActions,
  type WidgetActionPayload,
} from "./actions";

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
 * @param options.intervalMs  - Update interval in milliseconds (default `5000`).
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

  // Register the event listener BEFORE the immediate tick so a macOS
  // background poller / host emit cannot win the race and drop actions.
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

  if (immediate) {
    await tick();
  }

  const id = !cancelled && intervalMs > 0 ? setInterval(tick, intervalMs) : null;

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
