---
title: Cargo features
---

# Cargo features

OS backends follow `target_os` (no empty Cargo feature toggles). Optional features:

- `linux` (**default**) — gtk + x11rb for `_NET_WM_WINDOW_TYPE_DESKTOP` pin
- `rasterize` — SVG→PNG data-URI for Adaptive Cards chart/canvas/gauge (pulls resvg)
- `workerw` — Windows wallpaper WorkerW parenting
- `layer-shell` — Wayland gtk-layer-shell Background (requires `linux`)
- `macos-private-api` — transparent macOS desktop widget webviews (also enable Tauri `macosPrivateApi` in conf)
- `schema` / `codegen` — JSON Schema / TS codegen tooling (dev)

```toml
# Default (linux pin enabled; other OS via target_os)
tauri-plugin-widgets = "0.4"

# Slim: no linux pin / no gtk
tauri-plugin-widgets = { version = "0.4", default-features = false }

# Windows Widgets Board with chart rasterization
tauri-plugin-widgets = { version = "0.4", default-features = false, features = ["rasterize"] }
```
