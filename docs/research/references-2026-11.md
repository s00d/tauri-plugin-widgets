# Reference research, November 2026

Five deep-dive surveys (code and tests read, not READMEs) of comparable projects. Raw notes
and clones were kept outside the repo. This page is the ranked backlog distilled from them;
the **Status** column is updated as items land.

Sources read: Voltra, tauri-plugin-live-activity, DivKit, Adaptive Cards templating, rfw,
A2UI, json-render, Stac, Remote Compose, Beagle, JSON Forms, expo-widgets,
expo-apple-targets, react-native-widget-extension, home_widget, flutter_live_activities,
Capacitor widget/live-activity plugins, Home Assistant iOS, Wikipedia iOS, Ice Cubes,
Zeitgeist, opentask, XcodeGen, Glance samples and androidx Glance tests,
glance-experimental-tools, Roborazzi, Paparazzi, eww, Übersicht, zebar, Deskulpt,
Seelen-UI, Lively, Rainmeter, tauri-plugin-desktop-underlay, tauri-nspanel,
window-vibrancy, notify-rust, Tauri plugins-workspace, tauri-plugin-webdriver,
WindowsAppSDK widget samples, DevHome, cargo-mutants, Stryker, Flutter / React Native CI,
swift-snapshot-testing, serde_json, android-emulator-runner.

## Contract (resolve)

| Item | From | Status |
| --- | --- | --- |
| Structured `when` conditions (`eq/ne/gt/gte/lt/lte/exists/in/all/any/not`) | A2UI, json-render, AC `$when` | done (4 ports) |
| `requires` + fallback layout (old renderers ignore new keys silently today) | AC `requires`, Remote Compose profiles | done (4 ports) |
| `repeat` with splice semantics, `{{index}}`, limits | AC `$data`, DivKit item_builder | done (4 ports) |
| `templates` + `use` with `args` (instance overrides win) | DivKit templates, rfw | done (4 ports) |
| Fixture format v2: `warnings`, `platforms`/`skip` with reasons, executed-case manifest | DivKit test_data, A2UI conformance | done (`platforms`/`skip`, executed list; Rust fails on unrun files) |
| Live Activity data-only payloads (layout stored on device) | ludufre LiveActivitiesKit, ActivityKit split | done (`registerActivityTemplate`, `template` + `data`) |
| Local `state` + `set` action handled in-process (replaces Android list-toggle hack) | DivKit variables, Cap-go applyPatch | done (all platforms; Widgets Board shows defaults only) |
| `transition` hint between entries | DivKit transitions, SwiftUI contentTransition | done (Apple, desktop; Android / Board ignore) |
| Numbers like JavaScript everywhere (exponent outside 1e-6…1e21) | found by fast-check parity | done |
| Correctly rounded float parsing in Rust (`float_roundtrip`) | found by fast-check parity | done |

## Apple

| Item | Status |
| --- | --- |
| Extension version = app version in `init-ios` (App Store blocker) | done |
| `accentable` → `.widgetAccentable()` | done |
| Root `url` → `widgetURL` (small / accessory widgets ignore Link) | done |
| Accessory families (circular / rectangular / inline) + rendering-mode env | done (`accessory` layout; no chrome in accessory / non-full-color modes) |
| Activity lifecycle events (`activityStateUpdates`), relevance, alerts, final content on end, `timestamp` ordering, `NSSupportsLiveActivitiesFrequentUpdates` | done on iOS (`activity-state` / `activity-content` events; `relevanceScore`, `alert`, `timestamp` in the Rust model) |
| Island expanded regions, keyline, `isStale`, supplemental `.small` family | done |
| In-extension state patches + `Toggle(isOn:intent:)`, `invalidatableContent` | done (`WidgetStateIntent`; taps live once generated `WidgetElement` has `set`) |
| Widget push (`WidgetPushHandler`, iOS 26) | done (`TauriWidgetPushHandler`, token in the App Group) |
| `defaultResult()` for the instance picker; gallery preview from stored config | done |
| Lossy decoding of `children` (one bad element must not blank the widget) | done (generated `init(from:)`) |
| `TauriActivityAttributes` contract test shared by host and package | done (`StateTests.testActivityAttributesMatchHost`) |

## Android

| Item | Status |
| --- | --- |
| Size family stale after resize; wrong portrait/landscape pairs → `SizeMode.Exact` + family from `LocalSize` | done |
| Chrome bitmaps drawn at provider minimum size (`SizeMode.Single`); `frame.width` px vs dp | done (via `SizeMode.Exact`) |
| Geometry test compares only the root rect → diff the whole tree | done (21 stale baselines re-recorded) |
| Accessibility: chart/progress descriptions with values; native `Switch`/`CheckBox`; depth guard (Glance limit 50) | done (values spoken for progress / gauge / chart; native `Switch`/`CheckBox`; depth guard drops subtrees past 15 element levels with a `depth` receipt skip) |
| `glance-appwidget-testing` behaviour tests (click actions, checked state) | done (`GlanceBehaviourTest`: callback params incl. `set`, checked from `state`, generated preview) |
| Widget-info: loading layout, preview image/layout, description, reconfigurable | done (`tauri_widget_loading`, `tauri_widget_preview`, `reconfigurable\|configuration_optional`) |
| `onCompositionError` styled fallback + receipt | done (`tauri_widget_error`, tap opens the app; receipt `trigger: "error"`) |
| Day/night `ColorProvider(day, night)`, dynamic colors via `GlanceTheme` | done (`{light,dark}` → day/night provider when the background is not explicit; semantic names → day/night color resources, `accent` = `system_accent1` on 12+; resources instead of `GlanceTheme`) |
| Live Updates: `setRequestPromotedOngoing`, `setShortCriticalText`, `ProgressStyle` segments/points, promotion check + settings deep link | done (API 36 via the framework builder, reflection; `progressSegments` / `progressPoints` in the Rust model; `canPromoteActivities` / `openActivityPromotionSettings` in Rust and JS) |
| Generated previews (`providePreview`, Glance 1.2, compileSdk 35), pin with preview | done (Glance 1.2, compileSdk 35; `setWidgetPreviews` after `setWidgetConfig`, Android 15+; `requestWidget` pins with `PinPreviewWidget` showing the stored config) |
| Instrumented test: synchronous host view + latch, `WorkManagerTestInitHelper` | done (latch on `updateAppWidget` + 1.5 s quiet period instead of a fixed sleep, so the settled composition is captured; in-memory WorkManager; 4 goldens recorded from early frames re-recorded) |
| Source fetch in WorkManager (network constraint, backoff) | done (`WidgetSourceWorker`, unique per placement, 3 tries) |
| Images by content URI (RemoteViews bitmap budget) | done (cached downloads via `TauriWidgetFileProvider`, read grant to HOME apps; bitmap fallback) |
| Configuration activity (trampoline + `finishWidgetConfiguration`) | done (`WidgetConfigActivity`: picks the placement instance from `instances:{widgetId}`) |
| Samsung Now Bar extras, keyguard category, cover screen, scrollable list, update timestamp guard | done ([android-samsung-2026-11.md](./android-samsung-2026-11.md): `ongoingActivityNoti` extras on Samsung below API 36 only; `private` → redacted public version; `keyguard` + `sub_screen` meta-data, keyguard host → `small`; `list` → `LazyColumn` past the widget height; stale `timestamp` dropped, `applied: false`; not checked on Samsung hardware; Rust `update_activity` returns `applied`) |

## Desktop

| Item | Status |
| --- | --- |
| Linux X11 pin before first map (`visible(false)` → pin → `show()`), sticky/below, re-apply position | done (`src/linux/mod.rs`, GDK hints; not run on Linux here) |
| Wayland layer-shell anchors + margins from x/y, `set_monitor`, namespace, layer choice | done (`src/linux/mod.rs`, `placement::layer_anchor`; not run on Wayland here) |
| Window modes: `layer: normal/top/bottom/desktop`, `clickThrough`, `focusable`, `allWorkspaces`, `shadow`, `effects` | done (`src/desktop/window.rs`) |
| WorkerW: wire it up, Windows 11 24H2 Progman-child fallback, re-attach on explorer restart | done (`src/windows/workerw.rs`; not run on Windows) |
| Placement + persistence (monitor, anchor, offsets; restore with intersection check; DPI) | done (`src/desktop/placement.rs`, `widget-windows.json`) |
| Activity window: work area, no focus steal | done |
| Widgets Board: all sizes via `$host.widgetSize`, recover running widgets, `CustomState`, customization / analytics / errors interfaces | done (`adaptive_card::board_card`, `templates/windows-widget`; not run on Windows) |
| Notification fallback for activities (Windows toast progress, Linux notify-rust replace-id) | done (feature `notify`, `src/desktop/notify.rs`) |
| Host-registered poll / listen data sources (eww `defpoll`/`deflisten`), Rust API only | done (`src/desktop/sources.rs`) |
| Restore widget windows after autostart | done (`restoreWindowsOnStart`, `Widget::restore_windows`) |

## Testing

| Item | Status |
| --- | --- |
| proptest invariants for resolve / config round-trip / URL check | done (`tests/resolve_props.rs`) |
| fast-check differential test TS vs Rust (`resolve-stdin` oracle) | done (`tests/unit/resolve.parity.test.ts`) |
| cargo-fuzz target for untrusted JSON (resolve, normalize, AC, rasterize) | done (`fuzz/`) |
| CI matrix: macOS (Swift tests, iOS simulator build), Windows, Android/iOS target checks, Playwright in a pinned container, CLI + docs checks | done (`ci.yml`: `windows`, `ios-build`, `mobile-targets`, `playwright` in `mcr.microsoft.com/playwright:v1.63.0-noble`, `pnpm test:cli`) |
| Missing desktop golden must fail in CI; flaky Playwright surfaced; diff artifacts | done (`GOLDEN_STRICT` in `tests/desktop/cases.ts`, `tests/desktop/flaky-report.mjs`, `playwright-diffs` artifact) |
| Linux golden collision (Chromium vs WebKitGTK) → `linux-chromium` baseline | done (`tests/golden/linux-chromium`, `pnpm test:desktop:linux[:record]`) |
| Glance behaviour tests (`glance-appwidget-testing`: click actions) | done (Android, `GlanceBehaviourTest`) |
| Mock-runtime Rust tests (`tauri::test`) for commands / windows / activities | done (`tests/offline_model.rs` + stories in `tests/`, real ACL; activity start ignored: MockRuntime `primary_monitor` is `unimplemented!()`) |
| Mutation runs (cargo-mutants, Stryker, PIT, Muter) | local only by owner's choice; not in CI (`pnpm mutants:*`) |
| `init-ios.ts` unit tests on fixture `project.yml` + extension build e2e | done (`tests/cli/init-ios.test.mjs`; e2e: `pnpm test:ios:e2e`) |
| nextest flaky quarantine profile | done (`.config/nextest.toml`, `pnpm test:rust:nextest`) |
