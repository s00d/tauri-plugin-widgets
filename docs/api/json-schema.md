---
title: JSON Schema
---

# JSON Schema

The widget config schema is generated from Rust models (`pnpm codegen`):

[`schemas/widget-config.v1.json`](https://github.com/s00d/tauri-plugin-widgets/blob/main/schemas/widget-config.v1.json)

Raw URL for editors:

```text
https://raw.githubusercontent.com/s00d/tauri-plugin-widgets/main/schemas/widget-config.v1.json
```

The npm package (`tauri-plugin-widgets-api`) does **not** ship `schemas/` — point tools at the GitHub raw URL or a local clone of this repo.

## IDE association

Example VS Code / Cursor `settings.json`:

```json
{
  "json.schemas": [
    {
      "fileMatch": ["**/widget*.json", "**/fixtures/**/*.json"],
      "url": "https://raw.githubusercontent.com/s00d/tauri-plugin-widgets/main/schemas/widget-config.v1.json"
    }
  ]
}
```

Element reference: [Elements](/elements/).
