#!/usr/bin/env bash
# Pin iOS Simulator for visual stand.
set -euo pipefail

DEVICE_NAME="${DEVICE_NAME:-iPhone 16}"
RUNTIME_HINT="${IOS_RUNTIME:-iOS18.2}"
SIM_NAME="${SIM_NAME:-widgets-test}"

have_runtime() {
  xcrun simctl list runtimes 2>/dev/null | grep -q "$1"
}

RUNTIME=""
for candidate in "$RUNTIME_HINT" "iOS18.2" "iOS18.1" "iOS18.0" "iOS17.5" "iOS17.4"; do
  if have_runtime "$candidate"; then
    RUNTIME="$candidate"
    break
  fi
done

if [[ -z "$RUNTIME" ]]; then
  echo "no suitable iOS runtime found. Install via Xcode → Platforms." >&2
  xcrun simctl list runtimes >&2 || true
  exit 1
fi

UDID="$(xcrun simctl list devices available | grep "$SIM_NAME (" | sed -n 's/.*(\([A-F0-9-]*\)).*/\1/p' | head -1)"

if [[ -z "$UDID" ]]; then
  UDID="$(xcrun simctl create "$SIM_NAME" "$DEVICE_NAME" "$RUNTIME")"
  echo "created simulator $SIM_NAME ($UDID) runtime=$RUNTIME"
else
  echo "reusing simulator $SIM_NAME ($UDID)"
fi

if ! xcrun simctl list devices | grep "$UDID" | grep -q Booted; then
  xcrun simctl boot "$UDID" || true
fi

deadline=$((SECONDS + 120))
while ! xcrun simctl list devices | grep "$UDID" | grep -q Booted; do
  if (( SECONDS >= deadline )); then
    echo "simulator boot timeout" >&2
    exit 1
  fi
  sleep 1
done

xcrun simctl spawn "$UDID" defaults write -g AppleLocale -string en_US
xcrun simctl spawn "$UDID" defaults write -g AppleLanguages -array en
xcrun simctl ui "$UDID" appearance light >/dev/null 2>&1 || true

echo "ios-up ok: udid=$UDID device=$DEVICE_NAME runtime=$RUNTIME"
