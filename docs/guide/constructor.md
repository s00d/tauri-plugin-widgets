---
title: Widget constructor
aside: false
outline: false
pageClass: widget-constructor
---

# Widget constructor

Build a `WidgetConfig` visually: select a node in the tree, click a type in the palette to insert there, edit props from the JSON Schema, preview with the desktop renderer, then copy or download the JSON for your app.

Live preview is the HTML sandbox (desktop renderer) — not a native WidgetKit / Glance preview.

<WidgetConstructor />

## In your app

1. Export JSON from the constructor (Copy / Download).
2. Ship the file with the app (for example under `resources/widgets/`).
3. Load it at runtime:

```ts
import { resourceDir, join } from "@tauri-apps/api/path";
import { setWidgetConfigFromPath } from "tauri-plugin-widgets-api";

const path = await join(await resourceDir(), "widgets", "weather.json");
await setWidgetConfigFromPath(path, "group.com.example.myapp", "weather");
```

`path` must be readable by the app process (absolute path or a resolved resource / app-data path). Network URLs are not supported. Capability: `widgets:allow-set-widget-config-from-path` (included in `widgets:default`). Strict schema checks stay with the CLI `validate` command.

See also: [JavaScript API](/api/js), [Rust API](/api/rust), [JSON Schema](/api/json-schema).
