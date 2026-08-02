---
title: First widget
---

# First widget

Fastest path to something on screen: a **desktop webview widget** (no Xcode Widget Extension / Android Studio).

<div class="doc-illust">

![Desktop webview weather widget](/illustrations/first-widget.jpg)

</div>

## 1. Install and register

Follow [Install](/guide/install) — preferably `pnpm tauri add tauri-plugin-widgets` (or `npm` / `yarn` / `bun` / `cargo tauri add`), then `tauri_plugin_widgets::init()` and capabilities.

### macOS host caveat

Even for a **desktop-only** webview (no WidgetKit), macOS still resolves Apple transport at plugin init. Add `plugins.widgets` **before** `tauri dev` or the app will not start:

```json
{
  "plugins": {
    "widgets": {
      "appGroup": "group.com.example.myapp",
      "transport": "widgetContainer"
    }
  }
}
```

Use `widgetContainer` for local ad-hoc signing; switch to `appGroup` once Team ID + App Groups work. Linux / Windows hosts can skip this block. See [Transport](/guide/transport).

### Capabilities

Allow the widget window label (here `"weather"`):

```json
{
  "identifier": "default",
  "windows": ["main", "weather"],
  "permissions": ["core:default", "widgets:default"]
}
```

## 2. Push a config

```typescript
import { setWidgetConfig, createWidgetWindow } from "tauri-plugin-widgets-api";

const group = "group.com.example.myapp";
const widgetId = "weather";

await setWidgetConfig(
  {
    small: {
      type: "vstack",
      padding: 14,
      background: { light: "#E8F4FD", dark: "#1a1a2e" },
      spacing: 6,
      cornerRadius: 16,
      children: [
        {
          type: "hstack",
          spacing: 8,
          children: [
            { type: "image", systemName: "cloud.sun.fill", color: "#ffcc00", size: 28 },
            {
              type: "text",
              content: "72°",
              textStyle: "largeTitle",
              fontWeight: "bold",
              color: "label",
            },
          ],
        },
        { type: "text", content: "Partly Cloudy", textStyle: "footnote", color: "secondaryLabel" },
        { type: "progress", value: 0.65, tint: "#4CAF50", label: "Humidity" },
      ],
    },
  },
  group,
  widgetId,
);
```

`widgetId` is required. On **Android**, `group` must match the SharedPreferences name the Glance receiver reads (default: app package name) — see [Android setup](/guide/setup/android). This desktop walkthrough can keep any stable string.

## 3. Open a desktop window

```typescript
await createWidgetWindow({
  label: "weather",
  width: 280,
  height: 160,
  x: 80,
  y: 80,
  skipTaskbar: true,
  group,
  widgetId,
  size: "small",
});
```

- No `url` and **no** `widget.html` copy: the plugin serves its embedded renderer over the `widgetview` protocol ([Desktop webview](/guide/setup/desktop)).
- `group` and `widgetId` are required for that path; `size` defaults to `"small"` if omitted.
- On macOS, transparent webviews need Cargo feature `macos-private-api` and `app.macOSPrivateApi` / `tauri.conf` private API — otherwise the window still opens, but without transparency ([Cargo features](/api/cargo-features)).

Run `pnpm tauri dev`. You should see a frameless widget window painting the config from step 2.

## Next

- Native home-screen widgets: [Platform setup](/guide/setup/)
- Concepts (`group`, sizes, receipts): [Concepts](/guide/concepts)
- Browse presets: [Showcase](/showcase)
