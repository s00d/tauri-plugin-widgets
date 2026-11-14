# Platform support

The authoritative table is generated from `src/capabilities.rs` and published as
`schemas/capabilities.json` (in the npm package) and
https://s00d.github.io/tauri-plugin-widgets/guide/tiers. Read it for the exact cell.

Rules of thumb:

- Core elements are `full` everywhere except Windows Widgets Board `zstack` / `shape` (degraded).
- `chart`, `canvas`, `gauge` on the Widgets Board are rasterized (feature `rasterize`).
- `config.source` is not fetched by the Widgets Board — push `setWidgetData` from the app.
- `style.privacySensitive` only has an effect on iOS / macOS.
- `activity` is native on iOS and Android; desktop shows an always-on-top window.
- `npx tauri-widgets validate` reports degraded / unsupported uses for chosen platforms.
