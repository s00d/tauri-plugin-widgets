---
title: macOS setup
---

# macOS setup

macOS widgets require a "sidecar" Xcode project — a small project alongside your Tauri app that compiles the Widget Extension (`.appex`). After `init-macos`, a normal **`pnpm tauri build`** produces an `.app` (and DMG) with `Contents/PlugIns/*.appex` already inside.

> **Important:** On macOS, WidgetKit picks up extension widgets from the installed app bundle.  
> Addable widgets appear only after you **build** the app and move the resulting `.app` to **`/Applications`** (or install from the DMG).

**Requirements:** `xcodegen` (`brew install xcodegen`)

## Step 1: Initialize the widget project (automatic)

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

## Step 2: Configure tauri.conf.json

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

## Step 3: Build

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

## Apple data transport

See the dedicated guide: [Apple data transport](/guide/transport).

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

## Code Signing

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

## Debugging the widget separately

```bash
cd src-tauri/macos-widget
xcodegen generate
open TauriWidgetExtension.xcodeproj
```

Select the widget scheme in Xcode, set your app as the Host Application, and run with breakpoints.

---

## When it fails

### Widget shows "No configuration"

- Call `setWidgetConfig(...)` from your app before adding the widget.
- Verify the app is signed with a real certificate (`security find-identity -v -p codesigning`), not ad-hoc.
- Check that the app is **not sandboxed** (`App.entitlements` should not contain `com.apple.security.app-sandbox`) when using `widgetContainer`.
- Verify the widget's container has the data file:
  ```bash
  ls ~/Library/Containers/<your-bundle-id>.widgetkit/Data/widget_data.json
  ```
- Check widget logs: `log show --last 1m --predicate 'subsystem == "com.tauri.widgets"' --style compact`
- Ensure `TauriWidgetExtension.entitlements` contains `com.apple.security.app-sandbox` and the correct App Group.

### Widget doesn't update

- Apple limits widget refreshes to ~40–70 per day.
- Check plugin throttle on **iOS** (`TAURI_WIDGET_MIN_RELOAD_SECS`; Debug `0`, Release `900`). On the macOS app host this env is unused — inspect `ApplyOutcome.reload` from `setWidgetConfig`.
- Prefer `{ type: "timer" }` for live UI.

More symptoms: [Troubleshooting index](/guide/troubleshooting). Transport: [Apple data transport](/guide/transport).
