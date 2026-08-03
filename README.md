[![Platform](https://img.shields.io/badge/platform-Android%20%7C%20iOS%20%7C%20macOS%20%7C%20Windows%20%7C%20Linux-blue?style=for-the-badge)](https://s00d.github.io/tauri-plugin-widgets/)
[![npm version](https://img.shields.io/npm/v/tauri-plugin-widgets-api/latest?style=for-the-badge)](https://www.npmjs.com/package/tauri-plugin-widgets-api)
[![Crates.io](https://img.shields.io/crates/v/tauri-plugin-widgets?style=for-the-badge)](https://crates.io/crates/tauri-plugin-widgets)
[![Documentation](https://img.shields.io/badge/docs-s00d.github.io-blue?style=for-the-badge)](https://s00d.github.io/tauri-plugin-widgets/)
[![docs.rs](https://img.shields.io/badge/docs-docs.rs-blue?style=for-the-badge)](https://docs.rs/tauri-plugin-widgets)
[![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)](https://github.com/s00d/tauri-plugin-widgets/blob/main/LICENSE)
[![GitHub release](https://img.shields.io/github/v/release/s00d/tauri-plugin-widgets?style=for-the-badge)](https://github.com/s00d/tauri-plugin-widgets/releases)
[![GitHub downloads](https://img.shields.io/github/downloads/s00d/tauri-plugin-widgets/total?style=for-the-badge)](https://github.com/s00d/tauri-plugin-widgets/releases)
[![GitHub issues](https://img.shields.io/github/issues/s00d/tauri-plugin-widgets?style=for-the-badge)](https://github.com/s00d/tauri-plugin-widgets/issues)
[![GitHub stars](https://img.shields.io/github/stars/s00d/tauri-plugin-widgets?style=for-the-badge)](https://github.com/s00d/tauri-plugin-widgets/stargazers)
[![Donate](https://img.shields.io/badge/Donate-Donationalerts-ff4081?style=for-the-badge)](https://www.donationalerts.com/r/s00d88)

<p align="center">
  <img src="assets/readme-banner.jpg" alt="tauri-plugin-widgets — one JSON config, five native surfaces" width="100%" />
</p>

# Tauri Plugin Widgets

Native widgets for **Android**, **iOS**, **macOS**, **Windows**, and **Linux** from one declarative JSON configuration.

**Full documentation:** [https://s00d.github.io/tauri-plugin-widgets](https://s00d.github.io/tauri-plugin-widgets/)

---

## Two CLIs (read this first)

| | **Using the plugin in your app** | **Developing this repository** |
|---|---|---|
| Entry | `npx tauri-widgets` / `pnpm tauri-widgets` | `pnpm -C scripts cli` / `pnpm hosts` |
| Code | published `dist-cli/` (from `src-cli/`) | `scripts/` (TypeScript + `scripts/sh`) |
| Examples | `init`, `preview`, `signing`, `doctor`, `validate`, `trace`, `clean` | `hosts up`, `shot`, `test`, `docs-generate`, `triage` |

If you are building an app: ignore `pnpm test:android:visual` and the rest of the root `test:*` / `*-up` scripts — those are for plugin maintainers.

Maintainer map: [`scripts/README.md`](scripts/README.md) · [Contributing](docs/contributing/development.md).

---

## Install

```bash
pnpm tauri add tauri-plugin-widgets
```

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-widgets = "0.5"
```

```bash
pnpm add tauri-plugin-widgets-api
```

## Consumer CLI (app authors)

```bash
npx tauri-widgets init                 # detect platforms → init-macos / ios / windows
npx tauri-widgets preview ./widget.json --size medium --watch
npx tauri-widgets signing              # identities + transport verdict
npx tauri-widgets signing --apply      # write plugins.widgets
npx tauri-widgets validate ./widget.json --platforms ios,android
npx tauri-widgets doctor
npx tauri-widgets trace --follow
npx tauri-widgets clean --group group.com.example.app
npx tauri-widgets clean --cache
```

Preview needs **no native build** — edits to the JSON reload in the browser.

JSON Schema (IDE autocomplete):

```json
{
  "$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json"
}
```

## Example

```typescript
import { setWidgetConfig } from "tauri-plugin-widgets-api";

await setWidgetConfig(
  {
    $schema: "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json",
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

Register `tauri_plugin_widgets::init()` and `widgets:default` in capabilities. Details: [Install](https://s00d.github.io/tauri-plugin-widgets/guide/install) · [First widget](https://s00d.github.io/tauri-plugin-widgets/guide/first-widget).

## Platform setup

| Platform | Docs |
| --- | --- |
| Android | [Setup](https://s00d.github.io/tauri-plugin-widgets/guide/setup/android) |
| iOS | [Setup](https://s00d.github.io/tauri-plugin-widgets/guide/setup/ios) |
| macOS | [Setup](https://s00d.github.io/tauri-plugin-widgets/guide/setup/macos) |
| Windows | [Setup](https://s00d.github.io/tauri-plugin-widgets/guide/setup/windows) |
| Linux | [Setup](https://s00d.github.io/tauri-plugin-widgets/guide/setup/linux) |
| Desktop webview | [Setup](https://s00d.github.io/tauri-plugin-widgets/guide/setup/desktop) |

Or: `npx tauri-widgets init` / `init-macos` / `init-ios` / `init-windows`.

## Try from zero

Minimal widget JSON + preview (no Tauri app required for layout):

```bash
# clone / degit this folder
npx degit s00d/tauri-plugin-widgets/templates/starter my-widget
cd my-widget
npx tauri-widgets preview ./widget.json --watch --open
```

Full app: use the [example](examples/tauri-plugin-widgets-example) or `pnpm tauri add` then `npx tauri-widgets init`.

## Developing this plugin (maintainers)

```bash
pnpm hosts status
pnpm hosts up ios android
pnpm shot weather.small linux
pnpm cli:test -- --platform desktop
pnpm docs:generate
```

See [`scripts/README.md`](scripts/README.md).

## License

[MIT](LICENSE)
