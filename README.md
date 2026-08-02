[![npm version](https://img.shields.io/npm/v/tauri-plugin-widgets-api/latest?style=for-the-badge)](https://www.npmjs.com/package/tauri-plugin-widgets-api)
[![Crates.io](https://img.shields.io/crates/v/tauri-plugin-widgets?style=for-the-badge)](https://crates.io/crates/tauri-plugin-widgets)
[![Documentation](https://img.shields.io/badge/docs-docs.rs-blue?style=for-the-badge)](https://docs.rs/tauri-plugin-widgets/)
[![GitHub issues](https://img.shields.io/badge/github-issues-orange?style=for-the-badge)](https://github.com/s00d/tauri-plugin-widgets/issues)
[![GitHub stars](https://img.shields.io/badge/github-stars-yellow?style=for-the-badge)](https://github.com/s00d/tauri-plugin-widgets/stargazers)
[![Donate](https://img.shields.io/badge/Donate-Donationalerts-ff4081?style=for-the-badge)](https://www.donationalerts.com/r/s00d88)

# Tauri Plugin Widgets

A Tauri v2 plugin for **cross-platform widgets** from one JSON UI config:

- **Android** — Jetpack Glance AppWidget
- **iOS / macOS** — WidgetKit + SwiftUI (`TauriWidgets` package)
- **Windows** — Widgets Board via Adaptive Cards **and** desktop webview fallback
- **Linux** — desktop webview pinned as `_NET_WM_WINDOW_TYPE_DESKTOP` (optional gtk-layer-shell on Wayland)

Each OS backend follows `target_os` (optional Cargo features). See [Platform Support](#platform-support).

This README is the **integration guide**. Contributor docs (architecture, goldens, harnesses): [`docs/development.md`](docs/development.md).

## Contents

1. [Demo / Preview](#demo)
2. [Breaking changes](#breaking-changes-in-04) · [Capability matrix](#capability-matrix-element--platform)
3. [Quick Start](#quick-start) · [CLI](#cli--quick-init)
4. [Platform Setup](#platform-setup)
5. [Widget Config Schema](#widget-config-schema) · [API](#low-level-data-api)
6. [Troubleshooting](#troubleshooting)
7. [Development](docs/development.md)

## Demo

<div align="center">
  <img style="border-radius: 40px; border: 1px solid black;" src="https://raw.githubusercontent.com/s00d/tauri-plugin-widgets/refs/heads/main/demo.gif" alt="Demo" width="50%" />
</div>

## Preview

Same JSON config rendered natively on each platform (`nested-dashboard` fixture):

<table>
  <tr>
    <td align="center" width="33%">
      <img src="./docs/screenshots/desktop-nested-dashboard.png" alt="Desktop nested dashboard widget" width="280" /><br />
      <sub><b>Desktop</b> — HTML / webview</sub>
    </td>
    <td align="center" width="33%">
      <img src="./docs/screenshots/android-nested-dashboard.png" alt="Android nested dashboard widget" width="280" /><br />
      <sub><b>Android</b> — Jetpack Glance</sub>
    </td>
    <td align="center" width="33%">
      <img src="./docs/screenshots/ios-nested-dashboard.png" alt="iOS nested dashboard widget" width="280" /><br />
      <sub><b>iOS / macOS</b> — SwiftUI</sub>
    </td>
  </tr>
</table>

<div align="center">
  <img src="./preview.png" alt="Widgets preview on Android and iOS home screens" width="85%" />
</div>

---

## Breaking changes in 0.4

- `setWidgetConfig(config, group, widgetId, skipReload?)` — **`widgetId` required**
- `getWidgetConfig(group, widgetId)` — **`widgetId` required**
- `startWidgetUpdater(builder, group, widgetId, options?)` — **`widgetId` required**
- `createWidgetWindow` built-in renderer requires `widgetId`
- Storage keys renamed: `config:{widgetId}`, `pending_actions` (no `__widget_config__` / `__widget_pending_actions__` migration)
- Action payload: `{ action, payload?, ts, widgetId, group }`
- iOS `group` must start with `group.` (no silent rewrite)

## Capability matrix (element × platform)

Source of truth: Rust [`src/capabilities.rs`](src/capabilities.rs) → generated [`docs/capability-matrix.md`](docs/capability-matrix.md) (5 platforms: **iOS**, **macOS**, **Android**, **Desktop** HTML, **Windows** Adaptive Cards).

### Core elements

| Element | iOS | macOS | Android | Desktop | Windows |
|---------|-----|-------|---------|---------|---------|
| `vstack` / `hstack` / `grid` / `container` | full | full | full | full | full (AC 1.5) |
| `zstack` | full | full | full | full | degraded (flattened, no overlay) |
| `text` / `label` / `spacer` / `divider` | full | full | full | full | full (AC 1.5) |
| `progress` / `button` / `toggle` / `date` / `link` | full | full | full | full | full (AC 1.5) |
| `image` | full | full | degraded | full | degraded |
| `shape` | full | full | full | full | degraded (PNG) |

### Extended elements

| Element | iOS | macOS | Android | Desktop | Windows |
|---------|-----|-------|---------|---------|---------|
| `gauge` | full | full | full | full | degraded (PNG) |
| `chart` | full | full | degraded | full (SVG) | degraded (PNG) |
| `list` | full | full | full | full | degraded (TextBlocks) |
| `timer` | full | full | degraded | full | degraded (static) |
| `canvas` | full | full | degraded | full (SVG) | degraded (PNG) |

### Feature notes

| Feature | iOS | macOS | Android | Desktop | Windows |
|---------|-----|-------|---------|---------|---------|
| `image.url` | unsupported | unsupported | full (→ localPath) | full | full |
| `image.systemName` | full (SF Symbols) | full | degraded | degraded | unsupported |
| `background.gradient` | degraded (linear primary) | degraded | degraded (1st stop) | full | unsupported |
| `canvas.path` | degraded (M/L/H/V/Z) | degraded | degraded | full | degraded (PNG) |
| `timer.live` | full | full | **unsupported** | full | **unsupported** |

Full notes and wording: [`docs/capability-matrix.md`](docs/capability-matrix.md).

## Authoring widgets

**IR source of truth is Rust** (`src/models.rs`). TypeScript types are generated; native Swift/Kotlin renderers map the same JSON.

### JavaScript / TypeScript (primary DX)

```ts
import { setWidgetConfig, type WidgetConfig } from "tauri-plugin-widgets-api";

const config: WidgetConfig = {
  small: {
    type: "vstack",
    padding: 12,
    background: "#1a1a2e",
    children: [
      { type: "text", content: "72°", fontSize: 36, fontWeight: "bold", color: "#fff" },
      { type: "progress", value: 0.7, tint: "#4CAF50", label: "Humidity" },
    ],
  },
};
await setWidgetConfig(config, "group.com.example.myapp", "weather");
```

Regenerate types after model changes: `pnpm codegen` (see [`docs/development.md`](docs/development.md)).

On Android, each home-screen instance maps to a logical `widgetId` (meta `widgetId:{appWidgetId}`). Call `setWidgetConfig` per id after pinning so two widgets can show different configs.

Capability warnings (degraded / unsupported) are logged when a config **changes**, for the current platform only.

> Prefer flat `vstack` / `hstack` on Android; always set `progress.label`. Deeper architecture: [`docs/development.md`](docs/development.md).

---

## Features

- **Universal Widget UI** — one JSON IR, five renderers (SwiftUI, Glance, HTML, Adaptive Cards).
- **Three size families** — `small`, `medium`, `large` in a single config.
- **21 element types** — text, image, progress, gauge, chart, list, button, toggle, divider, spacer, date, link, shape, timer, label, canvas, and layout containers (vstack, hstack, zstack, grid, container).
- **Action buttons & tappable wrappers** — `button` / `link` emit `widget-action` events back to the app.
- **Dark mode & adaptive colors** — semantic names and `{ light, dark }` objects.
- **Semantic typography** — `textStyle` (`largeTitle`, `body`, `caption`, …).
- **Flexible layout** — `flex`, `clipShape`, padding, gradients, borders, shadows, frames.
- **Declarative canvas** — circles, lines, arcs, paths via JSON commands.
- **Swift Package** — reusable `TauriWidgets` for iOS/macOS extensions.
- **Windows Widgets Board** — Adaptive Cards transpile + optional WorkerW wallpaper parenting.
- **Linux desktop pin** — X11 `_NET_WM_WINDOW_TYPE_DESKTOP` (+ optional Wayland `layer-shell`).
- **Desktop widget windows** — frameless transparent Tauri webviews.
- **Low-level data API** — `setItems` / `getItems` shared with native widgets.

---

## Platform Support

| Platform | Surface | UI | Storage | Reload / update |
|----------|---------|----|---------|-----------------|
| **Android** | AppWidget | Jetpack Glance from JSON | SharedPreferences + Glance state | Glance `updateAll()` |
| **iOS** | WidgetKit (17+) | SwiftUI from JSON | App Group (`transport=appGroup`) | WidgetCenter |
| **macOS** | WidgetKit (14+) | SwiftUI from JSON | Config-chosen transport (see below) | WidgetCenter |
| **Windows** | Widgets Board **+** desktop webview | Adaptive Cards 1.5 **and** HTML/CSS | JSON file + `ac:template` / `ac:data` | Provider / Tauri |
| **Linux** | Desktop webview (X11 DESKTOP pin; optional layer-shell) | HTML/CSS from JSON | JSON file | Tauri |

OS backends follow `target_os` (no empty Cargo feature toggles). Optional features:

- `linux` (**default**) — gtk + x11rb for `_NET_WM_WINDOW_TYPE_DESKTOP` pin
- `rasterize` — SVG→PNG data-URI for Adaptive Cards chart/canvas/gauge (pulls resvg)
- `workerw` — Windows wallpaper WorkerW parenting
- `layer-shell` — Wayland gtk-layer-shell Background (requires `linux`)
- `macos-private-api` — transparent macOS webviews

```toml
# Default (linux pin enabled; other OS via target_os)
tauri-plugin-widgets = "0.4"

# Slim: no linux pin / no gtk
tauri-plugin-widgets = { version = "0.4", default-features = false }

# Windows Widgets Board with chart rasterization
tauri-plugin-widgets = { version = "0.4", default-features = false, features = ["rasterize"] }
```

---

## Quick Start

### 1. Install

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-widgets = "0.4"
# Or without linux pin:
# tauri-plugin-widgets = { version = "0.4", default-features = false }
# With Adaptive Cards rasterization:
# tauri-plugin-widgets = { version = "0.4", features = ["rasterize"] }
```

```bash
npm install tauri-plugin-widgets-api
# or
pnpm add tauri-plugin-widgets-api
```

### 2. Register the Plugin

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_widgets::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 3. Add Permissions

`src-tauri/capabilities/default.json`:

```json
{
  "permissions": [
    "core:default",
    "widgets:default"
  ]
}
```

### 4. Send a Widget Config

```typescript
import { setWidgetConfig } from "tauri-plugin-widgets-api";

await setWidgetConfig({
  small: {
    type: "vstack",
    padding: 14,
    background: { light: "#E8F4FD", dark: "#1a1a2e" },
    spacing: 6,
    cornerRadius: 16,
    children: [
      {
        type: "hstack",
        spacing: 8,
        children: [
          { type: "image", systemName: "cloud.sun.fill", color: "#ffcc00", size: 28 },
          { type: "text", content: "72°", textStyle: "largeTitle", fontWeight: "bold", color: "label" },
        ],
      },
      { type: "text", content: "Partly Cloudy", textStyle: "footnote", color: "secondaryLabel" },
      { type: "progress", value: 0.65, tint: "#4CAF50", label: "Humidity" },
    ],
  },
}, "group.com.example.myapp", "weather");
```

### Android Glance Layout Guidelines

To reduce rendering issues on Android launchers, keep widget trees simple:

- Prefer `vstack` / `hstack` with style props (`padding`, `background`, `cornerRadius`) over deep `container` nesting.
- Avoid `container -> vstack` wrappers when a styled `vstack` can express the same UI.
- In dense rows, use simple children (`text`, `image`, `gauge`, `progress`) with `flex`.
- Keep hierarchy depth low (about 3-4 levels per branch in most presets).
- Always provide `progress.label` so host UIs never show `null`.

Recommended card pattern:

```json
{
  "type": "vstack",
  "spacing": 2,
  "padding": 8,
  "background": { "light": "#EDE9FE", "dark": "#313244" },
  "cornerRadius": 8,
  "children": [
    { "type": "text", "content": "Revenue", "textStyle": "caption2", "color": "secondaryLabel" },
    { "type": "text", "content": "$12,450", "fontSize": 18, "fontWeight": "bold", "color": "#a6e3a1" }
  ]
}
```

---

## CLI — Quick Init

The plugin includes a CLI tool to scaffold native widget extensions. It reads your `tauri.conf.json` to auto-detect the bundle identifier and app group.

```bash
npx tauri-plugin-widgets-api init-macos
npx tauri-plugin-widgets-api init-ios
npx tauri-plugin-widgets-api init-windows
```

### macOS Widget Extension

```bash
npx tauri-plugin-widgets-api init-macos
```

This creates `src-tauri/macos-widget/` and auto-patches `tauri.conf.json` with `beforeBundleCommand`, `bundle.macOS.files` (→ `Contents/PlugIns/`), and host `entitlements`.

```bash
# One command: builds .appex, copies into PlugIns, signs, creates DMG
pnpm tauri build
```

**Options:**

| Flag | Description | Default |
|------|-------------|---------|
| `--bundle-id` | Widget bundle identifier | Auto: `<tauri identifier>.widgetkit` |
| `--app-group` | App Group identifier | Auto: `group.<tauri identifier>` |
| `--dir` | Target directory | `src-tauri/macos-widget` |
| `--force` | Overwrite existing files | `false` |

```bash
# Explicit identifiers:
npx tauri-plugin-widgets-api init-macos \
  --bundle-id com.example.myapp.widgetkit \
  --app-group group.com.example.myapp
```

### iOS Widget Extension

```bash
npx tauri-plugin-widgets-api init-ios
```

Creates `src-tauri/ios-widget/MyWidget.swift` and prints step-by-step Xcode setup instructions.
App Group is auto-generated from `tauri.conf.json` `identifier` (or can be overridden with `--app-group`).
If a Widget Extension target already exists in `src-tauri/gen/apple/*`, the CLI also auto-syncs its generated widget Swift file.

**Options:**

| Flag | Description | Default |
|------|-------------|---------|
| `--app-group` | App Group identifier | Auto: `group.<tauri identifier>` |
| `--dir` | Target directory | `src-tauri/ios-widget` |
| `--force` | Overwrite existing files | `false` |

> **Note:** Recommended flow: run `init-ios` once, create the Widget Extension target in Xcode, then run `init-ios` again to auto-replace the generated default widget code.

---

## Environment Variables

| Variable | Scope | Default | Description |
|----------|-------|---------|-------------|
| `WIDGET_TRANSPORT` | Apple host | `plugins.widgets.transport` | Override: `appGroup` \| `userDefaults` \| `widgetContainer` \| `auto`. |
| `WIDGET_SIGN_IDENTITY` | macOS (`build-widget.sh`) | `APPLE_SIGNING_IDENTITY` or ad-hoc (`-`) | Signing identity for the `.appex` before bundling. |
| `TAURI_WIDGET_MIN_RELOAD_SECS` | iOS/macOS runtime (plugin) | Debug: `0`, Release: `900` | Minimum seconds between plugin-triggered `reloadAllTimelines()`. Use `0` to disable plugin-side throttle. |
| `TAURI_DEV_HOST` | Example app dev (`vite.config.ts`) | — | Dev host used by Tauri/Vite during `tauri dev` (usually set automatically). |

Examples:

```bash
# iOS dev without plugin-side reload throttle
TAURI_WIDGET_MIN_RELOAD_SECS=0 pnpm tauri ios dev

# Force widget-container transport without editing tauri.conf.json
WIDGET_TRANSPORT=widgetContainer pnpm tauri dev

# macOS build with explicit signing identity for the .appex
WIDGET_SIGN_IDENTITY="Apple Development: you@example.com (TEAMID)" \
  pnpm tauri build
```

---

## Platform Setup

### iOS Setup

#### Step 1: Initialize the iOS project

```bash
pnpm tauri ios init
```

#### Step 2: Add a Widget Extension target

Open the generated Xcode project:

```bash
open src-tauri/gen/apple/*.xcodeproj
```

In Xcode: **File → New → Target → Widget Extension**. Name it (e.g. `WidgetExtension`), language: Swift.

In the **Choose options for your new target** dialog:
- Set `Product Name` to `WidgetExtension` (or your widget name)
- Select your Apple `Team` (e.g. Personal Team)
- Keep `Project` as your current iOS project (e.g. `myapp`)
- Set `Embed in Application` to your iOS app target (e.g. `myapp_iOS`)
- For the basic plugin setup, disable:
  - `Include Live Activity`
  - `Include Control`
  - `Include Configuration App Intent`

#### Step 3: Add the TauriWidgets Swift Package

In Xcode:
1. **File → Add Package Dependencies...**
2. Click **Add Local...**
3. Select the package folder:
   - for typical app projects: `node_modules/tauri-plugin-widgets-api/swift/`
   - for this repository example: `swift/` (repository root)
4. In **Add to Target**, choose your widget target (`WidgetExtension` / `WidgetExtensionExtension`).

Important: `TauriWidgets` must be linked to the widget target itself. If it is linked only to the main iOS app target, `import TauriWidgets` fails with `no such module`.

#### Step 4: Sync the widget code via CLI (recommended)

After creating the Widget Extension target, run:

```bash
npx tauri-plugin-widgets-api init-ios
```

This updates the generated `src-tauri/gen/apple/*/*.swift` widget entry file from the plugin template using the auto-generated App Group.
It also adapts the widget struct name to match Xcode-generated `*Bundle.swift` references, preventing `cannot find 'WidgetExtension' in scope`.

Manual fallback (if you prefer to edit/copy by hand):

```swift
import SwiftUI
import WidgetKit
import TauriWidgets

struct MyWidgetEntryView: View {
    var entry: TauriWidgetEntry
    var body: some View {
        TauriWidgetView(entry: entry)
    }
}

@main
struct MyWidget: Widget {
    let kind = "ExampleWidget"
    var body: some WidgetConfiguration {
        StaticConfiguration(
            kind: kind,
            // Use the same App Group that `init-ios` generated for your project.
            provider: TauriWidgetProvider(appGroup: "group.<your-tauri-identifier>")
        ) { entry in
            MyWidgetEntryView(entry: entry)
        }
        .configurationDisplayName("My Widget")
        .description("Powered by TauriWidgets")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
    }
}
```

Or copy from `templates/ios-widget/MyWidget.swift`.

#### Step 5: Configure App Groups

1. Select the **main app target** (your iOS app, e.g. `myapp_iOS`) → **Signing & Capabilities**.
2. Click **+ Capability** → add **App Groups**.
3. In the App Groups block click `+` and add your group (use the value printed by `init-ios`).
4. Repeat the same for the **WidgetExtension** target.
5. Verify the App Group value is **exactly the same** in both targets.
6. If **+ Capability** is disabled, set a valid **Team** in Signing for that target first.

#### Step 6: Run

```bash
pnpm tauri ios dev
```

---

### macOS Setup

macOS widgets require a "sidecar" Xcode project — a small project alongside your Tauri app that compiles the Widget Extension (`.appex`). After `init-macos`, a normal **`pnpm tauri build`** produces an `.app` (and DMG) with `Contents/PlugIns/*.appex` already inside.

> **Important:** On macOS, WidgetKit picks up extension widgets from the installed app bundle.  
> Addable widgets appear only after you **build** the app and move the resulting `.app` to **`/Applications`** (or install from the DMG).

**Requirements:** `xcodegen` (`brew install xcodegen`)

#### Step 1: Initialize the widget project (automatic)

```bash
# Recommended: use the CLI (auto-detects bundle ID from tauri.conf.json)
npx tauri-plugin-widgets-api init-macos

# Or with explicit identifiers:
npx tauri-plugin-widgets-api init-macos \
    --bundle-id com.example.myapp.widgetkit \
    --app-group group.com.example.myapp
```

This creates `src-tauri/macos-widget/` with:
- `Sources/MyWidget.swift` — widget code using `TauriWidgetProvider`
- `TauriWidgetExtension.entitlements` — App Group for the extension
- `App.entitlements` — App Group for the host app (wired via `bundle.macOS.entitlements`)
- `project.yml` — xcodegen spec
- `build-widget.sh` — builds and signs the `.appex` (`beforeBundleCommand`)
- `embed-widget.sh` — **deprecated** emergency re-embed; not part of the normal DX

#### Step 2: Configure tauri.conf.json

The CLI patches these fields (paths relative to `src-tauri/`):

```json
{
  "build": {
    "beforeBundleCommand": "./src-tauri/macos-widget/build-widget.sh"
  },
  "bundle": {
    "macOS": {
      "entitlements": "./macos-widget/App.entitlements",
      "files": {
        "PlugIns/TauriWidgetExtension.appex": "./macos-widget/build/Build/Products/Release/TauriWidgetExtension.appex"
      }
    }
  }
}
```

- **`beforeBundleCommand`** builds and signs the `.appex` (failures are not swallowed)
- **`bundle.macOS.files`** copies the `.appex` into `Contents/PlugIns/` during bundling
- Tauri nested-codesigns `PlugIns/` and can produce a normal DMG via `bundle.targets`

#### Step 3: Build

```bash
pnpm tauri build

# Optional: identity for signing the .appex before bundling
WIDGET_SIGN_IDENTITY="Apple Development: you@example.com (TEAMID)" pnpm tauri build
```

Pipeline:
1. Rust + frontend build
2. `build-widget.sh` → signed `.appex`
3. Tauri copies PlugIns, nested-codesigns, notarizes (if configured), writes `.app` / DMG

> **Tip:** `{ "scripts": { "build:macos": "tauri build" } }` then `pnpm build:macos`.

#### Apple data transport

The host writes widget data through **one** transport you choose in config. You know your signing setup — do not rely on runtime fan-out.

```json
{
  "plugins": {
    "widgets": {
      "appGroup": "group.com.example.myapp",
      "transport": "appGroup",
      "extensionBundleId": "com.example.myapp.widgetkit"
    }
  }
}
```

| Situation | `transport` |
|-----------|-------------|
| Release / Team ID + App Groups enabled | `appGroup` |
| Mac App Store | `appGroup` |
| Local ad-hoc signing (no shared App Group container) | `widgetContainer` |
| iOS (device and simulator) | `appGroup` only — other values fail at plugin init |
| Not sure yet | `auto` once at startup (dev only) — read the log, then pin the winner in conf |

| `transport` | Host write path | Requirements |
|-------------|-----------------|--------------|
| `appGroup` | `containerURL(group)/widget_data.json` | Real Team ID + App Groups on App + Extension |
| `userDefaults` | App Group `UserDefaults` suite | Same as `appGroup` |
| `widgetContainer` | `~/Library/Containers/<appex>/Data/widget_data.json` | macOS host **not** sandboxed; works with ad-hoc |
| `auto` | One-shot probe, then latch | Development only — never ship this |

Wrong `transport` / missing `appGroup` **fails plugin init** with a concrete message (empty widgets from a silent fallback are not a thing).

Override without editing conf: `WIDGET_TRANSPORT=widgetContainer`.

The **widget extension** still reads all channels and picks the freshest map (so it can find data wherever the host wrote). Host-side writes use only the configured driver. Render receipts feed `getWidgetDiagnostics`, not transport selection.

`setItems` skips disk I/O when the value is unchanged (no nonce bump).

#### Code Signing

`build-widget.sh` signs the `.appex` before bundling. Identity resolution:

1. `WIDGET_SIGN_IDENTITY`
2. `APPLE_SIGNING_IDENTITY`
3. Fallback: `-` (ad-hoc)

```bash
security find-identity -v -p codesigning
```

| Signing | Widget visible | Recommended `transport` | Distribution |
|---------|---------------|-------------------------|--------------|
| Ad-hoc (`-`) | Yes | `widgetContainer` | Local only |
| Apple Development | Yes | `appGroup` | Local + TestFlight |
| Developer ID | Yes | `appGroup` | Direct distribution |

**Important:** The main app's `App.entitlements` should **not** include `com.apple.security.app-sandbox` when using `widgetContainer` (host must write into the extension container). The widget extension is always sandboxed (required by WidgetKit).

#### Debugging the widget separately

```bash
cd src-tauri/macos-widget
xcodegen generate
open TauriWidgetExtension.xcodeproj
```

Select the widget scheme in Xcode, set your app as the Host Application, and run with breakpoints.

---

### Android Setup

#### Step 1: Initialize

```bash
pnpm tauri android init
```

The plugin automatically registers its Android bridge and Glance widget receiver.

**No Kotlin code required** — the built-in receiver reads widget config from shared storage and renders it via Jetpack Glance.

#### Step 2: Configure the App Group (optional)

By default, the plugin uses the application package name as the `SharedPreferences` group. To use a custom group, add a `<meta-data>` tag in your app's `AndroidManifest.xml`:

```xml
<receiver
    android:name="git.s00d.widgets.TauriGlanceWidgetReceiver"
    tools:replace="android:label"
    android:label="My Widget">
    <meta-data
        android:name="tauri_widget_group"
        android:value="group.com.example.myapp" />
</receiver>
```

#### Step 3: Run

```bash
pnpm tauri android dev
```

#### Step 4: Add widget to home screen

Long-press the home screen → **Widgets** → find your app → drag to screen.

#### Custom Widget Provider (advanced)

If you need custom behavior, create your own `GlanceAppWidget` / `GlanceAppWidgetReceiver` and keep the same shared group contract.

---

### Desktop Setup (Windows / Linux / macOS host)

Desktop hosts can show widgets as **frameless transparent webview windows** that render the JSON config via `widget.html` (built-in `widgetview` protocol).

**Windows** also supports **Widgets Board** via Adaptive Cards (see below). **Linux** pins webview windows with X11 `_NET_WM_WINDOW_TYPE_DESKTOP` when the `linux` feature is on (Wayland: optional `layer-shell`).

#### Option A: Declarative (tauri.conf.json)

```json
{
  "app": {
    "windows": [
      { "label": "main", "title": "My App", "width": 800, "height": 600 },
      {
        "label": "desktop-widget",
        "url": "/widget.html",
        "width": 300, "height": 150,
        "decorations": false, "transparent": true,
        "alwaysOnBottom": true, "skipTaskbar": true,
        "visible": false, "resizable": false
      }
    ]
  }
}
```

#### Option B: Dynamic (TypeScript)

```typescript
import { createWidgetWindow, closeWidgetWindow } from "tauri-plugin-widgets-api";

await createWidgetWindow({
  label: "weather",
  width: 280,
  height: 200,
  x: 80,
  y: 80,
  skipTaskbar: true,
  group: "group.com.example.myapp",
  widgetId: "weather",
  size: "small",
});

await closeWidgetWindow("weather");
```

Pass `group`, `widgetId`, and `size` so the built-in renderer loads the right config.

#### Windows Widgets Board (Adaptive Cards)

```bash
npx tauri-plugin-widgets-api init-windows
```

This scaffolds a C# `IWidgetProvider` under `templates/windows-widget/` (details: [`docs/windows-surfaces.md`](docs/windows-surfaces.md)). On `setWidgetConfig` the plugin writes Adaptive Cards JSON under `ac:template:{widgetId}` / `ac:data:{widgetId}`.

Optional wallpaper parenting (not Widgets Board): enable Cargo feature `workerw`.

---

## Widget Config Schema

The `WidgetConfig` object defines layouts per widget size family:

```typescript
interface WidgetConfig {
  version?: number;
  small?: WidgetElement;
  medium?: WidgetElement;
  large?: WidgetElement;
}
```

### Layout Containers

#### `vstack` — Vertical Stack

```json
{
  "type": "vstack",
  "spacing": 8,
  "alignment": "leading",
  "padding": 12,
  "background": "#1a1a2e",
  "cornerRadius": 12,
  "children": [...]
}
```

| Property | Type | Description |
|----------|------|-------------|
| `children` | `WidgetElement[]` | Child elements |
| `spacing` | `number` | Space between children (pt) |
| `alignment` | `"leading" \| "center" \| "trailing"` | Horizontal alignment |

#### `hstack` — Horizontal Stack

```json
{
  "type": "hstack",
  "spacing": 10,
  "alignment": "center",
  "children": [...]
}
```

| Property | Type | Description |
|----------|------|-------------|
| `children` | `WidgetElement[]` | Child elements |
| `spacing` | `number` | Space between children (pt) |
| `alignment` | `"top" \| "center" \| "bottom"` | Vertical alignment |

#### `zstack` — Overlay Stack

```json
{
  "type": "zstack",
  "alignment": "center",
  "children": [...]
}
```

Children are layered on top of each other. Uses `FrameLayout` on Android.

#### `grid` — Grid Layout

```json
{
  "type": "grid",
  "columns": 2,
  "spacing": 8,
  "rowSpacing": 8,
  "children": [...]
}
```

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `columns` | `number` | `2` | Number of columns |
| `spacing` | `number` | — | Column spacing |
| `rowSpacing` | `number` | — | Row spacing |

#### `container` — Single-Child Wrapper (Box)

A single-child wrapper for badges/avatars/overlays. For Android Glance stability, avoid long chains of nested `container` wrappers when a styled `vstack` is enough.

```json
{
  "type": "container",
  "contentAlignment": "center",
  "padding": 12,
  "background": { "light": "#F0F0FF", "dark": "#1C1C1E" },
  "cornerRadius": 16,
  "clipShape": "circle",
  "children": [
    { "type": "text", "content": "AK", "fontSize": 20, "fontWeight": "bold", "color": "#ffffff" }
  ]
}
```

| Property | Type | Description |
|----------|------|-------------|
| `children` | `WidgetElement[]` | Child elements (typically one) |
| `contentAlignment` | `string` | Content alignment: `"center"`, `"topLeading"`, `"bottomTrailing"`, etc. |

> **Platform mapping:** SwiftUI `ZStack`, Android Glance `Box`, HTML `div` with flexbox.

### Leaf Elements

#### `text`

```json
{
  "type": "text",
  "content": "Hello World",
  "fontSize": 18,
  "fontWeight": "bold",
  "fontDesign": "rounded",
  "textStyle": "headline",
  "color": "label",
  "alignment": "center",
  "lineLimit": 2
}
```

| Property | Type | Description |
|----------|------|-------------|
| `content` | `string` | Text to display |
| `fontSize` | `number` | Font size in points (overridden by `textStyle` if set) |
| `fontWeight` | `FontWeight` | `ultralight` `thin` `light` `regular` `medium` `semibold` `bold` `heavy` `black` |
| `fontDesign` | `FontDesign` | `default` `monospaced` `rounded` `serif` |
| `textStyle` | `TextStyle` | Semantic text style (see below) — respects Dynamic Type / accessibility |
| `color` | `ColorValue` | Hex color, semantic name, or adaptive `{ light, dark }` object |
| `alignment` | `TextAlignment` | `leading` `center` `trailing` |
| `lineLimit` | `number` | Max number of lines |

**TextStyle values:** `largeTitle`, `title`, `title2`, `title3`, `headline`, `subheadline`, `body`, `callout`, `footnote`, `caption`, `caption2`

> When `textStyle` is set, it determines font size semantically based on platform settings (Dynamic Type on iOS, accessibility on Android). This ensures widgets remain readable for users with vision accessibility needs.

> **Android:** Bold/heavy/semibold font weights are rendered via `SpannableString` with `StyleSpan`.

#### `image`

```json
{
  "type": "image",
  "systemName": "cloud.sun.fill",
  "size": 32,
  "color": "#ffcc00"
}
```

| Property | Type | Description |
|----------|------|-------------|
| `systemName` | `string` | SF Symbol name (Apple) / drawable resource name (Android) |
| `data` | `string` | Base64-encoded image data (with or without `data:image/...;base64,` prefix) |
| `url` | `string` | Remote image URL |
| `size` | `number` | Display size in points |
| `color` | `string` | Tint color (hex) |
| `contentMode` | `"fit" \| "fill"` | How image fills its frame |

> **Android:** Base64 images are decoded to `Bitmap` and displayed in a real `ImageView`. System names map to drawable resources.

#### `progress`

```json
{
  "type": "progress",
  "value": 0.7,
  "total": 1.0,
  "tint": "#4CAF50",
  "label": "Steps",
  "barStyle": "linear"
}
```

#### `gauge`

```json
{
  "type": "gauge",
  "value": 0.72,
  "min": 0,
  "max": 1,
  "label": "CPU",
  "currentValueLabel": "72%",
  "tint": "#7aa2f7",
  "gaugeStyle": "circular"
}
```

#### `chart`

```json
{
  "type": "chart",
  "chartType": "bar",
  "tint": "#89b4fa",
  "chartData": [
    { "label": "Mon", "value": 120 },
    { "label": "Tue", "value": 180, "color": "#a6e3a1" }
  ]
}
```

Supported types: `bar`, `line`, `area`, `pie`.

#### `list` — Collection List

```json
{
  "type": "list",
  "spacing": 4,
  "items": [
    { "text": "Buy groceries", "checked": true, "action": "task_buy" },
    { "text": "Call dentist", "checked": false, "action": "task_call", "payload": "id=42" }
  ]
}
```

| Property | Type | Description |
|----------|------|-------------|
| `items` | `ListItem[]` | List rows with text and optional checked/action/payload |
| `spacing` | `number` | Space between rows |

`ListItem`:

| Field | Type | Description |
|-------|------|-------------|
| `text` | `string` | Row label |
| `checked` | `boolean` | Optional checked marker |
| `action` | `string` | Optional action emitted as `widget-action` |
| `payload` | `string` | Optional payload for `widget-action` |

> Platform notes:
> - **Android:** rendered via Glance list/lazy composition.
> - **iOS/macOS:** rendered as SwiftUI row list.
> - **Desktop:** rendered as HTML row list.

#### `button`

```json
{
  "type": "button",
  "label": "Refresh",
  "action": "refresh_data",
  "backgroundColor": "#4CAF50",
  "color": "#ffffff"
}
```

| Property | Type | Description |
|----------|------|-------------|
| `label` | `string` | Button text |
| `url` | `string` | Deep-link URL |
| `action` | `string` | Action identifier — emits `widget-action` event |
| `backgroundColor` | `string` | Background color (hex) |
| `color` | `string` | Text color (hex) |

Listen for actions:

```typescript
import { onWidgetAction } from "tauri-plugin-widgets-api";

await onWidgetAction((data) => {
  console.log("Action:", data.action, data.payload);
});
```

#### `toggle`

```json
{ "type": "toggle", "isOn": true, "label": "Dark Mode", "tint": "#4CAF50" }
```

#### `divider`

```json
{ "type": "divider", "color": "#333333", "thickness": 1 }
```

#### `spacer`

```json
{ "type": "spacer", "minLength": 10 }
```

#### `date`

```json
{ "type": "date", "date": "2026-03-01T10:00:00Z", "dateStyle": "relative", "fontSize": 14, "color": "#ffffff" }
```

| `dateStyle` | Output Example |
|-------------|----------------|
| `time` | `10:00 AM` |
| `date` | `Mar 1, 2026` |
| `relative` | `in 2 days` |
| `offset` | `+2 days` |
| `timer` | `48:00:00` |

#### `link` — Tappable Wrapper

Wraps any child content and makes it tappable:

```json
{
  "type": "link",
  "action": "open_profile",
  "children": [
    { "type": "text", "content": "Tap me!", "fontSize": 16, "color": "#60a5fa" }
  ]
}
```

#### `shape`

```json
{ "type": "shape", "shapeType": "circle", "fill": "#ef4444", "size": 12 }
```

Supported shapes: `circle`, `capsule`, `rectangle`.

#### `timer`

```json
{
  "type": "timer",
  "targetDate": "2026-12-31T23:59:59Z",
  "counting": "down",
  "fontSize": 24,
  "fontWeight": "bold",
  "color": "#ffffff"
}
```

> **iOS/macOS:** Uses `Text(date, style: .timer)` which updates natively every second without consuming widget refresh budget.

#### `label` — Icon + Text

```json
{
  "type": "label",
  "text": "5 new messages",
  "systemName": "envelope.fill",
  "iconColor": "#60a5fa",
  "fontSize": 14,
  "color": "#e2e8f0"
}
```

#### `canvas` — Declarative Drawing

```json
{
  "type": "canvas",
  "width": 120,
  "height": 120,
  "elements": [
    { "draw": "circle", "cx": 60, "cy": 60, "r": 55, "fill": "#1e293b", "stroke": "#475569", "strokeWidth": 2 },
    { "draw": "line", "x1": 60, "y1": 60, "x2": 60, "y2": 20, "stroke": "#f1f5f9", "strokeWidth": 3, "lineCap": "round" },
    { "draw": "circle", "cx": 60, "cy": 60, "r": 4, "fill": "#ef4444" }
  ]
}
```

Draw commands: `circle`, `line`, `rect`, `arc`, `text`, `path`.

> **Android:** Canvas bitmaps are capped at 512px and compressed to stay under the 500KB Binder IPC limit, preventing `TransactionTooLargeException`.

### Common Style Properties

All elements support:

```json
{
  "padding": 12,
  "background": "#1a1a2e",
  "cornerRadius": 8,
  "opacity": 0.9,
  "frame": { "width": 100, "height": 50, "maxWidth": "infinity" },
  "border": { "color": "#333333", "width": 1 },
  "shadow": { "color": "#000000", "radius": 4, "x": 0, "y": 2 },
  "clipShape": "circle",
  "flex": 1
}
```

#### `clipShape` — Content Masking

Clips the element's content to a shape. Useful for circular avatars, pill-shaped badges, etc.

| Value | Description |
|-------|-------------|
| `"circle"` | Perfect circle (50% border-radius) |
| `"capsule"` | Pill shape (fully rounded ends) |
| `"rectangle"` | Rectangle with `cornerRadius` applied |

> **Platform mapping:** SwiftUI `.clipShape()`, CSS `border-radius` + `overflow: hidden`, Android Glance shape clipping.

#### `flex` — Proportional Layout

The `flex` property allows elements to occupy available space proportionally within stacks, analogous to CSS `flex`, Android `layout_weight`, or SwiftUI `layoutPriority`.

```json
{
  "type": "hstack",
  "children": [
    { "type": "text", "content": "Left", "flex": 1 },
    { "type": "text", "content": "Center (wider)", "flex": 2 },
    { "type": "text", "content": "Right", "flex": 1 }
  ]
}
```

#### Adaptive Colors (Dark Mode Support)

All color properties (`color`, `tint`, `fill`, `stroke`, `backgroundColor`, `iconColor`, `background`) support three formats:

**1. Hex string** (static):
```json
{ "color": "#FF5733" }
```

**2. Semantic color name** (auto-adapts to system theme):
```json
{ "color": "label" }
```

Available semantic colors: `label`, `secondaryLabel`, `tertiaryLabel`, `systemBackground`, `secondarySystemBackground`, `separator`, `accent`, `systemRed`, `systemGreen`, `systemBlue`, `systemOrange`, `systemYellow`, `systemPurple`, `systemPink`, `systemGray`

**3. Adaptive object** (explicit light/dark values):
```json
{ "color": { "light": "#000000", "dark": "#FFFFFF" } }
```

The `background` property additionally supports gradients:

```json
{
  "background": { "light": "#FFFFFF", "dark": "#1C1C1E" }
}
```

#### Gradient Backgrounds

```json
{
  "background": {
    "gradientType": "linear",
    "colors": ["#667eea", "#764ba2"],
    "direction": "topToBottom"
  }
}
```

Types: `linear`, `radial`, `angular`.

---

## Widget Updater

For periodic updates (clocks, dashboards):

```typescript
import { startWidgetUpdater } from "tauri-plugin-widgets-api";

const stop = await startWidgetUpdater(
  () => ({
    small: {
      type: "vstack", padding: 12, background: "#1a1a2e",
      children: [
        { type: "text", content: new Date().toLocaleTimeString(),
          fontSize: 32, fontWeight: "bold", color: "#fff" },
      ],
    },
  }),
  "group.com.example.myapp",
  "clock",
  {
    intervalMs: 60_000,
    reload: true,
    onAction: (action, payload) => {
      console.log("Widget action:", action, payload);
    },
  },
);

// Later:
stop();
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `intervalMs` | `number` | `5000` | Update interval (ms). Use `60000+` for native WidgetKit hosts. |
| `immediate` | `boolean` | `true` | Run builder immediately on start |
| `reload` | `boolean` | `false` | Call `reloadAllTimelines()` after each tick. Throttled by `TAURI_WIDGET_MIN_RELOAD_SECS` on iOS/macOS. |
| `onAction` | `function` | — | Subscribe to `widget-action` events |

> **Important:** Apple enforces a daily widget reload budget (~40-70 reloads/day). The Rust backend throttle is configurable via `TAURI_WIDGET_MIN_RELOAD_SECS`.
>
> Defaults:
> - **Debug:** `0` (no plugin-side throttle)
> - **Release:** `900` (15 minutes)
>
> Examples:
> - Disable plugin-side throttle: `TAURI_WIDGET_MIN_RELOAD_SECS=0`
> - Set custom throttle: `TAURI_WIDGET_MIN_RELOAD_SECS=5`
>
> Even with `0`, WidgetKit may still coalesce/defer refreshes — this is a platform-level limit. For second-by-second UI, prefer native timer/date styles (for example, `{ type: "timer" }`) instead of frequent reload calls.

---

## Low-Level Data API

```typescript
import { setItems, getItems, reloadAllTimelines } from "tauri-plugin-widgets-api";

await setItems("temperature", "72", "group.com.example.myapp");
const temp = await getItems("temperature", "group.com.example.myapp");
await reloadAllTimelines();
```

### API Reference

| Function | Description |
|----------|-------------|
| `setItems(key, value, group)` | Store a key-value pair |
| `getItems(key, group)` | Read a stored value |
| `setWidgetConfig(config, group, widgetId, skipReload?)` | Send a full UI config |
| `getWidgetConfig(group, widgetId)` | Read the current UI config |
| `setRegisterWidget(widgets)` | Register widget provider class names |
| `reloadAllTimelines()` | Reload all widget timelines |
| `reloadTimelines(ofKind)` | Reload a specific widget kind |
| `requestWidget()` | Pin a widget (Android only) |
| `createWidgetWindow(config)` | Create a desktop widget window |
| `closeWidgetWindow(label)` | Close a desktop widget window |
| `widgetAction(action, payload?)` | Emit a `widget-action` event |
| `onWidgetAction(callback)` | Listen for `widget-action` events |
| `startWidgetUpdater(builder, group, widgetId, options?)` | Periodic config updater |

---

## Rust API

```rust
use tauri::Manager;
use tauri_plugin_widgets::WidgetExt;

fn update_widget(app: &tauri::AppHandle) {
    let widget = app.widget();
    widget.set_items("key", "value", "group").unwrap();
    widget.reload_all_timelines().unwrap();
}
```

---

## Development

Architecture, repo layout, golden / visual tests, Linux harness, codegen:

→ **[`docs/development.md`](docs/development.md)**

---

## Troubleshooting

### Widget shows "No configuration" (macOS)

- Call `setWidgetConfig(...)` from your app before adding the widget.
- Verify the app is signed with a real certificate (`security find-identity -v -p codesigning`), not ad-hoc.
- Check that the app is **not sandboxed** (`App.entitlements` should not contain `com.apple.security.app-sandbox`).
- Verify the widget's container has the data file:
  ```bash
  ls ~/Library/Containers/<your-bundle-id>.widgetkit/Data/widget_data.json
  ```
- Check widget logs: `log show --last 1m --predicate 'subsystem == "com.tauri.widgets"' --style compact`
- Ensure `TauriWidgetExtension.entitlements` contains `com.apple.security.app-sandbox` and the correct App Group.

### Widget shows "No configuration" (iOS)

- Ensure the App Group identifier is **identical** in the main app and widget extension (Xcode → Signing & Capabilities → App Groups).
- Call `setWidgetConfig(...)` from your app before adding the widget.

### Widget doesn't update (iOS/macOS)

- Apple limits widget refreshes to ~40-70 per day.
- Check plugin throttle: `TAURI_WIDGET_MIN_RELOAD_SECS` (Debug default `0`, Release default `900`).
- For testing: in Xcode, use **Debug → Simulate Timeline → After Refresh**.
- For live counters, use `{ type: "timer" }` instead of frequent reloads.

### Android widget is empty or crashes

- Check `adb logcat | grep -i widget` for errors.
- Canvas bitmaps larger than 512px are automatically capped.
- Bitmap data is compressed to stay under the Binder IPC 500KB limit.

### Desktop widget window not appearing

- Call `createWidgetWindow(...)` (or define a window in `tauri.conf.json`).
- Allow the window label in capabilities (`widgets:default` / window permissions).
- On Linux, enable the `linux` feature for X11 DESKTOP pinning (see [`docs/linux-harness.md`](docs/linux-harness.md)).
- On Windows Widgets Board, run `init-windows` and verify `ac:template:*` after `setWidgetConfig`.

---

## License

MIT
