#!/usr/bin/env bash
# From Mac host: live widget PNG via Docker (one-shot webview, not SVG goldens).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

FX="${1:?usage: shot-linux.sh <fixture-name|path> [size]}"
SIZE="${2:-small}"
IMAGE="${WSHOT_IMAGE:-widgets-linux}"

# Resolve fixture path (presets/core before loose fixtures/).
if [[ -f "$FX" ]]; then
  SRC="$FX"
  OUT_STEM="$(basename "$FX" .json)"
elif [[ -f "tests/fixtures/presets/$FX.json" ]]; then
  SRC="tests/fixtures/presets/$FX.json"
  OUT_STEM="$FX"
elif [[ -f "tests/fixtures/core/$FX.json" ]]; then
  SRC="tests/fixtures/core/$FX.json"
  OUT_STEM="$FX"
elif [[ -f "tests/fixtures/$FX.json" ]]; then
  SRC="tests/fixtures/$FX.json"
  OUT_STEM="$FX"
elif [[ -f "tests/cases/$FX.json" ]]; then
  fixture_rel=$(jq -r '.fixture' "tests/cases/$FX.json")
  SIZE=$(jq -r --arg s "$SIZE" '.size // $s' "tests/cases/$FX.json")
  SRC="tests/fixtures/${fixture_rel}.json"
  OUT_STEM="$FX"
else
  echo "shot-linux: fixture not found: $FX" >&2
  exit 1
fi

mkdir -p out/linux
OUT="out/linux/${OUT_STEM}-${SIZE}.png"

# Ensure image exists (cheap if cached).
if ! docker image inspect "${IMAGE}" >/dev/null 2>&1; then
  docker build -t "${IMAGE}" tests/linux
fi

docker volume create widgets-cargo >/dev/null 2>&1 || true
docker volume create widgets-cargo-git >/dev/null 2>&1 || true

echo "shot-linux: one-shot capture ${SRC} (${SIZE}) → ${OUT}"
docker run --rm \
  -v "$ROOT:/work" \
  -v widgets-cargo:/root/.cargo/registry \
  -v widgets-cargo-git:/root/.cargo/git \
  -e DISPLAY=:99 \
  -e LIBGL_ALWAYS_SOFTWARE=1 \
  -e WEBKIT_DISABLE_COMPOSITING_MODE=1 \
  -e WEBKIT_DISABLE_DMABUF_RENDERER=1 \
  -e GSK_RENDERER=cairo \
  -e CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}" \
  -e CARGO_INCREMENTAL=0 \
  -e CARGO_TARGET_DIR=/tmp/widget-probe-target \
  -e RUSTFLAGS="${RUSTFLAGS:--C link-arg=-fuse-ld=lld}" \
  "${IMAGE}" bash tests/linux/shot-once.sh "/work/${SRC}" "$SIZE" "/work/${OUT}"

echo "shot-linux: wrote $OUT ($(wc -c < "$OUT" | tr -d ' ') bytes)"
