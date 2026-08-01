# Visual stand (Level-2 pixels)

Reproducible, case-driven screenshots. **No crop rectangles. No “sleep then shoot”.**
Hosts are pinned; readiness is polled (`awaitStable`). Goldens update **one case at a time**.

## Rules

1. Wait on a real condition (two identical frame hashes + non-uniform bitmap), with a deadline.
2. Capture the **element** (`AppWidgetHostView`, SwiftUI view bounds, `#root`) — never a full-screen crop.
3. Environment pinned: locale `en_US`, UTC, font scale 1.0, animations off, theme from the case.
4. Time frozen (`2026-08-01T12:00:00Z` on desktop; emulator/sim date in up-scripts). Fixtures use absolute ISO.
5. Store isolated per case (Android: reset prefs + delete appWidgetId).
6. Record only with an explicit flag **and** `CASE=<name>`.

## Layout

```
tests/cases/<name>.json     # { fixture, size, theme, locale, macos? }
tests/fixtures/...          # IR
tests/golden/{android,ios,desktop,macos}/<name>.png
out/{platform}/             # actual + diff (gitignored)
```

Add a check by dropping a JSON under `tests/cases/` — runners pick it up automatically.

Example case:

```json
{
  "fixture": "presets/weather",
  "size": "small",
  "theme": "dark",
  "locale": "en_US",
  "macos": { "width": 158, "height": 158 }
}
```

`macos` is the **native WidgetKit canvas on Mac** (not iPhone points, not the desktop WKWebView window). Defaults: small 158², medium 338×158, large 338×354.
## Android (AppWidgetHost)

Real Glance → RemoteViews → `AppWidgetHostView`. No launcher, no manual placement.

```bash
just android-up
# grantbind is applied by the script for git.s00d.widgets.test

just test-android-visual
just record-android weather.small
adb pull \
  /storage/emulated/0/Android/data/git.s00d.widgets.test/files/widgets-golden/ \
  tests/golden/android/
```

Filter / record via Gradle:

```bash
cd android && ./gradlew :connectedDebugAndroidTest \
  -Pgolden.record=true \
  -Pcase=weather.small \
  -Pandroid.testInstrumentationRunnerArguments.class=git.s00d.widgets.WidgetRenderTest
```

Robolectric (`pnpm test:android`) remains **Level-1 geometry only** — it does not own pixel goldens.

**Note:** `tests/golden/android/*.png` seeded from the old Robolectric path will not match AppWidgetHost until you re-record on a pinned emulator (`just record-android <case>` + `adb pull`).

## iOS (SwiftUI + WidgetKit chrome, not SpringBoard)

Renders `DynamicElementView` with shared `WidgetChrome` (container background + continuous corner radius). Not SpringBoard chrome.

```bash
just ios-up          # optional pinned simulator for xcodebuild destinations
just test-ios-visual
CASE=weather.small GOLDEN_RECORD=1 just test-ios-visual
# or:
just record-ios weather.small
```

`swift test` runs ImageRenderer on the host (macOS). Use `ios-up` when you switch to `xcodebuild test` on a simulator destination.

Bootstrap after chrome changes (migration only — not day-to-day):

```bash
GOLDEN_RECORD=1 GOLDEN_RECORD_ALL=1 swift test --filter RenderTests/testCases
```

Day-to-day always use `CASE=<name>` + `GOLDEN_RECORD=1` for a single case.

## macOS Level A — AppKit `DynamicElementView` (no sim)

On a Mac there are **three** different surfaces — do not mix them:

| Surface | What | Covered by |
|---------|------|------------|
| Desktop probe window | `widget.html` in WKWebView | Playwright (`tests/golden/desktop`) |
| Native WidgetKit | Notification Center / desktop widget | same `DynamicElementView` as iOS + **AppKit** branches |
| Transport / packaging | App Group / sandbox / UserDefaults, ad-hoc sign, `build-widget.sh` | **Level C** (below) |

## Cross-platform render receipts

Config writes are fire-and-forget on every platform (`updateAll`, `reloadAllTimelines`, `emit`). Renderers write a **receipt** after paint into a sibling store (`widget_receipts.json` / `__tauri_widget_receipts__`) — never the config map (would bump nonce).

```ts
const live = await getWidgetDiagnostics("group.com.example.app");
// [{ widgetId, instance, size, nonce, source, rendered, skipped, ts }]
```

- **macOS only:** `source` + `nonce` drive transport narrowing (fan-out → confirm → narrow → stale reopen).
- **iOS:** no narrowing — loud log if App Group `containerURL` is nil.
- **Android:** receipts list live `appWidgetId`s so `syncConfigToGlanceState` targets confirmed instances.
- **Desktop:** register listeners **before** `loadConfig()`; `widget.html` reports `source: push|pull` after render.

## Level C — macOS transports + packaging (no pixels)

Real host bugs live here: freshest pick, **async receipts → write narrowing**, action queue, App Group entitlements, `.appex` embed.

**Model:** fan-out write → widget plants receipt `{readFrom,nonce,ts}` (sibling file/key, not config map) → host `reconcile()` → after `CONFIRM_STREAK` confirmations narrow config writes to that transport → `STALE_AFTER_MS` silence re-opens fan-out. Availability ≠ delivery.

**C1 — transports** (seconds; override root, never mutate process `HOME`):

```bash
just test-macos-transports
# Rust unit (fake TransportSet): cargo test -p tauri-plugin-widgets transport::
# Rust file IO: cargo test --test macos_transports
# Swift: cd swift && swift test --filter TransportTests
```

Env knobs: `WIDGET_CONTAINER_ROOT`, `WIDGET_EXTENSION_BUNDLE`, `WIDGET_APP_GROUP_DATA_FILE` (and Swift `WIDGET_SANDBOX_DATA_FILE`). Install identity reset: `WIDGET_BUNDLE_VERSION` / `WIDGET_TEAM_ID_HASH`.

**C2 — build pipeline** (slow; needs `xcodegen` + Xcode):

```bash
just test-macos-pipeline
# → tests/macos/pipeline.sh
```

Asserts: `init-macos` → executable scripts → `.appex` builds → App Group in entitlements → `widgetkit-extension` → embed into `Contents/PlugIns/` (stub `.app`, `WIDGET_SKIP_DMG=1`).

Level A runs `swift test` on the host in seconds and hits `#elseif canImport(AppKit)` (`Color.adaptive` via `NSAppearance`, semantic `NSColor`, `NSImage` decode) — which an iOS-simulator run never executes.

```bash
just test-macos-visual
just record-macos weather.small
# bootstrap:
cd swift && GOLDEN_RECORD=1 GOLDEN_RECORD_ALL=1 swift test --filter MacRenderTests
```

Renders via `NSHostingView` at **`case.macos` points** into `tests/golden/macos/`. Not Notification Center chrome; not `widget.html`.

## Desktop (Playwright)

```bash
pnpm test:visual:desktop
just record-desktop weather.small
```

`#root` screenshot after `awaitStable`. Platform folder: `desktop` (darwin), `linux`, `windows` (Playwright desktop webview — not Widgets Board).

## Windows Widgets Board (Adaptive Cards PreviewHost)

Native AC pixels (not `widget.html`):

```bash
# Mac: regenerate PNG + AC JSON for the core trio
bash tools/gen-windows-goldens.sh

# UTM: capture via PreviewHost
just record-windows weather.small
just test-windows-visual
node tests/windows/compare.mjs
```

Goldens live in `tests/golden/windows/`. See [windows-surfaces.md](windows-surfaces.md).

Geometry Level-1 still lives in `tests/expected/geometry` and updates with `UPDATE_SNAPSHOTS=1` (bulk OK for trees). Pixel goldens never bulk-overwrite.

## Audit (batch contact sheets)

```bash
pnpm audit:sheets
# → out/audit/<case>.png + out/audit/index.html
# panels: Desktop | iOS | macOS | Android | Windows
open out/audit/index.html
```

Checklist when reviewing: padding/spacing, background fill, color/contrast, overflow/clip, empty/uniform, chrome mismatch.

Findings from the last full pass live in `out/audit/CATALOG.md` (gitignored under `out/`).

## Commands cheat sheet

| Action | Command |
|--------|---------|
| Desktop visual | `pnpm test:visual:desktop` |
| Record desktop case | `CASE=x.y GOLDEN_RECORD=1 pnpm test:visual:desktop` |
| Windows AC goldens (Mac) | `bash tools/gen-windows-goldens.sh` |
| Windows visual (UTM) | `just test-windows-visual` / `just record-windows weather.small` |
| Windows xwin check | `just check-windows-xwin` |
| Windows example xwin | `just build-windows-example-xwin` |
| Android visual | `just test-android-visual` |
| Record android case | `just record-android x.y` then `adb pull …` |
| iOS visual | `just test-ios-visual` |
| Record ios case | `just record-ios x.y` |
| macOS AppKit visual | `just test-macos-visual` |
| Record macOS case | `just record-macos x.y` |
| macOS transports (C1) | `just test-macos-transports` |
| macOS pipeline (C2) | `just test-macos-pipeline` |

See also [render-testing.md](./render-testing.md) for Level-1 geometry.
