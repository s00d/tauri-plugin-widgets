---
title: Install
---

# Install

<div class="doc-illust">

![Install steps: Cargo, npm, plugin init](/illustrations/install.jpg)

</div>

## Quick install (recommended)

Use the Tauri CLI — it adds the Rust crate and the JS package in one step. Works with any package manager, and with `cargo tauri` too:

```bash
# npm
npm run tauri add tauri-plugin-widgets

# pnpm
pnpm tauri add tauri-plugin-widgets

# yarn
yarn tauri add tauri-plugin-widgets

# bun
bun tauri add tauri-plugin-widgets

# cargo (no Node script required)
cargo tauri add tauri-plugin-widgets
```

Then finish [Register the plugin](#register-the-plugin), [Permissions](#permissions), and (on Apple hosts) [Plugin config](#apple-hosts-plugin-config) if the CLI did not wire them for your project layout.

Optional Cargo features: [Cargo features](/api/cargo-features).

## Manual install

### Cargo

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-widgets = "0.4"
```

Or:

```bash
cd src-tauri && cargo add tauri-plugin-widgets
```

### JavaScript package

```bash
pnpm add tauri-plugin-widgets-api
# npm i tauri-plugin-widgets-api
# yarn add tauri-plugin-widgets-api
# bun add tauri-plugin-widgets-api
```

## Register the plugin

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_widgets::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Apple hosts: plugin config

`init-macos` / `init-ios` do **not** write `plugins.widgets` for you. Without it, **plugin init fails** on macOS and iOS (`plugins.widgets.appGroup is required`).

Add to `src-tauri/tauri.conf.json` (use your real App Group id):

```json
{
  "plugins": {
    "widgets": {
      "appGroup": "group.com.example.myapp",
      "transport": "appGroup"
    }
  }
}
```

| Host | Notes |
| --- | --- |
| **iOS** | `transport` must be `appGroup` (or omit / `auto` → appGroup). |
| **macOS** local ad-hoc | Prefer `"transport": "widgetContainer"` until you have a Team ID + App Groups. |
| **macOS** Team ID / MAS | `"transport": "appGroup"`. |

Details: [Plugin config](/api/plugin-config) · [Transport](/guide/transport).

Linux / Windows desktop webview do not need this block for the built-in window path.

## Permissions

`src-tauri/capabilities/default.json` must list **every window label** that calls plugin commands — including desktop widget labels from `createWidgetWindow`:

```json
{
  "identifier": "default",
  "windows": ["main", "weather"],
  "permissions": [
    "core:default",
    "widgets:default"
  ]
}
```

Use `"*"` only if you intentionally allow all labels. Missing the widget label → the embedded renderer cannot call `getWidgetConfig` and the window stays empty.

Full list: [Permissions](/api/permissions).

## Scaffold native extensions

These copy templates into **your** app under `src-tauri/` (they do not leave files only inside `node_modules`):

```bash
npx tauri-plugin-widgets-api init-macos    # → src-tauri/macos-widget/
npx tauri-plugin-widgets-api init-ios     # syncs gen/apple widget Swift
npx tauri-plugin-widgets-api init-windows # → src-tauri/windows-widget/
```

Still add [Apple plugin config](#apple-hosts-plugin-config) yourself. Guides: [Platform setup](/guide/setup/).

Next: [First widget](/guide/first-widget) (desktop) or [Platform setup](/guide/setup/).
