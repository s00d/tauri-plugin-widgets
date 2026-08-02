# Changelog

## 0.4.3

### Fixes
- docs.rs build: stop passing `--cfg docsrs` via `rustc-args` (it infected dependencies and broke `winnow` on current nightly); detect docs.rs via `DOCS_RS` in `build.rs`

## 0.4.1

### Breaking (Apple host transport)
- macOS host no longer fan-outs writes across three channels at runtime
- Choose one driver in `plugins.widgets.transport`: `appGroup` | `userDefaults` | `widgetContainer` | `auto`
- Require `plugins.widgets.appGroup`; wrong/missing config fails plugin init (loud)
- Override with `WIDGET_TRANSPORT`; iOS accepts `appGroup` only
- Unchanged `setItems` values skip disk write / nonce bump

### Cleanup
- Removed Mac-side SVG Adaptive Cards compositor (`gen-windows-goldens`) and fake `tests/golden/windows/*.png`; keep `tests/snapshots/adaptive/` + PreviewHost for real WinUI3 pixels
- `rasterize` no longer in default features (opt-in resvg)
- Dropped empty Cargo features `windows` / `macos` / `ios` / `android` / `all-platforms` — backends follow `target_os`
- `gtk` is optional under feature `linux`
- Removed dead Kotlin helpers (`sparkline`, `countRenderableChildren`, `maxRenderableItems`)

## 0.4.0

First public cut of the redesign (unpublished drafts were never released separately).

### Breaking (data / identity / actions)
- Required `widgetId` on `setWidgetConfig`, `getWidgetConfig`, `startWidgetUpdater`, and built-in `createWidgetWindow`
- Storage keys: `config:{widgetId}`, `pending_actions`, `__meta_nonce__`, `__meta_updated_at__`
- Action envelope: `{ action, payload?, ts, widgetId, group }`
- iOS rejects groups that do not start with `group.`
- `Option` IR fields omit `null` on serialize (`skip_serializing_if`)
- `startWidgetUpdater` default `intervalMs` is **5000**
- No public Rust fluent `builder` module — author via JS / serde JSON
- Removed unused `WidgetItem` type

### IR SoT / DX
- Rust `models.rs` is IR source of truth; TS types from `IR_SPEC` emitter (`pnpm codegen`)
- Capability matrix (Core vs Extended) + `validate_config` warnings on config change
- JSON Schema `schemas/widget-config.v1.json`; golden fixtures + core layout snapshots

### Transport / platforms
- Config hash per `(group, widgetId)`; Apple multi-transport fan-out + freshest-by-nonce read
- iOS `pollPendingActions`; mobile drains pending actions on `Ready`/`Resumed`; updater also polls
- Android: live `setWidgetConfig`, image preprocess, `appWidgetId → widgetId` mapping, targeted Glance sync
- Desktop `poll_pending_actions`; transparent widget windows (macOS needs `macos-private-api`)

### Render parity (selected)
- Android: spacer flex, canvas density, gauge aspect, bar baseline, zstack alignment, capsule `2s×s`, textStyle>fontSize, list caps
- Swift/HTML: vertical divider in hstack, text alignment, frame-before-background, zstack alignment, timer cleanup

### Tooling
- CI: `cargo test --lib`, android/ios `cargo check --lib`, codegen drift, `pnpm build`
- Dropped library `[profile.release]` override; `android/.gradle` / `local.properties` gitignored

### Known / deferred
- macOS UserDefaults transport still write-mostly; `{app_id}.widgetkit` own-container hardcode; 500ms macOS action poller redesign
