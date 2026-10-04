#!/usr/bin/env bash
# Real macOS Level-A loop (AppKit NSHostingView + WidgetChrome), not SVG/autogen.
# Same cases as Android/iOS: tests/cases/<id>.json + tests/fixtures/…
# Uses macos canvas sizes from case.macos (defaults: 158² / 338×158 / 338×354).
#
#   scripts/sh/macos-shot.sh weather.small
#   GOLDEN_RECORD=1 scripts/sh/macos-shot.sh weather.small
#
# Agent workflow:
#   1) write/edit tests/cases/<id>.json + fixtures
#   2) run this script on a Mac host
#   3) open out/macos/<id>.actual.png
#   4) exit 0 = matches golden; non-zero = mismatch / crash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

CASE="${1:?usage: macos-shot.sh <case-id>}"
RECORD="${GOLDEN_RECORD:-0}"
HOST_OUT="${ROOT}/out/macos"
HOST_GOLDEN="${ROOT}/tests/golden/macos"

if [[ ! -f "tests/cases/${CASE}.json" ]]; then
  echo "macos-shot: missing tests/cases/${CASE}.json" >&2
  echo "  create a case (fixture/size/theme/locale[/macos]) then re-run." >&2
  exit 2
fi

if ! command -v swift >/dev/null 2>&1; then
  echo "macos-shot: swift not on PATH (need Xcode / Swift toolchain)" >&2
  exit 1
fi

mkdir -p "$HOST_OUT" "$HOST_GOLDEN"
rm -f "${HOST_OUT}/${CASE}.actual.png"

export CASE
if [[ "$RECORD" == "1" || "$RECORD" == "true" ]]; then
  export GOLDEN_RECORD=1
  echo "macos-shot: RECORD ${CASE} (AppKit NSHostingView → tests/golden/macos/)"
else
  unset GOLDEN_RECORD || true
  echo "macos-shot: ASSERT ${CASE} (AppKit vs tests/golden/macos/${CASE}.png)"
fi

set +e
(
  cd swift
  # Pin module.class — do not match RenderTests alone.
  swift test --filter 'TauriWidgetsTests\.MacRenderTests/testCases'
)
CODE=$?
set -e

if [[ ! -f "${HOST_OUT}/${CASE}.actual.png" ]]; then
  if [[ -f "${HOST_GOLDEN}/${CASE}.png" && ( "$RECORD" == "1" || "$RECORD" == "true" ) ]]; then
    cp -f "${HOST_GOLDEN}/${CASE}.png" "${HOST_OUT}/${CASE}.actual.png"
  fi
fi

if [[ "$RECORD" == "1" || "$RECORD" == "true" ]]; then
  if [[ "$CODE" -ne 0 ]]; then
    echo "macos-shot: record aborted (test exit=$CODE) — check swift output" >&2
  elif [[ -f "${HOST_GOLDEN}/${CASE}.png" ]]; then
    cp -f "${HOST_GOLDEN}/${CASE}.png" "${HOST_OUT}/${CASE}.actual.png"
    echo "macos-shot: recorded → tests/golden/macos/${CASE}.png"
  else
    echo "macos-shot: record missing ${HOST_GOLDEN}/${CASE}.png" >&2
    exit 1
  fi
fi

echo "macos-shot: actual → ${HOST_OUT}/${CASE}.actual.png"
[[ -f "${HOST_GOLDEN}/${CASE}.png" ]] && echo "macos-shot: golden → ${HOST_GOLDEN}/${CASE}.png"

exit "$CODE"
