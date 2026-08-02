# Windows surface split

Two independent Windows integrations. Do **not** mix them in one code path.

| Surface | Transport | UI | Package |
|---------|-----------|----|---------|
| **Widgets Board** | `widget_data.json` keys `ac:template:{id}` / `ac:data:{id}` + `pending_actions` | Adaptive Cards via C# `IWidgetProvider` | MSIX App Extension `com.microsoft.windows.widgets` |
| **Desktop wallpaper (WorkerW)** | Existing desktop webview / `widget.html` | Frameless Tauri webview HWND reparented under WorkerW | No MSIX widget extension; feature `workerw` |

## Widgets Board

- Transpile: Rust `to_adaptive_card` (+ rasterize chart/canvas/gauge → PNG data URI)
- Host write: `desktop::set_widget_config` on `cfg(windows)`
- Provider: `templates/windows-widget/WidgetProvider`
- Init: `npx tauri-plugin-widgets-api init-windows`
- Preview / goldens: `templates/windows-widget/PreviewHost` → `tests/golden/windows/` (record on Windows VM via `just record-windows <case>`)
- Adaptive Card JSON snapshots: `tests/snapshots/adaptive/` (`pnpm -C scripts cli gen-adaptive-snapshots`)

## WorkerW wallpaper

- Feature flag: `workerw` (default off)
- API: `tauri_plugin_widgets::windows::workerw::{find_workerw, attach_to_workerw}`
- Attach after creating a frameless desktop widget window; pass its HWND
- Compile-check via `cargo xwin check --features workerw`
- Runtime smoke only on an interactive Windows desktop session (`tools/win/workerw-smoke.ps1` optional)

## UTM tooling

| Command | Role |
|---------|------|
| `just win-up` | SSH healthcheck |
| `just win-bootstrap` | Install VS Build Tools / WinAppSDK via PS |
| `just win-sync` | Sync repo to `C:\work\tauri-plugin-widgets` |
| `just test-windows-remote` | `shot.ps1` smoke |
| `just test-windows-visual` / `record-windows` | PreviewHost PNG |
| `just win-pack` / `win-sideload` | Publish + sideload helpers |
| `just check-windows-xwin` | Cross-check plugin |
| `just build-windows-example-xwin` | Cross-build demo app |
