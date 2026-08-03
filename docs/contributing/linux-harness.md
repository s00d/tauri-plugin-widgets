# Linux harness (Docker)

Native **arm64** Ubuntu 24.04 container — do **not** use `--platform linux/amd64` on Apple Silicon (qemu is 5–10× slower).

## Platform features

OS backends follow `target_os`. The only OS-related Cargo feature is `linux` (gtk + x11rb for DESKTOP pin). `rasterize` is opt-in for Adaptive Cards chart/canvas/gauge → PNG.

| Feature | Default | What it gates |
|---------|---------|----------------|
| `linux` | yes | X11 desktop pin (`_NET_WM_*`) + gtk; prerequisite for `layer-shell` |
| `rasterize` | no | SVG→PNG for Adaptive Cards |
| `layer-shell` | no | Wayland gtk-layer-shell Background |
| `workerw` | no | Windows wallpaper WorkerW parenting |

```toml
# Default (linux pin)
tauri-plugin-widgets = "0.5"

# No pin / no gtk
tauri-plugin-widgets = { version = "0.5", default-features = false }

# Widgets Board charts
tauri-plugin-widgets = { version = "0.5", features = ["rasterize"] }
```

## What is tested

| Gate | How | Flaky? |
|------|-----|--------|
| X11 desktop hints | `xprop` `_NET_WM_WINDOW_TYPE_DESKTOP` + `_NET_WM_STATE_SKIP_TASKBAR` | No |
| Wayland layer-shell | `swaymsg get_tree` finds a layer surface | Low |
| Wayland fallback | probe stays alive **without** `layer-shell` feature | No |
| Screenshots | `xwd`/`import` of **widget window id** (not root) via one-shot Docker | Live triage |

Pixels are **not** the CI gate — they help when xprop fails.

> Live PNGs: `just shot-linux weather small` runs `tests/linux/shot-once.sh` (Xvfb + probe + `grab-window.sh`). Captures the largest `widget-probe` window — root grabs stay blank for `_NET_WM_WINDOW_TYPE_DESKTOP`. WebKit uses software GL + `WEBKIT_DISABLE_DMABUF_RENDERER=1` + dbus. Probe builds to `/tmp/widget-probe-target` with `lld` (avoids aarch64 link truncation / virtiofs OOM).

## Quick start

```bash
just linux-up          # optional persistent wshot (boot.sh)
just test-linux-x11    # xprop over tests/cases + out/png/linux-x11/*.png
just shot-linux weather small   # live webview PNG → out/linux/weather-small.png
just linux-up-wl
just test-linux-wayland
just test-linux-fallback
just linux-down
```

`scripts/sh/shot-linux.sh` is **one-shot** (no inbox watch): spins a container, opens the fixture, grabs the live window, exits.  
Persistent `wshot` is optional for interactive iteration.

## Files

- [`tests/linux/Dockerfile`](../tests/linux/Dockerfile)
- [`tests/linux/boot.sh`](../tests/linux/boot.sh) — long-running host
- [`tests/linux/shot-once.sh`](../tests/linux/shot-once.sh) — one-shot live capture
- [`tests/linux/grab-window.sh`](../tests/linux/grab-window.sh) — live WID capture
- [`tests/linux/run-x11.sh`](../tests/linux/run-x11.sh) — property gate + PNGs
- [`tests/linux/run-wayland.sh`](../tests/linux/run-wayland.sh) / [`run-wayland-fallback.sh`](../tests/linux/run-wayland-fallback.sh)
- [`examples/widget-probe/`](../examples/widget-probe/) — minimal Tauri app

## Plugin behaviour

On Linux, `create_widget_window` calls [`src/linux/mod.rs`](../src/linux/mod.rs):

1. If `WAYLAND_DISPLAY` is set and the host was built with `--features layer-shell`, remap onto gtk-layer-shell `Background`.
2. Else on X11, set `_NET_WM_WINDOW_TYPE_DESKTOP` (and skip-taskbar state).
3. Failures log and keep the ordinary frameless window (fallback).

## Plasma / KDE

QML plasmoid stubs (`org.kde.plasma.*`) are **backlog** — cost grows with every Plasma API used.
