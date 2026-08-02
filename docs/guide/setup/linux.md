---
title: Linux setup
---

# Linux setup

Linux desktop widgets use the same webview path as [Desktop webview](/guide/setup/desktop): `createWidgetWindow` without `url` loads the **embedded** `widget.html` (nothing to copy into the app).

- Enable the default `linux` Cargo feature for X11 `_NET_WM_WINDOW_TYPE_DESKTOP` pinning.
- Optional `layer-shell` for Wayland gtk-layer-shell Background layer.
- Harness and debugging notes: [Linux harness](/contributing/linux-harness).

## When it fails

- Enable the `linux` feature for X11 DESKTOP pinning.
- See [Linux harness](/contributing/linux-harness) for debugging pinned windows.

More symptoms: [Troubleshooting index](/guide/troubleshooting).
