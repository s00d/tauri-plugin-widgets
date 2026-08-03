#!/bin/bash
set -euo pipefail

# ─── macOS Widget Extension Builder ──────────────────────────────────────────
# Called automatically via tauri.conf.json → build.beforeBundleCommand.
#
# Builds and signs the .appex so `bundle.macOS.files` can copy it into
# Contents/PlugIns/ during the normal `tauri build` (Tauri nested-codesigns PlugIns).

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CONFIGURATION="${1:-Release}"
DERIVED_DATA="$SCRIPT_DIR/build"
HOST_ENTS="$SCRIPT_DIR/App.entitlements"
EXT_ENTS="$SCRIPT_DIR/TauriWidgetExtension.entitlements"
IDENTITY="${WIDGET_SIGN_IDENTITY:-${APPLE_SIGNING_IDENTITY:--}}"

# ─── Regenerate entitlements from plugins.widgets.appGroup ───────────────────
# Single source of truth in tauri.conf.json — avoids host/extension drift.

CONF="$SCRIPT_DIR/../tauri.conf.json"
if [[ ! -f "$CONF" ]]; then
  CONF="$(cd "$SCRIPT_DIR/../.." && pwd)/src-tauri/tauri.conf.json"
fi

gen_entitlements() {
  local dest="$1"
  local group="$2"
  local sandbox="$3"
  {
    echo '<?xml version="1.0" encoding="UTF-8"?>'
    echo '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">'
    echo '<plist version="1.0">'
    echo '<dict>'
    if [[ "$sandbox" == "1" ]]; then
      echo '    <key>com.apple.security.app-sandbox</key>'
      echo '    <true/>'
    fi
    echo '    <key>com.apple.security.application-groups</key>'
    echo '    <array>'
    echo "        <string>${group}</string>"
    echo '    </array>'
    echo '</dict>'
    echo '</plist>'
  } >"$dest"
}

if [[ -f "$CONF" ]] && command -v node >/dev/null 2>&1; then
  GROUP="$(node -e "
    const fs = require('fs');
    const conf = JSON.parse(fs.readFileSync(process.argv[1], 'utf8'));
    const g = conf.plugins && conf.plugins.widgets && conf.plugins.widgets.appGroup;
    if (!g) { console.error('ERROR: plugins.widgets.appGroup missing in', process.argv[1]); process.exit(1); }
    process.stdout.write(String(g));
  " "$CONF")"
  if [[ ! "$GROUP" =~ ^[A-Za-z0-9._-]+$ ]]; then
    echo "ERROR: invalid appGroup identifier: $GROUP" >&2
    exit 1
  fi
  gen_entitlements "$HOST_ENTS" "$GROUP" 0
  gen_entitlements "$EXT_ENTS" "$GROUP" 1
  echo "[widget] Regenerated entitlements for App Group: $GROUP"
  for swift in "$SCRIPT_DIR"/Sources/*.swift "$SCRIPT_DIR"/MyWidget.swift; do
    [[ -f "$swift" ]] || continue
    if grep -q 'let appGroup = "' "$swift" 2>/dev/null; then
      sed -i '' "s/let appGroup = \"[^\"]*\"/let appGroup = \"${GROUP}\"/" "$swift"
    fi
  done
  echo "[widget] Synced appGroup in Swift sources"
elif [[ -f "$EXT_ENTS" ]]; then
  echo "[widget] WARNING: could not read plugins.widgets.appGroup — using existing entitlements"
else
  echo "ERROR: need plugins.widgets.appGroup in tauri.conf.json (and node on PATH) to generate entitlements" >&2
  exit 1
fi

# ─── Generate Xcode project ─────────────────────────────────────────────────

if command -v xcodegen &>/dev/null; then
    echo "[widget] Generating Xcode project..."
    cd "$SCRIPT_DIR"
    xcodegen generate --spec project.yml 2>&1 | tail -3
    cd - > /dev/null
else
    echo "[widget] xcodegen not found, expecting .xcodeproj already exists"
fi

XCODEPROJ="$SCRIPT_DIR/TauriWidgetExtension.xcodeproj"

if [ ! -d "$XCODEPROJ" ]; then
    echo "ERROR: $XCODEPROJ not found. Install xcodegen: brew install xcodegen"
    exit 1
fi

# ─── Build (unsigned; we sign below with extension entitlements) ─────────────

echo "[widget] Building TauriWidgetExtension ($CONFIGURATION)..."

xcodebuild build \
    -project "$XCODEPROJ" \
    -scheme "TauriWidgetExtension" \
    -configuration "$CONFIGURATION" \
    -derivedDataPath "$DERIVED_DATA" \
    ONLY_ACTIVE_ARCH=NO \
    CODE_SIGN_IDENTITY="-" \
    CODE_SIGNING_REQUIRED=NO \
    CODE_SIGNING_ALLOWED=NO \
    2>&1 | tail -5

APPEX_PATH="$DERIVED_DATA/Build/Products/$CONFIGURATION/TauriWidgetExtension.appex"

if [ ! -d "$APPEX_PATH" ]; then
    echo "ERROR: .appex not found at $APPEX_PATH"
    exit 1
fi

# Strip accidental nested Frameworks (SwiftPM copy) — host app should not ship them twice.
rm -rf "$APPEX_PATH/Contents/Frameworks"

# ─── Sign .appex with WidgetKit entitlements ─────────────────────────────────

if [ -f "$EXT_ENTS" ]; then
    codesign --force --options runtime --sign "$IDENTITY" \
        --entitlements "$EXT_ENTS" \
        "$APPEX_PATH"
    echo "[widget] Signed .appex with entitlements ($IDENTITY)"
else
    echo "WARNING: Entitlements not found at $EXT_ENTS — signing without them"
    codesign --force --options runtime --sign "$IDENTITY" "$APPEX_PATH"
fi

echo "[widget] Built: $APPEX_PATH"
echo "[widget] Ready for bundle.macOS.files → Contents/PlugIns/"
