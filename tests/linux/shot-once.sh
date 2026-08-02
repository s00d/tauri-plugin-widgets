#!/usr/bin/env bash
# One-shot live capture inside the Linux image (no persistent --watch).
# Usage: shot-once.sh <fixture.json> <size> <out.png>
set -euo pipefail

cd /work
FX="${1:?fixture}"
SIZE="${2:?size}"
OUT="${3:?out.png}"

mkdir -p "$(dirname "$OUT")" out/png/linux-x11

export DISPLAY="${DISPLAY:-:99}"
export LIBGL_ALWAYS_SOFTWARE="${LIBGL_ALWAYS_SOFTWARE:-1}"
export WEBKIT_DISABLE_COMPOSITING_MODE="${WEBKIT_DISABLE_COMPOSITING_MODE:-1}"
export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"
export GSK_RENDERER="${GSK_RENDERER:-cairo}"

if [[ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]]; then
  eval "$(dbus-launch --sh-syntax)"
fi

if ! pgrep -x Xvfb >/dev/null 2>&1; then
  Xvfb "$DISPLAY" -screen 0 1280x800x24 -ac +extension GLX +render -noreset &
  sleep 1
fi
if ! pgrep -x openbox >/dev/null 2>&1; then
  openbox &
  sleep 1
fi

PROBE_BIN="${PROBE_BIN:-examples/widget-probe/target/release/widget-probe}"
if [[ ! -x "$PROBE_BIN" ]]; then
  export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
  export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/widget-probe-target}"
  export RUSTFLAGS="${RUSTFLAGS:--C link-arg=-fuse-ld=lld}"
  echo "shot-once: building widget-probe..."
  cargo build --release --manifest-path examples/widget-probe/Cargo.toml
  mkdir -p examples/widget-probe/target/release
  cp -f "$CARGO_TARGET_DIR/release/widget-probe" "$PROBE_BIN"
  strip "$PROBE_BIN" 2>/dev/null || true
fi

pkill -9 -f '/widget-probe' 2>/dev/null || true
sleep 0.2

"$PROBE_BIN" "$FX" "$SIZE" &
APP=$!
sleep 2.5

bash tests/linux/grab-window.sh "$OUT"
STATUS=$?

kill "$APP" 2>/dev/null || true
sleep 0.2
pkill -9 -f '/widget-probe' 2>/dev/null || true
exit "$STATUS"
