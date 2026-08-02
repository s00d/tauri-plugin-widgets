---
title: Windows setup
---

# Windows setup

## Windows Widgets Board (Adaptive Cards)

```bash
npx tauri-plugin-widgets-api init-windows
```

This scaffolds a C# `IWidgetProvider` under `templates/windows-widget/` (details: [Windows surfaces](/contributing/windows-surfaces)). On `setWidgetConfig` the plugin writes Adaptive Cards JSON under `ac:template:{widgetId}` / `ac:data:{widgetId}`.

Optional wallpaper parenting (not Widgets Board): enable Cargo feature `workerw`.

---

## Desktop webview fallback

Outside Widgets Board, use the same frameless webview path as other desktops: `createWidgetWindow` without `url` (built-in renderer). Do not put `"/widget.html"` in `tauri.conf.json` unless you ship your own frontend page — [Desktop webview](/guide/setup/desktop).

## When it fails

- Call `createWidgetWindow(...)` with `group` / `widgetId` / `size` for the built-in renderer.
- Allow the window label in capabilities (`widgets:default` / window permissions).
- On Windows Widgets Board, run `init-windows` and verify `ac:template:*` after `setWidgetConfig`.
- Surfaces overview: [Windows surfaces](/contributing/windows-surfaces).

More symptoms: [Troubleshooting index](/guide/troubleshooting).
