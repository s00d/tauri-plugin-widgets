import { invoke } from "@tauri-apps/api/core";
import { PLUGIN_ID } from "./plugin";

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
