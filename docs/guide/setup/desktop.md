---
title: Desktop webview
---

# Desktop webview

Desktop hosts can show widgets as **frameless transparent webview windows** that render the JSON config via `widget.html` (built-in `widgetview` protocol).

**Windows** also supports **Widgets Board** via Adaptive Cards (see below). **Linux** pins webview windows with X11 `_NET_WM_WINDOW_TYPE_DESKTOP` when the `linux` feature is on (Wayland: optional `layer-shell`).

## Option A: Declarative (tauri.conf.json)

```json
{
  "app": {
    "windows": [
      { "label": "main", "title": "My App", "width": 800, "height": 600 },
      {
        "label": "desktop-widget",
        "url": "/widget.html",
        "width": 300, "height": 150,
        "decorations": false, "transparent": true,
        "alwaysOnBottom": true, "skipTaskbar": true,
        "visible": false, "resizable": false
      }
    ]
  }
}
```

## Option B: Dynamic (TypeScript)

```typescript
import { createWidgetWindow, closeWidgetWindow } from "tauri-plugin-widgets-api";

await createWidgetWindow({
  label: "weather",
  width: 280,
  height: 200,
  x: 80,
  y: 80,
  skipTaskbar: true,
  group: "group.com.example.myapp",
  widgetId: "weather",
  size: "small",
});

await closeWidgetWindow("weather");
```

Pass `group`, `widgetId`, and `size` so the built-in renderer loads the right config.

## When it fails

- Call `createWidgetWindow(...)` (or define a window in `tauri.conf.json`).
- Allow the window label in capabilities (`widgets:default` / window permissions).
- On Linux, enable the `linux` feature for X11 DESKTOP pinning.
- On Windows Widgets Board, run `init-windows` and verify `ac:template:*` after `setWidgetConfig`.

More symptoms: [Troubleshooting index](/guide/troubleshooting).
