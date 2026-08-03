import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { PLUGIN_ID } from "./plugin";

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
