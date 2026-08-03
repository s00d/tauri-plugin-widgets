import { invoke } from "@tauri-apps/api/core";
import type { WidgetConfig } from "../generated/widget-types";
import { PLUGIN_ID } from "./plugin";

/**
 * Why a config write was skipped.
 */
export type SkipReason =
  | { reason: "unchanged"; hash: number }
  | { reason: "noInstances" }
  | { reason: "transportUnavailable"; name: string };

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
