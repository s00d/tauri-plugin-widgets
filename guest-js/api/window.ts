import { invoke } from "@tauri-apps/api/core";
import { PLUGIN_ID } from "./plugin";

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
