#!/usr/bin/env bash
# Start Xvfb + openbox + dbus + widget-probe --watch. Blocks forever.
set -euo pipefail

cd /work
mkdir -p out/inbox out/png/linux-x11 out/props out/linux

export DISPLAY="${DISPLAY:-:99}"
export LIBGL_ALWAYS_SOFTWARE="${LIBGL_ALWAYS_SOFTWARE:-1}"
export WEBKIT_DISABLE_COMPOSITING_MODE="${WEBKIT_DISABLE_COMPOSITING_MODE:-1}"
export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"
export GSK_RENDERER="${GSK_RENDERER:-cairo}"

# Session bus for GTK/AT-SPI (WebKit is happier with it under Xvfb).
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

PROBE_FEATURES="${PROBE_FEATURES:-}"
FEATURES_ARGS=()
if [[ -n "$PROBE_FEATURES" ]]; then
  FEATURES_ARGS=(--features "$PROBE_FEATURES")
fi

PROBE_BIN=examples/widget-probe/target/release/widget-probe
if [[ "${FORCE_REBUILD:-0}" == "1" || ! -x "$PROBE_BIN" ]]; then
  echo "linux boot: building widget-probe (release)..."
  # Release+Tauri on arm64 Docker is memory-hungry; cap jobs to avoid SIGKILL/OOM.
  # lld avoids aarch64 R_AARCH64_PREL32 truncation on huge binaries.
  # Build on container-local disk — virtiofs target dirs are slow and fill the VM.
  export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
  export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/widget-probe-target}"
  export RUSTFLAGS="${RUSTFLAGS:--C link-arg=-fuse-ld=lld}"
  cargo build --release --manifest-path examples/widget-probe/Cargo.toml "${FEATURES_ARGS[@]}"
  mkdir -p examples/widget-probe/target/release
  cp -f "$CARGO_TARGET_DIR/release/widget-probe" "$PROBE_BIN"
  strip "$PROBE_BIN" 2>/dev/null || true
else
  echo "linux boot: using existing $PROBE_BIN"
fi

SIZE="${PROBE_SIZE:-small}"
echo "linux boot: starting widget-probe --watch (size=${SIZE})"
exec "$PROBE_BIN" --watch /work/out/inbox/linux.json "$SIZE"
