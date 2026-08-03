import { invoke } from "@tauri-apps/api/core";
import { PLUGIN_ID } from "./plugin";

/**
 * Store a key-value pair in the widget data store.
 *
 * The storage backend depends on the platform:
 * - **Android** — `SharedPreferences` scoped by `group`.
 * - **iOS / macOS** — JSON file in App Group shared container.
 * - **Desktop** — JSON file in the app data directory (macOS: shared container when available).
 *
 * @param key   - Setting name (e.g. `"temperature"`).
 * @param value - Setting value. Use `JSON.stringify()` for complex data.
 * @param group - Widget group identifier:
 *                Android: SharedPreferences name;
 *                iOS/macOS: App Group ID (e.g. `"group.com.example.myapp"`);
 *                Desktop: arbitrary string used as filename.
 */
export async function setItems(
  key: string,
  value: string,
  group: string,
): Promise<boolean> {
  if (!key) throw new Error("setItems: 'key' must not be empty");
  if (!group) throw new Error("setItems: 'group' must not be empty");
  return await invoke<boolean>(`${PLUGIN_ID}|set_items`, { key, value, group });
}

/**
 * Read a previously stored value from the widget data store.
 *
 * @param key   - The key to look up.
 * @param group - The widget group identifier.
 * @returns The stored string value, or `null` if the key was never set.
 */
export async function getItems(
  key: string,
  group: string,
): Promise<string | null> {
  if (!key) throw new Error("getItems: 'key' must not be empty");
  if (!group) throw new Error("getItems: 'group' must not be empty");
  return await invoke<string | null>(`${PLUGIN_ID}|get_items`, { key, group });
}
