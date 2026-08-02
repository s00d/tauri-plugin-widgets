[![Platform](https://img.shields.io/badge/platform-Android%20%7C%20iOS%20%7C%20macOS%20%7C%20Windows%20%7C%20Linux-blue?style=for-the-badge)](https://s00d.github.io/tauri-plugin-widgets/)
[![npm version](https://img.shields.io/npm/v/tauri-plugin-widgets-api/latest?style=for-the-badge)](https://www.npmjs.com/package/tauri-plugin-widgets-api)
[![Crates.io](https://img.shields.io/crates/v/tauri-plugin-widgets?style=for-the-badge)](https://crates.io/crates/tauri-plugin-widgets)
[![Documentation](https://img.shields.io/badge/docs-s00d.github.io-blue?style=for-the-badge)](https://s00d.github.io/tauri-plugin-widgets/)
[![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)](https://github.com/s00d/tauri-plugin-widgets/blob/main/LICENSE)
[![GitHub release](https://img.shields.io/github/v/release/s00d/tauri-plugin-widgets?style=for-the-badge)](https://github.com/s00d/tauri-plugin-widgets/releases)
[![GitHub downloads](https://img.shields.io/github/downloads/s00d/tauri-plugin-widgets/total?style=for-the-badge)](https://github.com/s00d/tauri-plugin-widgets/releases)
[![GitHub issues](https://img.shields.io/badge/github-issues-orange?style=for-the-badge)](https://github.com/s00d/tauri-plugin-widgets/issues)
[![GitHub stars](https://img.shields.io/badge/github-stars-yellow?style=for-the-badge)](https://github.com/s00d/tauri-plugin-widgets/stargazers)
[![Donate](https://img.shields.io/badge/Donate-Donationalerts-ff4081?style=for-the-badge)](https://www.donationalerts.com/r/s00d88)

<p align="center">
  <img src="assets/readme-banner.jpg" alt="tauri-plugin-widgets — one JSON config, five native surfaces" width="100%" />
</p>

# Tauri Plugin Widgets

Native widgets for **Android**, **iOS**, **macOS**, **Windows**, and **Linux** from one declarative JSON configuration.

**Full documentation** (schema, platform setup, elements, showcase):  
[https://s00d.github.io/tauri-plugin-widgets](https://s00d.github.io/tauri-plugin-widgets/)

> The complete config schema, platform setup guides, and element reference live on the documentation site — they are no longer maintained in this README.

## Install

```bash
# any package manager — or: cargo tauri add tauri-plugin-widgets
pnpm tauri add tauri-plugin-widgets
```

Manual alternative:

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-widgets = "0.4"
```

```bash
pnpm add tauri-plugin-widgets-api
```

## Example

```typescript
import { setWidgetConfig } from "tauri-plugin-widgets-api";

await setWidgetConfig(
  {
    small: {
      type: "vstack",
      padding: 12,
      background: "#1a1a2e",
      children: [
        { type: "text", content: "72°", fontSize: 36, fontWeight: "bold", color: "#fff" },
        { type: "progress", value: 0.7, tint: "#4CAF50", label: "Humidity" },
      ],
    },
  },
  "group.com.example.myapp",
  "weather",
);
```

Register the plugin with `tauri_plugin_widgets::init()` and add `widgets:default` to capabilities. Details: [Install](https://s00d.github.io/tauri-plugin-widgets/guide/install) · [First widget](https://s00d.github.io/tauri-plugin-widgets/guide/first-widget).

## Platform setup

| Platform | Docs |
| --- | --- |
| Android | [Setup](https://s00d.github.io/tauri-plugin-widgets/guide/setup/android) |
| iOS | [Setup](https://s00d.github.io/tauri-plugin-widgets/guide/setup/ios) |
| macOS | [Setup](https://s00d.github.io/tauri-plugin-widgets/guide/setup/macos) |
| Windows | [Setup](https://s00d.github.io/tauri-plugin-widgets/guide/setup/windows) |
| Linux | [Setup](https://s00d.github.io/tauri-plugin-widgets/guide/setup/linux) |
| Desktop webview | [Setup](https://s00d.github.io/tauri-plugin-widgets/guide/setup/desktop) |

Quick scaffold: `npx tauri-plugin-widgets-api init-macos|init-ios|init-windows`.

## Contributing

Architecture, goldens, harnesses: [docs/contributing/development.md](docs/contributing/development.md).

## License

[MIT](LICENSE)
