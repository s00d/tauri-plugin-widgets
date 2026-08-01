#!/usr/bin/env bash
# Wayland + sway headless: assert a layer_shell surface exists.
set -euo pipefail

cd /work
mkdir -p out/png/linux-wayland

export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/tmp/xdg-wshot}"
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
export WLR_BACKENDS=headless
export WLR_LIBINPUT_NO_DEVICES=1
export GDK_BACKEND=wayland

pkill -x sway 2>/dev/null || true
pkill -f widget-probe 2>/dev/null || true
sleep 0.5

sway -c tests/linux/sway.conf >/tmp/sway-wshot.log 2>&1 &
SWAY_PID=$!
sleep 2

# Discover WAYLAND_DISPLAY (usually wayland-1 under headless).
export WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-wayland-1}"
if [[ ! -S "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY" ]]; then
  for cand in "$XDG_RUNTIME_DIR"/wayland-*; do
    if [[ -S "$cand" ]]; then
      export WAYLAND_DISPLAY="$(basename "$cand")"
      break
    fi
  done
fi
echo "wayland display: ${WAYLAND_DISPLAY:-none}"

FEATURES_ARGS=(--features layer-shell)

fx="${1:-tests/fixtures/presets/weather.json}"
size="${2:-small}"
name=$(basename "$fx" .json)

cargo run --quiet --release --manifest-path examples/widget-probe/Cargo.toml \
  "${FEATURES_ARGS[@]}" -- "$fx" "$size" &
APP=$!
sleep 4

if ! swaymsg -t get_tree | jq -e '
  .. | objects | select(.type? == "layer_shell" or .shell? == "layer") | .
' >/dev/null 2>&1; then
  # Broader: any node mentioning layer
  if ! swaymsg -t get_tree | jq -e 'tostring | test("layer")' >/dev/null; then
    echo "FAIL: layer-shell surface not created"
    swaymsg -t get_tree | head -c 4000 || true
    kill "$APP" 2>/dev/null || true
    kill "$SWAY_PID" 2>/dev/null || true
    exit 1
  fi
fi

grim "out/png/linux-wayland/$name.png" || true
kill "$APP" 2>/dev/null || true
wait "$APP" 2>/dev/null || true
kill "$SWAY_PID" 2>/dev/null || true
echo "linux wayland: OK ($name)"
