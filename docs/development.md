# Development guide

Contributor docs for **tauri-plugin-widgets**. For app integration, see the root [README](../README.md).

## Architecture

The plugin is a **library**, not a widget builder. You own the extension / provider target; the plugin supplies storage, reload, and renderers.

| Component | Role |
|-----------|------|
| **Rust** (`src/`) | Storage, FFI, reload, desktop windows, Adaptive Cards, receipts |
| **Swift** (`swift/TauriWidgets`) | SwiftUI WidgetKit views / models / store |
| **Android** (`android/`) | Jetpack Glance renderer + receivers |
| **Desktop HTML** (`widget.html`) | Frameless webview renderer |
| **Adaptive Cards** (`src/adaptive_card.rs`) | IR → Adaptive Cards 1.5 (Windows Widgets Board) |
| **Linux pin** (`src/linux/`) | X11 `_NET_WM_*` desktop hints; optional `layer-shell` |
| **Templates** (`templates/`) | iOS / macOS / Windows starters |

**Data flow:** `setWidgetConfig(json, group, widgetId)` → platform storage (`nonce`) → reload / Glance / Widgets Board → native UI for that `widgetId`.

On Apple the host **fan-outs** writes (App Group file, UserDefaults suite, macOS sandbox file); the extension picks the freshest map by `nonce` / `updatedAt`. Render **receipts** narrow fan-out after paint.

Storage keys (0.4+): `config:{widgetId}`, `pending_actions`, `__meta_nonce__`, `__meta_updated_at__`. Windows also `ac:template:{widgetId}` / `ac:data:{widgetId}`.

## Project layout

```
├── android/                    Jetpack Glance plugin
├── ios/                        iOS Tauri bridge
├── macos/                      macOS FFI (reload + container path)
├── swift/                      TauriWidgets Swift package
├── src/                        Rust core
├── guest-js/                   TypeScript API + generated IR types
├── schemas/widget-config.v1.json
├── docs/                       Capability matrix, harnesses, visual stand
├── tests/fixtures/             WidgetConfig JSON
├── tests/golden/{android,ios,macos,desktop,windows,linux}/
├── tests/snapshots/adaptive/   Adaptive Cards JSON snapshots
├── templates/{ios,macos,windows}-widget/
├── widget.html                 Desktop HTML renderer
└── examples/
```

## Codegen

After changing Rust IR (`src/models.rs` / capabilities):

```bash
pnpm codegen
# or: cargo run --bin gen-ts --features codegen
```

Capability table: [`capability-matrix.md`](capability-matrix.md) (generated from `src/capabilities.rs`).

## Visual / golden tests

Rules and runners: [`visual-stand.md`](visual-stand.md).

```bash
just test-macos-visual
just record-macos weather.small          # CASE + GOLDEN_RECORD=1
# iOS ImageRenderer:
CASE=weather.small GOLDEN_RECORD=1 just record-ios weather.small

pnpm audit:sheets                        # → out/audit/index.html
```

Windows PNG goldens: record on a Windows VM (`just record-windows <case>`), not a Mac SVG compositor. See [`windows-surfaces.md`](windows-surfaces.md).

Adaptive Cards JSON snapshots:

```bash
bash tools/gen-adaptive-snapshots.sh
FEATURES=rasterize bash tools/gen-adaptive-snapshots.sh   # chart/canvas data-URIs
```

## Linux harness

Docker X11 / Wayland webview shots and `_NET_WM_*` gates: [`linux-harness.md`](linux-harness.md).

```bash
just test-linux-x11
just shot-linux weather small
```

## Example artifacts

```bash
bash tools/build-example-artifacts.sh macos   # or ios / android / …
```

## Android notes

- Rendering path is **Jetpack Glance**.
- Prefer flat `vstack` / `hstack` trees; deep `container` nesting is flaky on some launchers.
- Always set `progress.label` so hosts never show `null`.

## Related docs

| Doc | Topic |
|-----|--------|
| [`visual-stand.md`](visual-stand.md) | Golden pixel rules |
| [`linux-harness.md`](linux-harness.md) | Docker desktop shots |
| [`windows-surfaces.md`](windows-surfaces.md) | Widgets Board + webview |
| [`render-testing.md`](render-testing.md) | Render test overview |
| [`capability-matrix.md`](capability-matrix.md) | Element × platform |
| [`visual-audit-notes.md`](visual-audit-notes.md) | Audit notes |
