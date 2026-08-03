# Windows Widgets Board provider (Adaptive Cards)

Scaffold produced by `npx tauri-plugin-widgets-api init-windows`.

## What this is

- C# `IWidgetProvider` that reads `widget_data.json` written by the Tauri Rust host.
- Keys: `ac:template:{widgetId}`, `ac:data:{widgetId}`, `pending_actions`.
- Adaptive Card JSON is **transpiled in Rust** (`to_adaptive_card`, with chart/canvas/gauge rasterized to PNG data URIs).
- `PreviewHost/` — headless AC → PNG for goldens (`tests/golden/windows/`, record on Windows VM).

See also: plugin docs `docs/contributing/windows-surfaces.md` (Widgets Board vs WorkerW wallpaper).

## Before packaging

1. Generate your own COM CLSID:

```powershell
[guid]::NewGuid().ToString()
```

2. Replace `{{WIDGET_PROVIDER_CLSID}}` in Provider.cs + Package.appxmanifest.fragment.xml

3. Merge the fragment into your MSIX `Package.appxmanifest` (`com` + `uap3`). Include `Assets/StoreLogo.png`.

4. Env for the provider process:

```text
TAURI_WIDGETS_DATA=<dir containing widget_data.json>
TAURI_WIDGET_LOGICAL_ID=default
```

## Build

```powershell
# Minimal UTM (no Appx/PRI tooling):
dotnet build WidgetProvider/WidgetProvider.csproj -c Release -p:Smoke=true -p:Platform=ARM64 -p:RuntimeIdentifier=win-arm64

# Full WinAppSDK (after just win-bootstrap / VS Build Tools):
dotnet build WidgetProvider/WidgetProvider.csproj -c Release -p:Smoke=false

# PreviewHost PNG:
dotnet build PreviewHost/PreviewHost.csproj -c Release -p:Smoke=true
# Full Adaptive Cards renderer (Widgets Board parity goldens):
dotnet build PreviewHost/PreviewHost.csproj -c Release -p:Smoke=false
# Capture: just test-windows-visual / scripts/win/shot.ps1 -Mode visual
```

Pack / sideload helpers live in the plugin repo: `scripts/win/pack.ps1`, `scripts/win/sideload.ps1` (`just win-pack` / `pnpm win-pack`, same for sideload).

## Dev certificate

`Identity/@Publisher` in your MSIX manifest **must** match the cert Subject. Scaffold includes `DevCert.ps1`; regenerate after setting Publisher:

```bash
npx tauri-widgets signing --dev-cert
# → prints script + writes src-tauri/windows-widget/DevCert.ps1
```

```powershell
# elevated
powershell -ExecutionPolicy Bypass -File src-tauri/windows-widget/DevCert.ps1 -MsixPath path\to\app.msix
```
