import { invoke } from "@tauri-apps/api/core";
import { PLUGIN_ID } from "./plugin";

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
