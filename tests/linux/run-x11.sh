#!/usr/bin/env bash
# X11 batch: xprop asserts (gate) + root screenshots (triage).
set -euo pipefail

cd /work
mkdir -p out/png/linux-x11 out/props

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

PROBE_FEATURES="${PROBE_FEATURES:-}"
FEATURES_ARGS=()
if [[ -n "$PROBE_FEATURES" ]]; then
  FEATURES_ARGS=(--features "$PROBE_FEATURES")
fi

PROBE_BIN="${PROBE_BIN:-examples/widget-probe/target/release/widget-probe}"
if [[ ! -x "$PROBE_BIN" ]]; then
  echo "linux x11: building widget-probe..."
  export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
  export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/widget-probe-target}"
  export RUSTFLAGS="${RUSTFLAGS:--C link-arg=-fuse-ld=lld}"
  cargo build --release --manifest-path examples/widget-probe/Cargo.toml "${FEATURES_ARGS[@]}"
  mkdir -p examples/widget-probe/target/release
  cp -f "$CARGO_TARGET_DIR/release/widget-probe" "$PROBE_BIN"
  strip "$PROBE_BIN" 2>/dev/null || true
fi

wait_wid() {
  local wid=""
  for _ in $(seq 1 30); do
    wid=$(xdotool search --name widget-probe 2>/dev/null | tail -1 || true)
    if [[ -n "$wid" ]]; then
      echo "$wid"
      return 0
    fi
    sleep 0.4
  done
  return 1
}

assert_props() {
  local name="$1" wid="$2"
  xprop -id "$wid" _NET_WM_WINDOW_TYPE _NET_WM_STATE > "out/props/$name.txt" || true
  if ! grep -q "_NET_WM_WINDOW_TYPE_DESKTOP" "out/props/$name.txt"; then
    echo "FAIL type-hint: $name"
    cat "out/props/$name.txt"
    return 1
  fi
  if ! grep -q "_NET_WM_STATE_SKIP_TASKBAR" "out/props/$name.txt"; then
    echo "FAIL taskbar: $name"
    cat "out/props/$name.txt"
    return 1
  fi
  return 0
}

stop_probe() {
  local pid="${1:-}"
  if [[ -n "$pid" ]]; then
    kill "$pid" 2>/dev/null || true
    sleep 0.3
    kill -9 "$pid" 2>/dev/null || true
  fi
  pkill -9 -f '/widget-probe' 2>/dev/null || true
}

# Prefer cases → fixtures; fall back to a small fixture list.
CASES=()
if [[ -d tests/cases ]]; then
  while IFS= read -r -d '' f; do
    CASES+=("$f")
  done < <(find tests/cases -name '*.json' -print0 | sort -z)
fi

# Optional limit for smoke: LINUX_CASES="weather.small upcoming-payments.medium"
if [[ -n "${LINUX_CASES:-}" ]]; then
  FILTERED=()
  for case_path in "${CASES[@]}"; do
    base=$(basename "$case_path" .json)
    for want in $LINUX_CASES; do
      if [[ "$base" == "$want" ]]; then
        FILTERED+=("$case_path")
      fi
    done
  done
  CASES=("${FILTERED[@]}")
fi

if [[ ${#CASES[@]} -eq 0 ]]; then
  echo "No cases — smoke weather + upcoming-payments"
  FAIL=0
  for fx in tests/fixtures/presets/weather.json tests/fixtures/presets/upcoming-payments.json; do
    [[ -f "$fx" ]] || continue
    name=$(basename "$fx" .json)
    echo "== $name (small) =="
    stop_probe
    sleep 0.2
    "$PROBE_BIN" "$fx" small &
    APP=$!
    if ! WID=$(wait_wid); then
      echo "FAIL: no window for $name"
      stop_probe "$APP"
      FAIL=1
      continue
    fi
    assert_props "$name" "$WID" || FAIL=1
    sleep 2.5
    bash tests/linux/grab-window.sh "out/png/linux-x11/$name.png" || FAIL=1
    stop_probe "$APP"
  done
  stop_probe
  [[ "$FAIL" -eq 0 ]] || exit 1
  echo "linux x11: OK (smoke)"
  exit 0
fi

FAIL=0
for case_path in "${CASES[@]}"; do
  name=$(basename "$case_path" .json)
  fixture_rel=$(jq -r '.fixture // empty' "$case_path")
  size=$(jq -r '.size // "small"' "$case_path")
  if [[ -z "$fixture_rel" ]]; then
    echo "SKIP $name (no fixture)"
    continue
  fi
  fx="tests/fixtures/${fixture_rel}.json"
  if [[ ! -f "$fx" ]]; then
    echo "SKIP $name (missing $fx)"
    continue
  fi

  echo "== $name ($size) =="
  stop_probe
  sleep 0.2

  "$PROBE_BIN" "$fx" "$size" &
  APP=$!
  if ! WID=$(wait_wid); then
    echo "FAIL: no window for $name"
    stop_probe "$APP"
    FAIL=1
    continue
  fi

  assert_props "$name" "$WID" || FAIL=1
  sleep 2.5
  bash tests/linux/grab-window.sh "out/png/linux-x11/$name.png" || FAIL=1
  stop_probe "$APP"
done

stop_probe
if [[ "$FAIL" -ne 0 ]]; then
  echo "linux x11: FAILED"
  exit 1
fi
echo "linux x11: OK (${#CASES[@]} cases)"
