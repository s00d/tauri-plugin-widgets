#!/usr/bin/env bash
# Refresh ALL Windows Adaptive Card JSON snapshots + PNG goldens (Mac host).
# Optional: CASE=weather.small or CASES="a.small b.medium"
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
mkdir -p tests/golden/windows tests/snapshots/adaptive

if [[ -n "${CASE:-}" ]]; then
  CASE="$CASE" cargo run --quiet --bin gen-windows-goldens --features rasterize -- --dump-ac
elif [[ -n "${CASES:-}" ]]; then
  for c in $CASES; do
    CASE="$c" cargo run --quiet --bin gen-windows-goldens --features rasterize -- --dump-ac
  done
else
  # Full corpus — all tests/cases
  cargo run --quiet --bin gen-windows-goldens --features rasterize -- --dump-ac
fi

COUNT=$(find tests/golden/windows -name '*.png' | wc -l | tr -d ' ')
echo "OK → tests/golden/windows ($COUNT png) + tests/snapshots/adaptive"
