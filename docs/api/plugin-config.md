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
| `appGroup` | string | App Group / shared storage group identifier |
| `transport` | `appGroup` \| `userDefaults` \| `widgetContainer` \| `auto` | Host write driver (Apple). See [Transport](/guide/transport). |
| `extensionBundleId` | string | Widget extension bundle id (macOS `widgetContainer` / tooling) |

Permissions: [Permissions](/api/permissions). Environment overrides: [Environment](/api/env).
