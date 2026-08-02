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
ENTITLEMENTS="$SCRIPT_DIR/TauriWidgetExtension.entitlements"
IDENTITY="${WIDGET_SIGN_IDENTITY:-${APPLE_SIGNING_IDENTITY:--}}"

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

if [ -f "$ENTITLEMENTS" ]; then
    codesign --force --options runtime --sign "$IDENTITY" \
        --entitlements "$ENTITLEMENTS" \
        "$APPEX_PATH"
    echo "[widget] Signed .appex with entitlements ($IDENTITY)"
else
    echo "WARNING: Entitlements not found at $ENTITLEMENTS — signing without them"
    codesign --force --options runtime --sign "$IDENTITY" "$APPEX_PATH"
fi

echo "[widget] Built: $APPEX_PATH"
echo "[widget] Ready for bundle.macOS.files → Contents/PlugIns/"
