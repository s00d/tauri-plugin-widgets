import { invoke } from "@tauri-apps/api/core";
import { PLUGIN_ID } from "./plugin";

/** A widget placed right now on a home screen, desktop or Widgets Board. */
export interface ActiveWidget {
  /** Logical widget id (WidgetKit kind / config `widgetId`). */
  widgetId: string;
  /**
   * Placement id: Android `appWidgetId`, the Apple Edit Widget choice, a window
   * instance / label, a Widgets Board id. Absent when the placement has none.
   */
  instance?: string;
  /** `small` | `medium` | `large` when known. */
  size?: string;
  /** Unix ms of the last render, when known from render receipts. */
  lastSeenMs?: number;
}

/** A choice in the Apple Edit Widget sheet (e.g. a city). */
export interface WidgetInstanceOption {
  /** Placement id: `{{instance}}`, `setWidgetConfig(…, instance)`. */
  id: string;
  /** Text shown to the user. */
  title: string;
}

/**
 * List placed widgets. Exact on Android (`AppWidgetManager`) and iOS (`WidgetCenter`);
 * on desktop: open widget windows plus placements that rendered in the last 48 h
 * (macOS WidgetKit, Windows Widgets Board).
 */
export async function getActiveWidgets(group?: string): Promise<ActiveWidget[]> {
  return await invoke<ActiveWidget[]>(`${PLUGIN_ID}|get_active_widgets`, {
    group: group || null,
  });
}

/**
 * Values for `{{data.path}}` placeholders, without resending the layout. Pass `null` to
 * clear. Layers: `config.data` < this < the widget's own `source` fetch.
 *
 * @example
 * ```ts
 * await setWidgetConfig({ small: { type: "text", content: "{{data.temp}}°" } }, undefined, "weather");
 * await setWidgetData("weather", { temp: 21 });
 * ```
 */
export async function setWidgetData(
  widgetId: string,
  data: unknown,
  options: { instance?: string; group?: string; skipReload?: boolean } = {},
): Promise<boolean> {
  if (!widgetId) throw new Error("setWidgetData: 'widgetId' must not be empty");
  return await invoke<boolean>(`${PLUGIN_ID}|set_widget_data`, {
    widgetId,
    data: data ?? null,
    instance: options.instance || null,
    group: options.group || null,
    skipReload: options.skipReload ?? false,
  });
}

/** Where a widget's local state lives: one placement (`instance`) or all of them. */
export type WidgetStateOptions = { instance?: string; group?: string; skipReload?: boolean };

/**
 * Change a widget's local state (`{{state.…}}`, config `state` defaults) the way a `set`
 * tap does: `"$toggle"`, `"$inc"`, `"$dec"`, `null` removes a key (the default shows
 * again), other values are stored. `patch: null` clears the stored state. Resolves with
 * the merged state; the app also gets `widget-state-changed` `{widgetId, instance, state}`.
 *
 * @example
 * ```ts
 * await setWidgetState("tasks", { tab: "today", count: "$inc" });
 * ```
 */
export async function setWidgetState(
  widgetId: string,
  patch: Record<string, unknown> | null,
  options: WidgetStateOptions = {},
): Promise<Record<string, unknown>> {
  if (!widgetId) throw new Error("setWidgetState: 'widgetId' must not be empty");
  return await invoke<Record<string, unknown>>(`${PLUGIN_ID}|set_widget_state`, {
    widgetId,
    patch: patch ?? null,
    instance: options.instance || null,
    group: options.group || null,
    skipReload: options.skipReload ?? false,
  });
}

/** A widget's merged local state: config `state` defaults, then what taps and
 *  {@link setWidgetState} stored (the placement's own over the widget's). */
export async function getWidgetState(
  widgetId: string,
  options: Omit<WidgetStateOptions, "skipReload"> = {},
): Promise<Record<string, unknown>> {
  if (!widgetId) throw new Error("getWidgetState: 'widgetId' must not be empty");
  return await invoke<Record<string, unknown>>(`${PLUGIN_ID}|get_widget_state`, {
    widgetId,
    instance: options.instance || null,
    group: options.group || null,
  });
}

/**
 * Choices the user picks from per placement. iOS / macOS: the Edit Widget sheet of a
 * widget built with `TauriInstanceWidget` / `TauriInstanceProvider`. The chosen id is the
 * placement's `instance`.
 */
export async function setWidgetInstances(
  widgetId: string,
  instances: WidgetInstanceOption[],
  group?: string,
): Promise<boolean> {
  if (!widgetId) throw new Error("setWidgetInstances: 'widgetId' must not be empty");
  return await invoke<boolean>(`${PLUGIN_ID}|set_widget_instances`, {
    widgetId,
    instances,
    group: group || null,
  });
}
