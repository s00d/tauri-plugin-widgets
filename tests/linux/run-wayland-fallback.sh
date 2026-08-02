#!/usr/bin/env bash
# Wayland without layer-shell feature: probe must stay up (ordinary window fallback).
set -euo pipefail

cd /work
mkdir -p out/png/linux-wayland-fallback

export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp/xdg-wshot-fb}"
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
export WLR_BACKENDS=headless
export WLR_LIBINPUT_NO_DEVICES=1
export GDK_BACKEND=wayland

pkill -x sway 2>/dev/null || true
pkill -f widget-probe 2>/dev/null || true
sleep 0.5

sway -c tests/linux/sway.conf >/tmp/sway-wshot-fb.log 2>&1 &
SWAY_PID=$!
sleep 2

export WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-wayland-1}"
if [[ ! -S "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY" ]]; then
  for cand in "$XDG_RUNTIME_DIR"/wayland-*; do
    if [[ -S "$cand" ]]; then
      export WAYLAND_DISPLAY="$(basename "$cand")"
      break
    fi
  done
fi

fx="${1:-tests/fixtures/presets/weather.json}"
size="${2:-small}"
name=$(basename "$fx" .json)

# Explicitly NO layer-shell feature — should not crash.
cargo run --quiet --release --manifest-path examples/widget-probe/Cargo.toml -- "$fx" "$size" &
APP=$!
sleep 4

if ! kill -0 "$APP" 2>/dev/null; then
  echo "FAIL: probe exited (expected fallback window, not crash)"
  kill "$SWAY_PID" 2>/dev/null || true
  exit 1
fi

grim "out/png/linux-wayland-fallback/$name.png" || true
kill "$APP" 2>/dev/null || true
wait "$APP" 2>/dev/null || true
kill "$SWAY_PID" 2>/dev/null || true
echo "linux wayland-fallback: OK ($name)"
