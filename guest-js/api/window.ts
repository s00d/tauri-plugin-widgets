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
   * Placement id: the built-in renderer reads `config:{widgetId}#{instance}` first,
   * fills `{{instance}}`, and `getActiveWidgets` reports it. Default: the window label.
   */
  instance?: string;
  /**
   * Size family the built-in renderer should display:
   * `"small"`, `"medium"`, or `"large"`.  Defaults to `"small"`.
   */
  size?: "small" | "medium" | "large";
  /**
   * Stacking layer. Default: `"top"` when `alwaysOnTop`, else `"desktop"` on Linux and
   * `"normal"` on macOS / Windows. `"desktop"`: X11 DESKTOP type, Wayland layer-shell
   * Background, macOS desktop level, Windows behind the desktop icons.
   */
  layer?: WidgetWindowLayer;
  /** Let clicks pass through to whatever is below. Default `false`. */
  clickThrough?: boolean;
  /** Whether the window can take keyboard focus. Default `true`. */
  focusable?: boolean;
  /** Show on every virtual desktop / Space. Default: `true` for the `desktop` layer. */
  allWorkspaces?: boolean;
  /** System window shadow (macOS / Windows). */
  shadow?: boolean;
  /** Backdrop material hint (needs a transparent window; ignored on Linux). */
  effects?: "vibrancy" | "mica" | "acrylic" | "blur";
  /** Place relative to a monitor's work area instead of `x` / `y`. */
  placement?: WidgetPlacement;
  /**
   * Remember where the user moved the window (per label) and reopen it there while that
   * spot is on a connected monitor. Default `true`.
   */
  rememberPosition?: boolean;
}

/** Stacking layer of a desktop widget window. */
export type WidgetWindowLayer = "normal" | "top" | "bottom" | "desktop";

/** Monitor-relative placement for {@link WidgetWindowConfig.placement}. */
export interface WidgetPlacement {
  /** Monitor index (`availableMonitors` order) or name. Default: primary. */
  monitor?: number | string;
  /** Work-area corner / edge. Default `"top-left"`. */
  anchor?:
    | "top-left"
    | "top"
    | "top-right"
    | "left"
    | "center"
    | "right"
    | "bottom-left"
    | "bottom"
    | "bottom-right";
  /** Logical px inward from the anchored vertical edge. Default 0. */
  offsetX?: number;
  /** Logical px inward from the anchored horizontal edge. Default 0. */
  offsetY?: number;
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
