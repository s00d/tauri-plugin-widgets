---
title: JSON Schema
---

# JSON Schema

The widget config schema is generated from Rust models (`pnpm codegen`):

[`schemas/widget-config.v1.json`](https://github.com/s00d/tauri-plugin-widgets/blob/main/schemas/widget-config.v1.json)

## IDE association

Point your editor at the schema for `WidgetConfig` JSON files, for example VS Code:

```json
{
  "json.schemas": [
    {
      "fileMatch": ["**/widget*.json", "**/fixtures/**/*.json"],
      "url": "./node_modules/tauri-plugin-widgets-api/schemas/widget-config.v1.json"
    }
  ]
}
```

(If the schema is not published inside the npm package path in your version, use the GitHub raw URL or a local checkout.)

Element reference: [Elements](/elements/).
