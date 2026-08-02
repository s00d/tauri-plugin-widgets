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

## When it fails

- Call `createWidgetWindow(...)` or define a window in `tauri.conf.json`.
- Allow the window label in capabilities (`widgets:default` / window permissions).
- On Windows Widgets Board, run `init-windows` and verify `ac:template:*` after `setWidgetConfig`.
- Surfaces overview: [Windows surfaces](/contributing/windows-surfaces).

More symptoms: [Troubleshooting index](/guide/troubleshooting).
