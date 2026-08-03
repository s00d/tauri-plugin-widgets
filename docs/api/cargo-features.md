---
title: Cargo features
---

# Cargo features

OS backends follow `target_os` (no empty Cargo feature toggles). Optional features:

- `linux` (**default**) — gtk + x11rb for `_NET_WM_WINDOW_TYPE_DESKTOP` pin
- `image-prefetch` (**default**) — host-side remote `image.url` → `data:` URI with disk/memory TTL cache (ureq). **Not linked on Android** (target-gated; Kotlin `localPath` owns remote images). Disable on desktop/iOS/macOS/Windows to drop TLS from the dependency tree.
- `rasterize` — SVG→PNG data-URI for Adaptive Cards chart/canvas/gauge/shape/zstack, gradient backgrounds, and `image.systemName` glyphs (pulls resvg). **Without it, Windows Widgets Board falls back to emoji `TextBlock` for SF symbols.** Enable for Windows Widgets Board parity.
- `workerw` — Windows wallpaper WorkerW parenting
- `layer-shell` — Wayland gtk-layer-shell Background (requires `linux`)
- `macos-private-api` — transparent macOS desktop widget webviews (also enable Tauri `macosPrivateApi` in conf)
- `schema` / `codegen` — JSON Schema / TS codegen tooling (dev)

```toml
# Default (linux pin + image-prefetch; other OS via target_os)
tauri-plugin-widgets = "0.5"

# Slim: no linux pin / no gtk / no ureq
tauri-plugin-widgets = { version = "0.5", default-features = false }

# Windows Widgets Board with chart rasterization
tauri-plugin-widgets = { version = "0.5", default-features = false, features = ["rasterize"] }
```
