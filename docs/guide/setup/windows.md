---
title: Windows setup
---

# Windows setup

Two surfaces: **Widgets Board** (Adaptive Cards + MSIX provider) and/or the same **desktop webview** as other OSes.

## Widgets Board (Adaptive Cards)

### 1. Scaffold into your app

```bash
npx tauri-plugin-widgets-api init-windows
```

This copies the C# `IWidgetProvider` **into your project** at `src-tauri/windows-widget/` (source templates live in the plugin under `templates/windows-widget/` — you do not edit those in `node_modules` for day-to-day work).

Generated layout:

- `WidgetProvider/` — COM provider process
- `PreviewHost/` — headless AC → PNG (goldens / debugging)
- `Package.appxmanifest.fragment.xml` — merge into your MSIX manifest
- `Assets/StoreLogo.png`
- `README.md` — same checklist as below

### 2. How data flows

On `setWidgetConfig` the Rust host writes Adaptive Cards JSON into the widget store under `ac:template:{widgetId}` / `ac:data:{widgetId}` (plus the normal `config:{widgetId}` IR). The C# provider reads that store and feeds Widgets Board.

Point the provider at the host data directory:

```text
TAURI_WIDGETS_DATA=<dir containing widget_data.json>
```

(Default location is under the app data dir / `%LOCALAPPDATA%` when unset — see host logs if unsure.)

Optional: `TAURI_WIDGET_LOGICAL_ID` (default `default`) must match the `widgetId` you pass to `setWidgetConfig`.

### 3. Package / CLSID

1. Keep or regenerate the COM CLSID (`init-windows` already writes one into `Provider.cs` + the manifest fragment).
2. Merge `Package.appxmanifest.fragment.xml` into your MSIX `Package.appxmanifest` (add `xmlns:com` and `xmlns:uap3` if missing).
3. Include `Assets/StoreLogo.png` in the package.

### 4. Build

```powershell
# Smoke build (no full Appx/PRI tooling):
dotnet build src-tauri/windows-widget/WidgetProvider/WidgetProvider.csproj -c Release -p:Smoke=true

# Full WinAppSDK provider (VS Build Tools / WinAppSDK installed):
dotnet build src-tauri/windows-widget/WidgetProvider/WidgetProvider.csproj -c Release -p:Smoke=false
```

Pack / sideload helpers used in this repo: `just win-pack`, `just win-sideload` (see [Windows surfaces](/contributing/windows-surfaces)).

Optional wallpaper parenting (not Widgets Board): Cargo feature `workerw`.

---

## Desktop webview fallback

Outside Widgets Board, use the frameless webview path: `createWidgetWindow` without `url` (built-in renderer). Do not put `"/widget.html"` in `tauri.conf.json` unless you ship your own frontend page — [Desktop webview](/guide/setup/desktop).

Allow the window label in capabilities (`windows` + `widgets:default`).

## When it fails

- Confirm scaffold path is **`src-tauri/windows-widget/`**, not the plugin’s `templates/` folder.
- Set `TAURI_WIDGETS_DATA` so the provider sees the same `widget_data.json` the Rust host writes.
- After `setWidgetConfig`, verify `ac:template:{widgetId}` exists in that store.
- Match `TAURI_WIDGET_LOGICAL_ID` / Swift–JS `widgetId` conventions if you use a non-`default` id.
- Surfaces overview: [Windows surfaces](/contributing/windows-surfaces).

More symptoms: [Troubleshooting index](/guide/troubleshooting).
