---
title: Linux setup
---

# Linux setup

Linux desktop widgets use the same **embedded** webview path as [Desktop webview](/guide/setup/desktop): `createWidgetWindow` without `url` (no `widget.html` copy). Extra Linux behavior is window pinning.

## How it works

1. `setWidgetConfig` writes IR into the desktop file store for `(group, widgetId)`.
2. `createWidgetWindow` opens a frameless Tauri webview on the `widgetview` protocol.
3. With Cargo feature `linux` (**default**), the plugin sets X11 `_NET_WM_WINDOW_TYPE_DESKTOP` so the window behaves like a desktop pin.
4. Optional feature `layer-shell` remaps the window onto gtk-layer-shell **Background** on Wayland.

## Setup

1. Keep default features, or enable explicitly:

```toml
tauri-plugin-widgets = { version = "0.5", features = ["linux"] }
# Wayland:
# tauri-plugin-widgets = { version = "0.5", features = ["linux", "layer-shell"] }
```

2. Allow the window label in capabilities (`windows` + `widgets:default`) — [Install](/guide/install#permissions).
3. Call `createWidgetWindow` as in [First widget](/guide/first-widget).

Contributor harness / debugging: [Linux harness](/contributing/linux-harness).

## When it fails

- Enable the `linux` feature for X11 DESKTOP pinning (`default-features = false` drops it).
- Wayland: try `layer-shell`; plain X11 hints will not apply.
- See [Linux harness](/contributing/linux-harness) for debugging pinned windows.

More symptoms: [Troubleshooting index](/guide/troubleshooting).
