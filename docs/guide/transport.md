---
title: Apple data transport
---

# Apple data transport

The host writes widget data through **one** transport you choose in config. You know your signing setup — do not rely on runtime fan-out.

<div class="doc-illust">

![Single transport channel from host app to widget surface](/illustrations/transport.jpg)

</div>

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

| Situation | `transport` |
|-----------|-------------|
| Release / Team ID + App Groups enabled | `appGroup` |
| Mac App Store | `appGroup` |
| Local ad-hoc signing (no shared App Group container) | `widgetContainer` |
| iOS (device and simulator) | `appGroup` only — other values fail at plugin init |
| Not sure yet | `auto` once at startup (dev only) — read the log, then pin the winner in conf |

| `transport` | Host write path | Requirements |
|-------------|-----------------|--------------|
| `appGroup` | `containerURL(group)/widget_data.json` | Real Team ID + App Groups on App + Extension |
| `userDefaults` | App Group `UserDefaults` suite | Same as `appGroup` |
| `widgetContainer` | `~/Library/Containers/<appex>/Data/widget_data.json` | macOS host **not** sandboxed; works with ad-hoc |
| `auto` | One-shot probe, then latch | Development only — never ship this |

Wrong `transport` / missing `appGroup` **fails plugin init** with a concrete message (empty widgets from a silent fallback are not a thing).

`init-macos` / `init-ios` do **not** write this block — add `plugins.widgets` yourself ([Install](/guide/install#apple-hosts-plugin-config)).

Override without editing conf: `WIDGET_TRANSPORT=widgetContainer`.

The **widget extension** still reads all channels and picks the freshest map (so it can find data wherever the host wrote). Host-side writes use only the configured driver. Render receipts feed `getWidgetDiagnostics`, not transport selection.

`setItems` skips disk I/O when the value is unchanged (no nonce bump).
