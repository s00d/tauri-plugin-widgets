# Contract v3 (shared spec for parallel work)

`CONTRACT_VERSION` becomes **3** in every port (Rust `src/resolve.rs`, TS
`widget-src/resolve.ts`, Swift `swift/Sources/TauriWidgets/Resolve.swift`, Kotlin
`android/src/main/java/Resolve.kt`). v2 behaviour is unchanged.

## 1. Local state (`state`) and in-process `set`

Lets a widget change itself on tap without the app running (checklists, tabs, counters).

- `WidgetConfig.state: { key: JSON }`: defaults.
- Stored per placement, shallow-merged over the defaults (later wins per top-level key):
  store key `state:{widgetId}` and `state:{widgetId}#{instance}` (same scoping and
  sanitizing as `data:`; the instance key wins over the widget key).
- Resolve: new root `state` beside `data`: `{{state.k}}`, `when: {path: "state.k"}`,
  `bind`, `repeat.each: "state.list"`. Locals still shadow it. `ResolveContext` gets
  `state` (already merged) like `data`.
- Element field `set: { key: JSON }` on `button`, `toggle`, `link`:
  on tap the renderer writes each pair into the placement's stored state **in-process**
  (WidgetKit AppIntent / Android broadcast receiver / desktop renderer) and redraws.
  The special value `"$toggle"` flips a boolean (missing → `true`). `"$inc"` / `"$dec"`
  add / subtract 1 (missing → 0 first). If the element also has `action`, the action event
  is still sent to the app (after the state write).
- Event `widget-state-changed` `{ widgetId, instance, state }` to the app when it is running.
- JS API (guest-js): `setWidgetState(widgetId, patch, { instance?, group? })`,
  `getWidgetState(widgetId, { instance?, group? })`. Rust commands `set_widget_state`,
  `get_widget_state` (+ permissions).

## 2. `transition` hint

`WidgetConfig.transition?: "none" | "opacity" | "slide" | "numeric"` and the same field on
any element (`ElementStyle.transition`). Hint for how content changes between timeline
entries / redraws. iOS/macOS: `.contentTransition(.numericText())` for `numeric`,
`.transition(.opacity / .push(from: .bottom))`. Desktop HTML: CSS transition on swap.
Android / Adaptive Cards: ignored (degraded in the matrix). Not used by resolve.

## 3. Live Activity templates (data-only payloads)

Push updates stay small when the layout lives on the device.

- `registerActivityTemplate(name, layout)` (JS) / `register_activity_template` (Rust
  command) stores `ActivityContent`-shaped layout fields (lockScreen, island regions, …)
  under store key `activity-template:{name}` in the shared store (App Group on Apple).
- `ActivityContent.template?: string` + `ActivityContent.data?: JSON`. When `template` is
  set, the renderer loads the stored template and resolves every string in it with the
  same resolve contract (`{{data.x}}`, `when`, `repeat`, …) using `data`.
  Fields present directly on the content override the template's.
- Size check (4 KiB) counts only the payload actually sent.

## 4. Fixture format v2

`tests/fixtures/resolve/*.json` may add:
- `"state": {...}` beside `"data"` in the case input (merged state);
- `"platforms": ["rust","ts","swift","kotlin"]` / `"skip": {"swift": "reason"}`;
- `"warnings": [...]` expected (optional, ports may ignore).
Every fixture runner prints/records the list of executed case files, and the Rust runner
fails if a fixture file is neither run nor skipped with a reason.

## Ownership while ports are written in parallel

| Area | Files |
| --- | --- |
| Contract owner | `src/models/**`, `src/resolve.rs`, `src/store.rs`, `src/commands.rs`, `src/lib.rs`, `build.rs`, `permissions/**`, `guest-js/**`, `widget-src/**`, `tests/fixtures/resolve/**`, `tests/unit/**`, generated files (`cargo run --bin gen-native --features codegen`, `gen-schema`, `gen-ts`) |
| Android owner | `android/**` (incl. `Resolve.kt`) except generated `render/WireFields.kt` / `WireTypes.kt` |
| Apple owner | `swift/**` (incl. `Resolve.swift`) except `Generated/`, `ios/Sources/**`, `templates/ios/**`, `templates/macos/**` |
| Desktop owner | `src/desktop.rs` (window code), `src/linux/**` |
| Windows owner | `src/windows/**`, `src/adaptive_card/**`, `templates/windows-widget/**` |
