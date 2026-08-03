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

export * from "./generated/widget-types";

export { PLUGIN_ID } from "./api/plugin";
export { setItems, getItems } from "./api/data";
export { reloadAllTimelines, reloadTimelines } from "./api/reload";
export { setRegisterWidget, requestWidget } from "./api/register";
export {
  createWidgetWindow,
  closeWidgetWindow,
  type WidgetWindowConfig,
} from "./api/window";
export {
  setWidgetConfig,
  getWidgetConfig,
  type SkipReason,
  type ReloadOutcome,
  type ApplyOutcome,
} from "./api/config";
export {
  reportReceipt,
  getWidgetDiagnostics,
  getWidgetTrace,
  flushWidgetTrace,
  type SkippedElement,
  type WidgetRenderReceipt,
  type TraceEvent,
  type TraceEntry,
  type WidgetTrace,
} from "./api/diagnostics";
export {
  widgetAction,
  onWidgetAction,
  pollPendingWidgetActions,
  type WidgetActionPayload,
} from "./api/actions";
export { startWidgetUpdater } from "./api/updater";
