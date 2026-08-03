---
title: CLI
---

# Consumer CLI (`tauri-widgets`)

Published with the npm package. **Not** the same as maintainer `pnpm -C scripts cli` (stands / goldens / docs).

```bash
npx tauri-widgets <command>
# or: pnpm tauri-widgets <command>
```

| Command | Purpose |
| --- | --- |
| `init` | Detect platforms → run `init-macos` / `init-ios` / `init-windows` |
| `init-macos` / `init-ios` / `init-windows` | Scaffold native extension |
| `preview <config.json>` | Browser preview of `widget.html` (`--watch`, `--size`, `--open`) |
| `signing` | Codesign identities + transport verdict (`--apply`, `--write-entitlements`, `--dev-cert`) |
| `validate <config.json>` | Schema shape + `capabilities.json` per platform |
| `doctor [app-root]` | Transport / entitlements / identities / trace / plugin-config |
| `trace` | Print / `--follow` `widget_trace.json` |
| `clean` | Clear store/trace (`--group`) or `--cache` (image prefetch) |

## Preview

```bash
npx tauri-widgets preview ./widget.json --size medium --watch --open
```

No Xcode / Gradle build — JSON edits reload over SSE.

## Validate

```bash
npx tauri-widgets validate ./widget.json --platforms ios,android,windows
```

Uses `schemas/capabilities.json` (generated from `src/capabilities.rs`).

## Schemas

| File | URL (Pages) |
| --- | --- |
| Widget IR | https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json |
| `plugins.widgets` | https://s00d.github.io/tauri-plugin-widgets/schemas/plugin-config.v1.json |
| Capabilities | https://s00d.github.io/tauri-plugin-widgets/schemas/capabilities.json |

Add `"$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json"` to widget JSON for editor autocomplete.

Maintainer CLI: [scripts/README.md](https://github.com/s00d/tauri-plugin-widgets/blob/main/scripts/README.md) · [Doctor](/guide/doctor).
