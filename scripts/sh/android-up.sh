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
PACKAGE="${TEST_PACKAGE:-git.s00d.widgets}"
# Instrumentation host may be either the app package or the .test package.
PACKAGES_TO_BIND=("$PACKAGE" "git.s00d.widgets" "git.s00d.widgets.test")
# Frozen wall clock (UTC): 2026-08-01 12:00:00
FIXED_DATE="${FIXED_DATE:-080112002026.00}"

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

if ! command -v sdkmanager >/dev/null 2>&1; then
  echo "sdkmanager not on PATH — source Android SDK env first" >&2
  exit 1
fi

yes | sdkmanager --licenses >/dev/null 2>&1 || true
sdkmanager "$SYS_IMAGE" "platform-tools" "emulator" >/dev/null

echo "no" | avdmanager create avd -n "$AVD_NAME" -k "$SYS_IMAGE" -f >/dev/null 2>&1 || true

# Default avdmanager skins are ~320×640. RemoteViews bitmap budget ≈ 6×W×H of the
# physical display — too small for xxhdpi (480) widget bitmaps (charts/gauges →
# "Can't show content"). Pin a phone-sized panel + density before boot.
AVD_DIR="${ANDROID_AVD_HOME:-$HOME/.android/avd}/${AVD_NAME}.avd"
if [[ -f "$AVD_DIR/config.ini" ]]; then
  python3 - <<'PY' "$AVD_DIR/config.ini"
import re, sys
path = sys.argv[1]
text = open(path).read()
def set_key(text, key, value):
    pat = re.compile(rf"(?m)^{re.escape(key)}=.*$")
    line = f"{key}={value}"
    if pat.search(text):
        return pat.sub(line, text)
    return text.rstrip() + "\n" + line + "\n"
for k, v in {
    "hw.lcd.width": "1080",
    "hw.lcd.height": "2400",
    "hw.lcd.depth": "16",
    "hw.lcd.density": "480",
    "skin.name": "1080x2400",
    "skin.path": "1080x2400",
}.items():
    text = set_key(text, k, v)
open(path, "w").write(text)
PY
fi

# Boot the named AVD if not already running; pin all adb commands to its serial.
resolve_serial() {
  adb devices | awk -v avd="$AVD_NAME" '
    /emulator-[0-9]+[[:space:]]+device/ { print $1; exit }
  '
}
SERIAL="$(resolve_serial || true)"
if [[ -z "${SERIAL:-}" ]]; then
  # nohup/disown: agent shells otherwise SIGHUP the qemu child on exit.
  nohup emulator -avd "$AVD_NAME" -no-window -no-audio -no-snapshot \
    >/tmp/widgets-emulator.log 2>&1 &
  disown || true
  echo "emulator starting (log: /tmp/widgets-emulator.log)"
  deadline=$((SECONDS + 120))
  while [[ -z "${SERIAL:-}" ]]; do
    if (( SECONDS >= deadline )); then
      echo "emulator serial timeout for AVD=$AVD_NAME" >&2
      exit 1
    fi
    sleep 1
    SERIAL="$(resolve_serial || true)"
  done
fi
export ANDROID_SERIAL="$SERIAL"
echo "using ANDROID_SERIAL=$ANDROID_SERIAL (AVD=$AVD_NAME)"

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
# Goldens are xxhdpi (170dp → 510px). Fresh avdmanager AVDs default to mdpi 160.
adb shell wm density 480 >/dev/null 2>&1 || true
# Case default theme is dark; light cases flip this from the test harness.
adb shell cmd uimode night yes >/dev/null 2>&1 || true
adb shell service call alarm 3 s16 UTC >/dev/null 2>&1 || true
# Best-effort freeze wall clock (may require root on some images)
adb shell "su 0 date ${FIXED_DATE}" >/dev/null 2>&1 || \
  adb shell "date ${FIXED_DATE}" >/dev/null 2>&1 || \
  echo "warn: could not set date to ${FIXED_DATE} (relative date cases may drift)" >&2

for pkg in "${PACKAGES_TO_BIND[@]}"; do
  adb shell appwidget grantbind --package "$pkg" >/dev/null 2>&1 || \
    echo "warn: grantbind failed for $pkg (bind may still work after install)" >&2
done

echo "android-up ok: avd=$AVD_NAME arch=$ARCH package=$PACKAGE"
