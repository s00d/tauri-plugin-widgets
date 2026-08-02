#!/bin/bash
set -euo pipefail

# DEPRECATED: WidgetKit .appex is now embedded by `tauri build` via
#   build.beforeBundleCommand → build-widget.sh
#   bundle.macOS.files → Contents/PlugIns/TauriWidgetExtension.appex
#
# This script is kept for emergency re-sign / DMG rebuild only.
# Prefer: pnpm tauri build

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
IDENTITY="${WIDGET_SIGN_IDENTITY:-${APPLE_SIGNING_IDENTITY:-${1:--}}}"
CONFIGURATION="Release"
WIDGET_SCHEME="TauriWidgetExtension"
APPEX_PATH="$SCRIPT_DIR/build/Build/Products/$CONFIGURATION/${WIDGET_SCHEME}.appex"
ENTITLEMENTS="$SCRIPT_DIR/${WIDGET_SCHEME}.entitlements"
APP_ENTITLEMENTS="$SCRIPT_DIR/App.entitlements"

echo "[widget] WARNING: embed-widget.sh is deprecated."
echo "[widget]          Use \`pnpm tauri build\` — PlugIns are copied via bundle.macOS.files."

if [ ! -d "$APPEX_PATH" ]; then
    echo "[widget] No prebuilt .appex — running build-widget.sh…"
    "$SCRIPT_DIR/build-widget.sh" "$CONFIGURATION"
fi

PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PRODUCT_NAME=""
APP_VERSION=""

for CONF_PATH in "$PROJECT_ROOT/src-tauri/tauri.conf.json" "$PROJECT_ROOT/tauri.conf.json"; do
    if [ -f "$CONF_PATH" ]; then
        PRODUCT_NAME=$(python3 -c "
import json
d = json.load(open('$CONF_PATH'))
print(d.get('productName', d.get('package', {}).get('productName', '')))
" 2>/dev/null || true)
        APP_VERSION=$(python3 -c "
import json
d = json.load(open('$CONF_PATH'))
print(d.get('version', d.get('package', {}).get('version', '')))
" 2>/dev/null || true)
        break
    fi
done

if [ -z "$PRODUCT_NAME" ]; then
    echo "ERROR: Could not detect productName from tauri.conf.json"
    exit 1
fi

APP_BUNDLE=""
for SEARCH in \
    "$PROJECT_ROOT/src-tauri/target/release/bundle/macos/${PRODUCT_NAME}.app" \
    "$PROJECT_ROOT/target/release/bundle/macos/${PRODUCT_NAME}.app"; do
    if [ -d "$SEARCH" ]; then
        APP_BUNDLE="$SEARCH"
        break
    fi
done

if [ -z "$APP_BUNDLE" ] || [ ! -d "$APP_BUNDLE" ]; then
    echo "ERROR: App bundle not found. Run \`pnpm tauri build\` first."
    exit 1
fi

PLUGINS_DIR="$APP_BUNDLE/Contents/PlugIns"
mkdir -p "$PLUGINS_DIR"
rm -rf "$PLUGINS_DIR/${WIDGET_SCHEME}.appex"
cp -R "$APPEX_PATH" "$PLUGINS_DIR/"
rm -rf "$PLUGINS_DIR/${WIDGET_SCHEME}.appex/Contents/Frameworks"
echo "[widget] Embedded: $PLUGINS_DIR/${WIDGET_SCHEME}.appex"

if [ -f "$ENTITLEMENTS" ]; then
    codesign --force --options runtime --sign "$IDENTITY" \
             --entitlements "$ENTITLEMENTS" \
             "$PLUGINS_DIR/${WIDGET_SCHEME}.appex"
else
    codesign --force --options runtime --sign "$IDENTITY" \
             "$PLUGINS_DIR/${WIDGET_SCHEME}.appex"
fi

if [ -f "$APP_ENTITLEMENTS" ]; then
    codesign --force --options runtime --sign "$IDENTITY" \
             --entitlements "$APP_ENTITLEMENTS" \
             "$APP_BUNDLE"
else
    codesign --force --options runtime --sign "$IDENTITY" "$APP_BUNDLE"
fi

if [ "${WIDGET_SKIP_DMG:-}" = "1" ]; then
    echo "[widget] Skipping DMG rebuild (WIDGET_SKIP_DMG=1)"
    echo "[widget] Done!"
    exit 0
fi

echo "[widget] Done (DMG: use bundle.targets / normal tauri build)."
