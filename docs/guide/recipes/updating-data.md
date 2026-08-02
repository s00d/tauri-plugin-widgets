---
title: Widget updater
---

# Widget updater

For periodic updates (clocks, dashboards):

<div class="doc-illust">

![Widget state updating over time](/illustrations/updating.jpg)

</div>

```typescript
import { startWidgetUpdater } from "tauri-plugin-widgets-api";

const stop = await startWidgetUpdater(
  () => ({
    small: {
      type: "vstack", padding: 12, background: "#1a1a2e",
      children: [
        { type: "text", content: new Date().toLocaleTimeString(),
          fontSize: 32, fontWeight: "bold", color: "#fff" },
      ],
    },
  }),
  "group.com.example.myapp",
  "clock",
  {
    intervalMs: 60_000,
    reload: true,
    onAction: (action, payload) => {
      console.log("Widget action:", action, payload);
    },
  },
);

// Later:
stop();
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `intervalMs` | `number` | `5000` | Update interval (ms). Use `60000+` for native WidgetKit hosts. |
| `immediate` | `boolean` | `true` | Run builder immediately on start |
| `reload` | `boolean` | `false` | Call `reloadAllTimelines()` after each tick. On **iOS/Android** throttled by `TAURI_WIDGET_MIN_RELOAD_SECS` (not applied on the macOS desktop host). |
| `onAction` | `function` | — | Subscribe to `widget-action` events |

> **Important:** Apple enforces a daily widget reload budget (~40-70 reloads/day). On **iOS/Android** the Rust mobile host also throttles via `TAURI_WIDGET_MIN_RELOAD_SECS`. The **macOS desktop host does not** use that throttle — check `setWidgetConfig` → `ApplyOutcome.reload` instead.
>
> Defaults (mobile only):
> - **Debug:** `0` (no plugin-side throttle)
> - **Release:** `900` (15 minutes)
>
> Examples:
> - Disable plugin-side throttle (iOS/Android): `TAURI_WIDGET_MIN_RELOAD_SECS=0`
> - Set custom throttle: `TAURI_WIDGET_MIN_RELOAD_SECS=5`
>
> Even with `0`, WidgetKit may still coalesce/defer refreshes — this is a platform-level limit. For second-by-second UI, prefer native timer/date styles (for example, `{ type: "timer" }`) instead of frequent reload calls.
>
> `setWidgetConfig` returns [`ApplyOutcome`](/api/js) (`written`, `reload`, `skip`) — never treat a bare `true` as “reload happened”.

---
