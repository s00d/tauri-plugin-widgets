# Development guide

Contributor docs for **tauri-plugin-widgets**. For app integration, start at the [Guide](/guide/) (install, first widget, platform setup).

## Architecture

The plugin is a **library**, not a widget builder. You own the extension / provider target; the plugin supplies storage, reload, and renderers.

| Component | Role |
|-----------|------|
| **Rust** (`src/`) | Storage, FFI, reload, desktop windows, Adaptive Cards, receipts |
| **Swift** (`swift/TauriWidgets`) | SwiftUI WidgetKit views / models / store |
| **Android** (`android/`) | Jetpack Glance renderer + receivers |
| **Desktop HTML** (`widget.html`) | Frameless webview renderer (embedded via `widgetview`; users do not copy it by default) |
| **Adaptive Cards** (`src/adaptive_card.rs`) | IR → Adaptive Cards 1.5 (Windows Widgets Board) |
| **Linux pin** (`src/linux/`) | X11 `_NET_WM_*` desktop hints; optional `layer-shell` |
| **Templates** (`templates/`) | iOS / macOS / Windows starters → copied into `src-tauri/*-widget/` by CLI |

**Data flow:** `setWidgetConfig(json, group, widgetId)` → platform storage (`nonce`) → reload / Glance / Widgets Board → native UI for that `widgetId`.

### Apple transport drivers

Host writes use **one** driver from `plugins.widgets.transport` (`appGroup` | `userDefaults` | `widgetContainer` | `auto`). See [Apple data transport](/guide/transport).

| Piece | Role |
|-------|------|
| [`src/config.rs`](../src/config.rs) | `WidgetsPluginConfig` / `TransportKind` |
| [`src/transport.rs`](../src/transport.rs) | `Transport` trait, `build_driver`, `probe_once` |
| [`src/macos_transport.rs`](../src/macos_transport.rs) | Concrete file / UserDefaults drivers |
| Swift `WidgetDataStore` | Extension still multi-reads + freshest pick |

Receipts are diagnostics (`getWidgetDiagnostics`), not a runtime selector. Unit tests: `cargo test -p tauri-plugin-widgets transport::`.

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

Capability table: embedded in [Core vs extended](/guide/tiers) (source file `docs/guide/_generated/capability-matrix.md` from `src/capabilities.rs`).

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
pnpm -C scripts cli gen-adaptive-snapshots
FEATURES=rasterize pnpm -C scripts cli gen-adaptive-snapshots   # chart/canvas data-URIs
```

## Linux harness

Docker X11 / Wayland webview shots and `_NET_WM_*` gates: [`linux-harness.md`](linux-harness.md).

```bash
just test-linux-x11
just shot-linux weather small
```

## Example artifacts

```bash
pnpm build:example macos   # or ios / android / …
```

## Android notes

- Rendering path is **Jetpack Glance**.
- Prefer flat `vstack` / `hstack` trees; deep `container` nesting is flaky on some launchers.
- Always set `progress.label` so hosts never show `null`.

## Documentation site

VitePress is the `@tauri-plugin-widgets/docs` workspace (`docs/`). Maintainer tooling is `@tauri-plugin-widgets/scripts` (citty + TypeScript under `scripts/src`). Consumer CLI `bin/cli.mjs` (`init-macos|ios|windows`) stays separate.

```bash
pnpm docs:generate   # gallery, elements, shots, permissions, …
pnpm docs:check      # marker drift + coverage audit
pnpm docs:audit      # coverage + schema docs asserts
pnpm docs:dev        # generate + local VitePress
pnpm docs:build      # generate + static site
pnpm -C scripts cli --help          # all maintainer commands
pnpm -C scripts typecheck
```

Root aliases (`pnpm triage`, `pnpm ios-up`, `pnpm build:example`, …) call `pnpm -C scripts cli …`. Bash stands under `tools/*.sh` are unchanged; citty only `spawn`s them.

Generated trees (`docs/public/shots/**`, showcase/elements marker bodies) are rebuilt by `pnpm docs:generate`.

## Related docs

| Doc | Topic |
|-----|--------|
| [`visual-stand.md`](visual-stand.md) | Golden pixel rules |
| [`linux-harness.md`](linux-harness.md) | Docker desktop shots |
| [`windows-surfaces.md`](windows-surfaces.md) | Widgets Board + webview |
| [`render-testing.md`](render-testing.md) | Render test overview |
| [Core vs extended](/guide/tiers) | Element × platform matrix |

