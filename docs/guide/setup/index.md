---
title: Platform setup
---

# Platform setup

Pick a surface. Desktop is the fastest feedback loop; native extensions take longer but ship on the home screen.

<div class="doc-illust">

![Platform surfaces: desktop, Android, iOS, macOS, Windows, Linux](/illustrations/platforms.jpg)

</div>

| Platform | Surface | Rough time | Guide |
| --- | --- | --- | --- |
| Desktop | Frameless webview | ~10 min | [Desktop](/guide/setup/desktop) · [First widget](/guide/first-widget) |
| Android | Jetpack Glance AppWidget | ~30–60 min | [Android](/guide/setup/android) |
| iOS | WidgetKit extension | ~1–2 h (Xcode) | [iOS](/guide/setup/ios) |
| macOS | WidgetKit `.appex` + `tauri build` | ~1–2 h | [macOS](/guide/setup/macos) |
| Windows | Widgets Board and/or desktop webview | ~1 h | [Windows](/guide/setup/windows) |
| Linux | Desktop webview + X11 pin | ~30 min | [Linux](/guide/setup/linux) |

**Before any Apple surface (including desktop webview on a Mac):** add `plugins.widgets.appGroup` — [Install](/guide/install#apple-hosts-plugin-config). CLI scaffolds do not write it.

Each setup page ends with **When it fails**. Symptom index: [Troubleshooting](/guide/troubleshooting).
