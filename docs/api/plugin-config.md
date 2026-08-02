---
title: Plugin config
---

# Plugin config

Configure the plugin under `plugins.widgets` in `tauri.conf.json`:

```json
{
  "plugins": {
    "widgets": {
      "appGroup": "group.com.example.myapp",
      "transport": "appGroup",
      "extensionBundleId": "com.example.myapp.widgetkit"
    }
  }
}
```

| Field | Type | Description |
| --- | --- | --- |
| `appGroup` | string | **Required on macOS and iOS.** App Group / shared storage group identifier. Missing → plugin init error. |
| `transport` | `appGroup` \| `userDefaults` \| `widgetContainer` \| `auto` | Host write driver (Apple). iOS: `appGroup` only. See [Transport](/guide/transport). |
| `extensionBundleId` | string | Widget extension bundle id (macOS `widgetContainer` / tooling) |

`init-macos` / `init-ios` do not invent this block for you — add it under `plugins.widgets` in `tauri.conf.json`. Linux / Windows desktop webview can omit it.

Permissions: [Permissions](/api/permissions). Environment overrides: [Environment](/api/env).
