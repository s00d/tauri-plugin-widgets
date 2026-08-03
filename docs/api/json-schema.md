---
title: JSON Schema
---

# JSON Schema

Generated from Rust (`pnpm codegen` / `cargo run --bin gen-schema --features codegen`):

| Schema | Pages URL |
| --- | --- |
| Widget IR | https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json |
| `plugins.widgets` | https://s00d.github.io/tauri-plugin-widgets/schemas/plugin-config.v1.json |
| Capabilities matrix | https://s00d.github.io/tauri-plugin-widgets/schemas/capabilities.json |

Also in the repo under `schemas/` and shipped in the npm package (`schemas/`).

GitHub raw (fallback):

```text
https://raw.githubusercontent.com/s00d/tauri-plugin-widgets/main/schemas/widget-config.v1.json
```

## IDE association

Put `$schema` in the widget JSON (preferred):

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
  "small": { "type": "text", "content": "Hi", "color": "#fff" }
}
```

Or VS Code / Cursor `settings.json`:

```json
{
  "json.schemas": [
    {
      "fileMatch": ["**/widget*.json", "**/fixtures/**/*.json"],
      "url": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json"
    }
  ]
}
```

Validate from the CLI: `npx tauri-widgets validate ./widget.json --platforms ios,android`.

Element reference: [Elements](/elements/) · [CLI](/guide/cli).
