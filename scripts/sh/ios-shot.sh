#!/usr/bin/env bash
# Real iOS Level-2 loop (SwiftUI ImageRenderer + WidgetChrome), not SVG/autogen.
# Same cases as Android: tests/cases/<id>.json + tests/fixtures/…
#
#   scripts/sh/ios-shot.sh weather.small
#   GOLDEN_RECORD=1 scripts/sh/ios-shot.sh weather.small
#
# Agent workflow:
#   1) write/edit tests/cases/<id>.json + fixtures
#   2) run this script (host macOS — no Simulator required)
#   3) open out/ios/<id>.actual.png
#   4) exit 0 = matches golden; non-zero = mismatch / crash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

CASE="${1:?usage: ios-shot.sh <case-id>}"
RECORD="${GOLDEN_RECORD:-0}"
HOST_OUT="${ROOT}/out/ios"
HOST_GOLDEN="${ROOT}/tests/golden/ios"

if [[ ! -f "tests/cases/${CASE}.json" ]]; then
  echo "ios-shot: missing tests/cases/${CASE}.json" >&2
  echo "  create a case (fixture/size/theme/locale) then re-run." >&2
  exit 2
fi

if ! command -v swift >/dev/null 2>&1; then
  echo "ios-shot: swift not on PATH (need Xcode / Swift toolchain)" >&2
  exit 1
fi

mkdir -p "$HOST_OUT" "$HOST_GOLDEN"
rm -f "${HOST_OUT}/${CASE}.actual.png"

export CASE
if [[ "$RECORD" == "1" || "$RECORD" == "true" ]]; then
  export GOLDEN_RECORD=1
  echo "ios-shot: RECORD ${CASE} (SwiftUI ImageRenderer → tests/golden/ios/)"
else
  unset GOLDEN_RECORD || true
  echo "ios-shot: ASSERT ${CASE} (SwiftUI vs tests/golden/ios/${CASE}.png)"
fi

set +e
(
  cd swift
  # XCTest name is like "-[TauriWidgetsTests.RenderTests testCases]".
  # Bare "RenderTests" also matches MacRenderTests — pin the module.class.
  swift test --filter 'TauriWidgetsTests\.RenderTests/testCases'
)
CODE=$?
set -e

if [[ ! -f "${HOST_OUT}/${CASE}.actual.png" ]]; then
  # Fallback: copy newly recorded golden so the agent always has a bitmap to Read.
  if [[ -f "${HOST_GOLDEN}/${CASE}.png" && ( "$RECORD" == "1" || "$RECORD" == "true" ) ]]; then
    cp -f "${HOST_GOLDEN}/${CASE}.png" "${HOST_OUT}/${CASE}.actual.png"
  fi
fi

if [[ "$RECORD" == "1" || "$RECORD" == "true" ]]; then
  if [[ "$CODE" -ne 0 ]]; then
    echo "ios-shot: record aborted (test exit=$CODE) — check swift output" >&2
  elif [[ -f "${HOST_GOLDEN}/${CASE}.png" ]]; then
    cp -f "${HOST_GOLDEN}/${CASE}.png" "${HOST_OUT}/${CASE}.actual.png"
    echo "ios-shot: recorded → tests/golden/ios/${CASE}.png"
  else
    echo "ios-shot: record missing ${HOST_GOLDEN}/${CASE}.png" >&2
    exit 1
  fi
fi

echo "ios-shot: actual → ${HOST_OUT}/${CASE}.actual.png"
[[ -f "${HOST_GOLDEN}/${CASE}.png" ]] && echo "ios-shot: golden → ${HOST_GOLDEN}/${CASE}.png"

exit "$CODE"
