#!/usr/bin/env bash
# Real Android Level-2 loop (AppWidgetHost → bitmap), not Robolectric/autogen.
#
#   just android-up                    # once per machine/session
#   scripts/sh/android-shot.sh weather.small
#   GOLDEN_RECORD=1 scripts/sh/android-shot.sh weather.small
#
# Agent workflow:
#   1) write/edit tests/cases/<id>.json + tests/fixtures/…
#   2) run this script
#   3) open out/android/<id>.actual.png (and .diff.png on fail)
#   4) exit 0 = matches golden; non-zero = mismatch / crash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

CASE="${1:?usage: android-shot.sh <case-id>}"
RECORD="${GOLDEN_RECORD:-0}"
PKG_TEST="${TEST_PACKAGE:-git.s00d.widgets.test}"
DEVICE_OUT="/storage/emulated/0/Android/data/${PKG_TEST}/files/widgets-out"
DEVICE_GOLDEN="/storage/emulated/0/Android/data/${PKG_TEST}/files/widgets-golden"
HOST_OUT="${ROOT}/out/android"
HOST_GOLDEN="${ROOT}/tests/golden/android"

if [[ ! -f "tests/cases/${CASE}.json" ]]; then
  echo "android-shot: missing tests/cases/${CASE}.json" >&2
  echo "  create a case (fixture/size/theme/locale) then re-run." >&2
  exit 2
fi

export PATH="${ANDROID_HOME:-$HOME/Library/Android/sdk}/platform-tools:${PATH}"
if ! adb devices 2>/dev/null | grep -qE 'emulator-[0-9]+[[:space:]]+device'; then
  if [[ "${ANDROID_UP:-0}" == "1" || "${ANDROID_UP:-}" == "true" ]]; then
    bash scripts/sh/android-up.sh
  else
    echo "android-shot: no emulator device — run: just android-up" >&2
    echo "  (or ANDROID_UP=1 $0 ${CASE})" >&2
    exit 1
  fi
fi

# Bind before + after install: first connected run installs .test APK.
grantbind() {
  adb shell appwidget grantbind --package git.s00d.widgets >/dev/null 2>&1 || true
  adb shell appwidget grantbind --package "${PKG_TEST}" >/dev/null 2>&1 || true
}
grantbind

mkdir -p "$HOST_OUT" "$HOST_GOLDEN"
# Clear prior device artifacts for this case so pulls are unambiguous.
adb shell "rm -f '${DEVICE_OUT}/${CASE}.actual.png' '${DEVICE_OUT}/${CASE}.diff.png' '${DEVICE_GOLDEN}/${CASE}.png'" \
  >/dev/null 2>&1 || true

# -Pcase / -Pgolden.record are mapped in android/build.gradle.kts → runner args.
GRADLE_ARGS=(
  :connectedDebugAndroidTest
  "-Pandroid.testInstrumentationRunnerArguments.class=git.s00d.widgets.WidgetRenderTest"
  "-Pcase=${CASE}"
)
if [[ "$RECORD" == "1" || "$RECORD" == "true" ]]; then
  GRADLE_ARGS+=(-Pgolden.record=true)
  echo "android-shot: RECORD ${CASE} (AppWidgetHost)"
else
  echo "android-shot: ASSERT ${CASE} (AppWidgetHost vs tests/golden/android/${CASE}.png)"
fi

# Ensure test package exists before bind (installDebugAndroidTest is cheap if up-to-date).
bash scripts/sh/android-gradle.sh :installDebugAndroidTest >/dev/null 2>&1 || true
grantbind

set +e
bash scripts/sh/android-gradle.sh "${GRADLE_ARGS[@]}"
CODE=$?
set -e

# Always try to surface bitmaps for inspection (even on failure).
adb pull "${DEVICE_OUT}/" "${HOST_OUT}/.pull-out/" >/dev/null 2>&1 || true
if [[ -f "${HOST_OUT}/.pull-out/${CASE}.actual.png" ]]; then
  mv -f "${HOST_OUT}/.pull-out/${CASE}.actual.png" "${HOST_OUT}/${CASE}.actual.png"
fi
# Drop stale host diff unless this run produced a new one.
rm -f "${HOST_OUT}/${CASE}.diff.png"
if [[ -f "${HOST_OUT}/.pull-out/${CASE}.diff.png" ]]; then
  mv -f "${HOST_OUT}/.pull-out/${CASE}.diff.png" "${HOST_OUT}/${CASE}.diff.png"
fi
rm -rf "${HOST_OUT}/.pull-out"

if [[ "$RECORD" == "1" || "$RECORD" == "true" ]]; then
  if [[ "$CODE" -ne 0 ]]; then
    echo "android-shot: record aborted (test exit=$CODE) — golden not updated" >&2
  else
    adb pull "${DEVICE_GOLDEN}/${CASE}.png" "${HOST_GOLDEN}/${CASE}.png" >/dev/null 2>&1 || true
    if [[ -f "${HOST_GOLDEN}/${CASE}.png" ]]; then
      cp -f "${HOST_GOLDEN}/${CASE}.png" "${HOST_OUT}/${CASE}.actual.png"
      echo "android-shot: recorded → tests/golden/android/${CASE}.png"
    else
      echo "android-shot: record pull failed (${DEVICE_GOLDEN}/${CASE}.png)" >&2
      exit 1
    fi
  fi
fi

echo "android-shot: actual → ${HOST_OUT}/${CASE}.actual.png"
[[ -f "${HOST_OUT}/${CASE}.diff.png" ]] && echo "android-shot: diff   → ${HOST_OUT}/${CASE}.diff.png"
[[ -f "${HOST_GOLDEN}/${CASE}.png" ]] && echo "android-shot: golden → ${HOST_GOLDEN}/${CASE}.png"

exit "$CODE"
