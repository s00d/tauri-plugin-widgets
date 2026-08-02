---
title: First widget
---

# First widget

Fastest path to something on screen: a **desktop webview widget** (no Xcode / Android Studio).

<div class="doc-illust">

![Desktop webview weather widget](/illustrations/first-widget.jpg)

</div>

## 1. Install and register

Follow [Install](/guide/install) — preferably `pnpm tauri add tauri-plugin-widgets` (or `npm` / `yarn` / `bun` / `cargo tauri add`), then `tauri_plugin_widgets::init()` and `widgets:default` if needed.

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

`widgetId` is required.

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

No `url` and **no** `widget.html` copy into your app: the plugin serves its embedded renderer over the `widgetview` protocol. Details: [Desktop webview](/guide/setup/desktop).

Run `pnpm tauri dev`. You should see a frameless widget window painting the config from step 2.

## Next

- Native home-screen widgets: [Platform setup](/guide/setup/)
- Concepts (`group`, sizes, receipts): [Concepts](/guide/concepts)
- Browse presets: [Showcase](/showcase)
