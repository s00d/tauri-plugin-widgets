---
title: tauri-plugin-widgets
description: Native widgets for Android, iOS, macOS, Windows, and Linux from one declarative JSON config.
navigation: false
---

```ts [main.ts]
import { setWidgetConfig, createWidgetWindow } from "tauri-plugin-widgets-api";

await setWidgetConfig({
  small: {
    type: "vstack", padding: 12, background: "#1a1a2e",
    children: [
      { type: "text", content: "72°", fontSize: 36, fontWeight: "bold", color: "#fff" },
      { type: "progress", value: 0.7, tint: "#4CAF50" },
    ],
  },
}, "group.com.example.myapp", "weather");

// Desktop: a frameless window with the built-in renderer.
await createWidgetWindow({ label: "weather", group: "group.com.example.myapp", widgetId: "weather", size: "small" });
```
