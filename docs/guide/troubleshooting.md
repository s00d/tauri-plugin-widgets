---
title: Troubleshooting
---

# Troubleshooting

Find the symptom, then jump into the platform page (setup guides keep the detailed steps).

| Symptom | Where to look |
| --- | --- |
| Widget shows "No configuration" (macOS) | [macOS setup → When it fails](/guide/setup/macos#when-it-fails) · [Transport](/guide/transport) |
| Widget shows "No configuration" (iOS) | [iOS setup → When it fails](/guide/setup/ios#when-it-fails) |
| Widget doesn't update (Apple) | [iOS](/guide/setup/ios#when-it-fails) / [macOS](/guide/setup/macos#when-it-fails) · reload budget in [Updating data](/guide/recipes/updating-data) |
| Android widget empty or crashes | [Android setup → When it fails](/guide/setup/android#when-it-fails) |
| Desktop window not appearing | [Desktop setup → When it fails](/guide/setup/desktop#when-it-fails) |
| Looking for `widget.html` in the app | You don't copy it for the default path — [Desktop webview](/guide/setup/desktop) |
| Windows Widgets Board missing template | [Windows setup → When it fails](/guide/setup/windows#when-it-fails) |
| Linux pin / layer-shell issues | [Linux setup → When it fails](/guide/setup/linux#when-it-fails) · [Linux harness](/contributing/linux-harness) |
| Empty widget after wrong Apple transport | [Transport](/guide/transport) — init fails loud; check `plugins.widgets.transport` |
| Capability degraded / unsupported | [Core vs extended](/guide/tiers) · `getWidgetDiagnostics` in [JS API](/api/js) |

Contributor tooling (goldens, harnesses): [Contributing](/contributing/).
