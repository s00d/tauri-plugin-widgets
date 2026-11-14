import { addPluginListener, invoke, type PluginListener } from "@tauri-apps/api/core";
import { PLUGIN_ID } from "./plugin";
import type { WidgetElement } from "../generated/widget-types";

/** A button of a live activity; taps arrive as `widget-action` with `widgetId: "activity.<id>"`. */
export interface ActivityAction {
  id: string;
  label: string;
  /** Icon (SF Symbol; Material / emoji elsewhere). */
  systemName?: string;
  /** Open this deep link instead of queuing a `widget-action`. */
  url?: string;
}

/**
 * A live activity: iOS Live Activity (Lock Screen, Dynamic Island), Android ongoing
 * notification (Live Update chip on Android 16), desktop always-on-top widget window.
 * Without `layout` one is built from `title` / `body` / `progress` / `endsAt` / `actions`.
 * The serialized request — including the layout the plugin builds when `layout` is
 * omitted — must stay under 4 KiB (the ActivityKit limit, checked on every OS).
 */
export interface ActivityContent {
  /** Your key (letters, digits, `.`, `_`, `-`). */
  id: string;
  /** Required unless the `template` gives it. */
  title?: string;
  body?: string;
  /** `0..1`. */
  progress?: number;
  /** Unix ms the countdown runs to. */
  endsAt?: number;
  /** SF Symbol name (Material / emoji off Apple). */
  icon?: string;
  /** Hex accent color. */
  tint?: string;
  layout?: WidgetElement;
  compactLeading?: WidgetElement;
  compactTrailing?: WidgetElement;
  minimal?: WidgetElement;
  /** iOS Lock Screen layout when it should differ from `layout`. */
  lockScreen?: WidgetElement;
  /** Expanded Dynamic Island regions. */
  islandLeading?: WidgetElement;
  islandTrailing?: WidgetElement;
  islandCenter?: WidgetElement;
  /** Hex colors: Dynamic Island keyline, Lock Screen background. */
  keylineTint?: string;
  backgroundTint?: string;
  /** Android: notification sub text; Android 16 status-bar chip text (≤ 7 chars). */
  subText?: string;
  shortCriticalText?: string;
  /** Unix ms: a stopwatch counts up from here when `endsAt` is unset. */
  startedAt?: number;
  /** Android lock-screen visibility. */
  visibility?: "public" | "private" | "secret";
  /** Android notification category. */
  category?: "progress" | "navigation" | "transport" | "workout" | "status" | "alarm" | "service" | "event";
  /** At most 3. */
  actions?: ActivityAction[];
  /** Deep link opened on tap. */
  url?: string;
  /** Unix ms after which the content counts as outdated (iOS). */
  staleAt?: number;
  /** iOS: ask for an APNs token (`pushToken`, `activity-token` event). */
  push?: boolean;
  /** Name given to {@link registerActivityTemplate}: the device fills that layout with
   *  `data`, so the request (and its 4 KiB budget) carries only data. Fields set here
   *  override the template's. */
  template?: string;
  /** Values for `{{data.…}}` in the `template`. */
  data?: Record<string, unknown>;
  /** iOS: which of several activities the system shows first (higher wins). */
  relevanceScore?: number;
  /** iOS: alert shown with this update (`sound`: a file in the app bundle). */
  alert?: { title: string; body: string; sound?: string };
  /** Unix ms the content was produced; an update older than the shown one is dropped. */
  timestamp?: number;
}

/**
 * Store a live activity layout on the device (shared store / App Group) under `name`.
 * `layout` takes the `ActivityContent` layout fields (`layout`, `lockScreen`,
 * `compactLeading`, `islandCenter`, `title`, …) with `{{data.…}}`, `when`, `repeat`;
 * activities started with `template: name` then send only `data`. `null` removes it.
 *
 * @example
 * ```ts
 * await registerActivityTemplate("ride", {
 *   title: "Driver {{data.driver}}",
 *   lockScreen: { type: "text", content: "Arrives in {{data.eta}} min" },
 * });
 * await startActivity({ id: "ride-42", template: "ride", data: { driver: "Ann", eta: 4 } });
 * ```
 */
export async function registerActivityTemplate(
  name: string,
  layout: Partial<Omit<ActivityContent, "id" | "template" | "data">> | Record<string, unknown> | null,
  group?: string,
): Promise<boolean> {
  if (!name) throw new Error("registerActivityTemplate: 'name' must not be empty");
  return await invoke<boolean>(`${PLUGIN_ID}|register_activity_template`, {
    name,
    layout: layout ?? null,
    group: group || null,
  });
}

export interface ActivityHandle {
  id: string;
  /** `ios` | `android` | `desktop`. */
  surface: string;
  pushToken?: string;
}

/** Start a live activity, or update it when `id` already runs. */
export async function startActivity(content: ActivityContent, group?: string): Promise<ActivityHandle> {
  return await invoke<ActivityHandle>(`${PLUGIN_ID}|start_activity`, { content, group: group || null });
}

/** Replace the content of a running activity. */
export async function updateActivity(content: ActivityContent, group?: string): Promise<boolean> {
  return await invoke<boolean>(`${PLUGIN_ID}|update_activity`, { content, group: group || null });
}

/** Options of {@link endActivity}. */
export interface EndActivityOptions {
  /** Final content shown while the activity is dismissed. */
  content?: ActivityContent;
  /** `immediate` removes it at once; `default` lets the OS keep it a while (iOS: up to 4 h).
   *  `dismissAfterMs` wins over it. */
  dismissalPolicy?: "default" | "immediate";
  group?: string;
}

/** End an activity; `dismissAfterMs` keeps the last state visible that long (iOS, desktop). */
export async function endActivity(
  id: string,
  dismissAfterMs?: number,
  options: EndActivityOptions = {},
): Promise<boolean> {
  return await invoke<boolean>(`${PLUGIN_ID}|end_activity`, {
    id,
    dismissAfterMs: dismissAfterMs ?? null,
    content: options.content ?? null,
    dismissalPolicy: options.dismissalPolicy ?? null,
    group: options.group || null,
  });
}

/** Lifecycle of an activity reported by the OS. */
export type ActivityStateEvent = { id: string; state: "active" | "ended" | "dismissed" | "stale" | string };
/** Content the OS shows now (for example after a push update). */
export type ActivityContentEvent = { id: string; content: ActivityContent };

/** Activity state changes from the native side (`activity-state`; no-op where none). */
export async function onActivityState(cb: (e: ActivityStateEvent) => void): Promise<() => void> {
  return await nativeListener("activity-state", cb);
}

/** Activity content changes from the native side (`activity-content`, e.g. push updates). */
export async function onActivityContent(cb: (e: ActivityContentEvent) => void): Promise<() => void> {
  return await nativeListener("activity-content", cb);
}

async function nativeListener<T>(event: string, cb: (e: T) => void): Promise<() => void> {
  try {
    const l = await addPluginListener<T>("widgets", event, cb);
    return () => void l.unregister();
  } catch {
    return () => {}; // no native plugin events on this platform
  }
}

/** WidgetKit push token (hex) for server-driven widget reloads; `null` where none exists. */
export async function getWidgetPushToken(group?: string): Promise<string | null> {
  return await invoke<string | null>(`${PLUGIN_ID}|get_widget_push_token`, { group: group || null });
}

/**
 * iOS 17.2+ push-to-start token (hex): your server can start a Live Activity with it
 * (APNs `event: "start"`, `attributes-type: "TauriActivityAttributes"`,
 * `attributes: {id, group}`, `content-state: {json: "<ActivityContent JSON>"}`).
 * `null` on other platforms or before iOS issued one; changes arrive via
 * {@link onActivityTokens}.
 */
export async function getActivityPushToStartToken(): Promise<string | null> {
  return await invoke<string | null>(`${PLUGIN_ID}|get_activity_push_to_start_token`);
}

/**
 * Android 16+: whether activity notifications may show as Live Updates (status-bar chip,
 * lock screen). `true` where no such gate exists (iOS, desktop, older Android).
 */
export async function canPromoteActivities(): Promise<boolean> {
  return await invoke<boolean>(`${PLUGIN_ID}|can_promote_activities`);
}

/** Android: open the system screen that allows Live Updates for this app; `false` if none. */
export async function openActivityPromotionSettings(): Promise<boolean> {
  return await invoke<boolean>(`${PLUGIN_ID}|open_activity_promotion_settings`);
}

/** Token events from iOS: per-activity APNs tokens and push-to-start token updates. */
export type ActivityTokenEvent =
  | { kind: "activity"; id: string; pushToken: string }
  | { kind: "pushToStart"; pushToStartToken: string };

/** Subscribe to Live Activity token changes (iOS only; a no-op elsewhere). */
export async function onActivityTokens(cb: (e: ActivityTokenEvent) => void): Promise<() => void> {
  const listeners: PluginListener[] = [];
  try {
    listeners.push(
      await addPluginListener<{ id: string; pushToken: string }>("widgets", "activity-token", (p) =>
        cb({ kind: "activity", id: p.id, pushToken: p.pushToken }),
      ),
    );
    listeners.push(
      await addPluginListener<{ pushToStartToken: string }>("widgets", "activity-push-to-start-token", (p) =>
        cb({ kind: "pushToStart", pushToStartToken: p.pushToStartToken }),
      ),
    );
  } catch {
    // no native plugin events on this platform
  }
  return () => listeners.forEach((l) => void l.unregister());
}

/** Ids of running activities this app started. */
export async function listActivities(): Promise<string[]> {
  return await invoke<string[]>(`${PLUGIN_ID}|list_activities`);
}
