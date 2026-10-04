---
title: JavaScript API
---

# JavaScript API

Package: `tauri-plugin-widgets-api`. Types for the widget IR are generated from Rust (`guest-js/generated/widget-types.ts`) — do not hand-edit them.

<div class="doc-illust">

![Desktop widget driven from the JavaScript API](/illustrations/first-widget.jpg)

</div>

```ts
import {
  setWidgetConfig,
  setWidgetConfigFromPath,
  createWidgetWindow,
  onWidgetAction,
  startWidgetUpdater,
} from "tauri-plugin-widgets-api";
```

## Config and storage

| Function | Purpose |
| --- | --- |
| `setWidgetConfig(config, group, widgetId, skipReload?)` | Push a full `WidgetConfig`. Returns `ApplyOutcome` (`written`, `reload`, `transports`, optional `skip`) — not a bare boolean |
| `setWidgetConfigFromPath(path, group, widgetId, skipReload?)` | Read a `WidgetConfig` JSON file from disk, then push it like `setWidgetConfig`. Path must be readable by the app process (absolute or resolved `resourceDir()` / app-data). No network URLs. Needs `widgets:allow-set-widget-config-from-path` (included in `widgets:default`) |
| `getWidgetTrace(group, { since? })` | Host delivery journal + receipt history (debug / `WIDGET_DEBUG=1`) |
| `flushWidgetTrace()` | Flush journal to disk (desktop) |
| `getWidgetConfig(group, widgetId)` | Read the stored config |
| `setItems(key, value, group)` | Low-level key/value write shared with native widgets |
| `getItems(key, group)` | Low-level key/value read |

```ts
import { resourceDir, join } from "@tauri-apps/api/path";
import { setWidgetConfigFromPath } from "tauri-plugin-widgets-api";

const path = await join(await resourceDir(), "widgets", "weather.json");
await setWidgetConfigFromPath(path, "group.com.example.myapp", "weather");
```

## Timelines and Android pin

| Function | Purpose |
| --- | --- |
| `reloadAllTimelines()` | Ask the host to reload widget timelines |
| `reloadTimelines(ofKind)` | Reload a specific widget kind |
| `setRegisterWidget(widgets)` | Register provider class names (Android) |
| `requestWidget()` | Prompt the user to pin a widget (Android) |

## Desktop windows

Omit `url` to use the **embedded** renderer (`widget.html` inside the crate, served over `widgetview`). You do not copy that file into the app. Optional custom `url` and declarative-config caveats: [Desktop webview](/guide/setup/desktop).

| Function | Purpose |
| --- | --- |
| `createWidgetWindow(config)` | Frameless transparent window; pass `group`, `widgetId`, `size` for the built-in renderer |
| `closeWidgetWindow(label)` | Close a window created earlier |

```ts
await createWidgetWindow({
  label: "weather",
  width: 280,
  height: 160,
  group: "group.com.example.myapp",
  widgetId: "weather",
  size: "small",
});
```

## Actions

| Function | Purpose |
| --- | --- |
| `widgetAction(action, payload?, opts?)` | Emit a `widget-action` from the host |
| `onWidgetAction(callback)` | Subscribe to taps from `button` / `link` / list rows |
| `pollPendingWidgetActions(group)` | Drain native-queued actions (Android fallback) |

Action payload: `{ action, payload?, ts, widgetId, group }`.

## Diagnostics

| Function | Purpose |
| --- | --- |
| `reportReceipt(receipt)` | Native renderers report what they drew / skipped |
| `getWidgetDiagnostics(group)` | Inspect latest receipts for a group |

## Updater

| Function | Purpose |
| --- | --- |
| `startWidgetUpdater(builder, group, widgetId, options?)` | Periodically rebuild and push config; returns a `stop` function |

See [Updating data](/guide/recipes/updating-data) for WidgetKit reload budget notes.

## Rust

Host-side API: [Rust API](/api/rust) · [docs.rs](https://docs.rs/tauri-plugin-widgets/).
