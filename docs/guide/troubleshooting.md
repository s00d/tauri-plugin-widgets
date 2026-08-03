---
title: Troubleshooting
---

# Troubleshooting

Find the symptom, then jump into the platform page (setup guides keep the detailed steps).

| Symptom | Where to look |
| --- | --- |
| App won't start on macOS/iOS (`appGroup is required`) | [Install → Apple hosts](/guide/install#apple-hosts-plugin-config) · [Plugin config](/api/plugin-config) · [Transport & signing](/guide/transport) |
| App Group “Identifier is not available” in Developer portal | [Transport → App Groups & signing](/guide/transport#app-groups--signing-plugin-consumers) — id owned by another team; pick a new group or switch Team |
| Widget shows "No configuration" (macOS) | [macOS setup → When it fails](/guide/setup/macos#when-it-fails) · [Transport](/guide/transport) · [`doctor`](/guide/doctor) |
| Widget shows "No configuration" (iOS) | [iOS setup → When it fails](/guide/setup/ios#when-it-fails) — also check Swift `widgetId` vs JS |
| Widget doesn't update (Apple) | [iOS](/guide/setup/ios#when-it-fails) / [macOS](/guide/setup/macos#when-it-fails) · reload budget in [Updating data](/guide/recipes/updating-data) · [`doctor` trace](/guide/doctor) |
| Android widget empty or crashes | [Android setup → When it fails](/guide/setup/android#when-it-fails) — JS `group` must match package / meta-data |
| Desktop window not appearing / empty | [Desktop setup → When it fails](/guide/setup/desktop#when-it-fails) — capabilities `windows` labels |
| Looking for `widget.html` in the app | You don't copy it for the default path — [Desktop webview](/guide/setup/desktop) |
| Windows Widgets Board missing template | [Windows setup](/guide/setup/windows) — path is `src-tauri/windows-widget/`, set `TAURI_WIDGETS_DATA` |
| Linux pin / layer-shell issues | [Linux setup → When it fails](/guide/setup/linux#when-it-fails) · [Linux harness](/contributing/linux-harness) |
| Empty widget after wrong Apple transport | [Transport](/guide/transport) — init fails loud; check `plugins.widgets.transport` · [`doctor`](/guide/doctor) |
| Capability degraded / unsupported | [Core vs extended](/guide/tiers) · `getWidgetDiagnostics` in [JS API](/api/js) |
| Need a one-shot health dump | [`pnpm tauri-widgets doctor`](/guide/doctor) |

Contributor tooling (goldens, harnesses): [Contributing](/contributing/).
