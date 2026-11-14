---
name: tauri-widgets
description: Build, review, configure or debug native widgets with tauri-plugin-widgets (Tauri 2) — widget JSON layouts, setWidgetConfig / setWidgetData, scheduled states, localized strings, widget-fetched data, per-placement instances, live activities (iOS Live Activity, Android ongoing notification, desktop window), widget actions, platform setup (iOS/macOS WidgetKit, Android Glance, Windows Widgets Board, desktop webview) and the tauri-widgets CLI.
metadata:
  author: s00d
  version: 1.0.0
---

# tauri-plugin-widgets

One JSON layout (`WidgetConfig`) drives WidgetKit (iOS/macOS), Glance (Android), the
Windows Widgets Board (Adaptive Cards) and desktop webview windows. Load only the
reference the task needs.

## Ground truth

- Never write SwiftUI / Glance / XAML UI for widget content. Build `WidgetConfig` JSON and
  push it with `setWidgetConfig(config, group?, widgetId)` from `tauri-plugin-widgets-api`,
  or `app.widget().set_widget_config(...)` in Rust.
- Native code is only the generated extension shell: `npx tauri-widgets init` (iOS, macOS,
  Windows) writes it. Edit `MyWidget.swift` only to add widgets to the bundle
  (`TauriLiveActivity()`, an instance widget), never to draw content.
- `widgetId` on the JS side must match the native widget's `widgetId` (default `"default"`).
- Before using an element or field, check `references/support.md`: some cells are
  `degraded` / `unsupported` per platform. Prefer the full set for the platforms shipped.
- Validate layouts with `npx tauri-widgets validate ./widget.json --platforms ios,android`
  and preview with `npx tauri-widgets preview ./widget.json --watch` (no native build).
- Things that change while the app is closed belong in the config (`entries`, `strings`,
  `source`), not in a JS timer. `startWidgetUpdater` only runs while the app runs.
- Placeholders `{{…}}` fill strings only (text, labels, URLs). Numeric fields stay literal.
- Live activity requests must stay under 4 KiB of JSON on every platform.
- Register the plugin (`tauri_plugin_widgets::init()`) and `widgets:default` in capabilities.

## Reference routing

- Layout JSON, elements, style fields: `references/layout.md`
- Platform support per element / feature: `references/support.md`
- Scheduled states, localized strings, `setWidgetData`, widget-fetched `source`: `references/dynamic.md`
- Same widget placed twice, `getActiveWidgets`, Apple Edit Widget picker: `references/instances.md`
- Live activities (Lock Screen, Dynamic Island, Android ongoing notification, desktop): `references/live-activities.md`
- Buttons, links, `onWidgetAction`, cold-start taps: `references/actions.md`
- Install, `init-*`, App Groups, signing, transport, `doctor`, `trace`: `references/setup.md`
- Hosted docs for anything deeper: https://s00d.github.io/tauri-plugin-widgets/
