---
title: Environment variables
---

# Environment variables

| Variable | Scope | Default | Description |
|----------|-------|---------|-------------|
| `WIDGET_TRANSPORT` | Apple host | `plugins.widgets.transport` | Override: `appGroup` \| `userDefaults` \| `widgetContainer` \| `auto`. |
| `WIDGET_SIGN_IDENTITY` | macOS (`build-widget.sh`) | `APPLE_SIGNING_IDENTITY` or ad-hoc (`-`) | Signing identity for the `.appex` before bundling. |
| `WIDGET_DEBUG` | Host runtime | off in release | Set to `1` to enable the in-memory delivery journal (`getWidgetTrace`) in release builds. Debug builds always record. |
| `TAURI_WIDGET_MIN_RELOAD_SECS` | **iOS / Android** plugin host (`mobile.rs`) | Debug: `0`, Release: `900` | Minimum seconds between plugin-triggered `reloadAllTimelines()`. Use `0` to disable. **Not used on the macOS/desktop host** (`desktop.rs` has no plugin-side throttle). |
| `TAURI_WIDGETS_DATA` | Windows Widgets Board provider (C#) | host app-data dir | Directory that contains `widget_data.json` written by the Rust host. Required so the sideloaded provider sees the same store. |
| `TAURI_WIDGET_LOGICAL_ID` | Windows Widgets Board provider | `default` | Logical `widgetId` the provider reads (`ac:template:{id}` / `config:{id}`). Must match JS `setWidgetConfig`. |
| `TAURI_DEV_HOST` | Example app dev (`vite.config.ts`) | — | Dev host used by Tauri/Vite during `tauri dev` (usually set automatically). |

Examples:

```bash
# iOS / Android release: disable plugin-side reload throttle
TAURI_WIDGET_MIN_RELOAD_SECS=0 pnpm tauri ios dev

# macOS desktop host: this env does nothing for reload (no mobile throttle).
# Stale native widgets on Mac are usually transport/nonce — not this flag.

# Force widget-container transport without editing tauri.conf.json
WIDGET_TRANSPORT=widgetContainer pnpm tauri dev

# macOS build with explicit signing identity for the .appex
# (copy the string from `npx tauri-widgets signing`)
WIDGET_SIGN_IDENTITY="Apple Development: you@example.com (TEAMID)" \
  pnpm tauri build
```

---
