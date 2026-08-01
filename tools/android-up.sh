#!/usr/bin/env bash
# Pin Android emulator for visual stand (AppWidgetHost tests).
set -euo pipefail

AVD_NAME="${AVD_NAME:-widgets-test}"
# Prefer arm64 on Apple Silicon; x86_64 on Intel.
ARCH="${ANDROID_ARCH:-}"
if [[ -z "$ARCH" ]]; then
  case "$(uname -m)" in
    arm64|aarch64) ARCH="arm64-v8a" ;;
    *) ARCH="x86_64" ;;
  esac
fi
SYS_IMAGE="system-images;android-34;google_apis;${ARCH}"
PACKAGE="${TEST_PACKAGE:-git.s00d.widgets.test}"
# Frozen wall clock (UTC): 2026-08-01 12:00:00
FIXED_DATE="${FIXED_DATE:-080112002026.00}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if ! command -v sdkmanager >/dev/null 2>&1; then
  echo "sdkmanager not on PATH — source Android SDK env first" >&2
  exit 1
fi

yes | sdkmanager --licenses >/dev/null 2>&1 || true
sdkmanager "$SYS_IMAGE" "platform-tools" "emulator" >/dev/null

echo "no" | avdmanager create avd -n "$AVD_NAME" -k "$SYS_IMAGE" -f >/dev/null 2>&1 || true

# Boot if not already running
if ! adb devices | grep -qE 'emulator-[0-9]+\s+device'; then
  emulator -avd "$AVD_NAME" -no-window -no-audio -no-snapshot -wipe-data >/tmp/widgets-emulator.log 2>&1 &
  echo "emulator starting (log: /tmp/widgets-emulator.log)"
fi

adb wait-for-device
# Boot completed (poll, not fixed sleep as readiness signal — bounded wait)
deadline=$((SECONDS + 180))
while [[ -z "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" ]]; do
  if (( SECONDS >= deadline )); then
    echo "emulator boot timeout" >&2
    exit 1
  fi
  sleep 1
done

adb shell settings put global window_animation_scale 0
adb shell settings put global transition_animation_scale 0
adb shell settings put global animator_duration_scale 0
adb shell settings put system font_scale 1.0
adb shell service call alarm 3 s16 UTC >/dev/null 2>&1 || true
# Best-effort freeze wall clock (may require root on some images)
adb shell "su 0 date ${FIXED_DATE}" >/dev/null 2>&1 || \
  adb shell "date ${FIXED_DATE}" >/dev/null 2>&1 || \
  echo "warn: could not set date to ${FIXED_DATE} (relative date cases may drift)" >&2

adb shell appwidget grantbind --package "$PACKAGE" >/dev/null 2>&1 || \
  echo "warn: grantbind failed for $PACKAGE (bind may still work after install)" >&2

echo "android-up ok: avd=$AVD_NAME arch=$ARCH package=$PACKAGE"
