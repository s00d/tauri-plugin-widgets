#!/usr/bin/env bash
# Refresh Adaptive Card JSON snapshots (transpile only — not Windows PNG goldens).
# Optional: CASE=weather.small
# For chart/canvas/gauge data-URIs: FEATURES=rasterize
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
mkdir -p tests/snapshots/adaptive
FEATURES="${FEATURES:-}"
feat_args=()
if [[ -n "$FEATURES" ]]; then
  feat_args=(--features "$FEATURES")
fi

if [[ -n "${CASE:-}" ]]; then
  CASE="$CASE" cargo run --quiet --bin gen-adaptive-snapshots "${feat_args[@]}"
elif [[ -n "${CASES:-}" ]]; then
  for c in $CASES; do
    CASE="$c" cargo run --quiet --bin gen-adaptive-snapshots "${feat_args[@]}"
  done
else
  cargo run --quiet --bin gen-adaptive-snapshots "${feat_args[@]}"
fi

COUNT=$(find tests/snapshots/adaptive -name '*.json' | wc -l | tr -d ' ')
echo "OK → tests/snapshots/adaptive ($COUNT json)"
echo "Windows PNG goldens: just record-windows <case> (PreviewHost on VM)"
