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

Then finish [Register the plugin](#register-the-plugin) and [Permissions](#permissions) if the CLI did not wire them for your project layout.

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

For Apple transport / App Group, pass config via `tauri.conf.json` — see [Plugin config](/api/plugin-config) and [Transport](/guide/transport).

## Permissions

`src-tauri/capabilities/default.json`:

```json
{
  "permissions": [
    "core:default",
    "widgets:default"
  ]
}
```

Full list: [Permissions](/api/permissions).

## Scaffold native extensions

```bash
npx tauri-plugin-widgets-api init-macos
npx tauri-plugin-widgets-api init-ios
npx tauri-plugin-widgets-api init-windows
```

Next: [First widget](/guide/first-widget) (desktop, ~10 minutes) or [Platform setup](/guide/setup/).
