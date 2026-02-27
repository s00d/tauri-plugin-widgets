#!/bin/bash
set -euo pipefail

# ─── Embed Widget Extension into Tauri App Bundle ────────────────────────────
# Run AFTER `tauri build` completes.
#
# Usage:
#   ./src-tauri/macos-widget/embed-widget.sh                  # ad-hoc signing (local dev)
#   ./src-tauri/macos-widget/embed-widget.sh "Developer ID"   # production signing
#
# This script:
#   1. Finds the .appex built by build-widget.sh
#   2. Copies it into YourApp.app/Contents/PlugIns/
#   3. Signs the widget with its own entitlements
#   4. Re-signs the entire app bundle (with hardened runtime for notarization)
#   5. Rebuilds the .dmg so it contains the app WITH the widget

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
IDENTITY="${WIDGET_SIGN_IDENTITY:-${1:--}}"
CONFIGURATION="Release"
WIDGET_SCHEME="TauriWidgetExtension"
APPEX_PATH="$SCRIPT_DIR/build/Build/Products/$CONFIGURATION/${WIDGET_SCHEME}.appex"
ENTITLEMENTS="$SCRIPT_DIR/${WIDGET_SCHEME}.entitlements"
APP_ENTITLEMENTS="$SCRIPT_DIR/App.entitlements"

if [ ! -d "$APPEX_PATH" ]; then
    echo "ERROR: .appex not found at $APPEX_PATH"
    echo "Run 'tauri build' first (it triggers build-widget.sh via beforeBundleCommand)."
    exit 1
fi

# ─── Detect product name and version from tauri.conf.json ─────────────────────

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

# ─── Find .app bundle ────────────────────────────────────────────────────────

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
    echo "ERROR: App bundle not found."
    echo "Expected: src-tauri/target/release/bundle/macos/${PRODUCT_NAME}.app"
    echo "Make sure 'tauri build' completed successfully."
    exit 1
fi

# ─── Embed .appex ─────────────────────────────────────────────────────────────

PLUGINS_DIR="$APP_BUNDLE/Contents/PlugIns"
mkdir -p "$PLUGINS_DIR"
rm -rf "$PLUGINS_DIR/${WIDGET_SCHEME}.appex"
cp -R "$APPEX_PATH" "$PLUGINS_DIR/"

rm -rf "$PLUGINS_DIR/${WIDGET_SCHEME}.appex/Contents/Frameworks"

echo "[widget] Embedded: $PLUGINS_DIR/${WIDGET_SCHEME}.appex"

# ─── Sign ─────────────────────────────────────────────────────────────────────
# Order matters: sign inner components first, then the outer app WITHOUT --deep
# (--deep would re-sign the .appex without its entitlements)

if [ -f "$ENTITLEMENTS" ]; then
    codesign --force --options runtime --sign "$IDENTITY" \
             --entitlements "$ENTITLEMENTS" \
             "$PLUGINS_DIR/${WIDGET_SCHEME}.appex"
    echo "[widget] Signed .appex with entitlements ($IDENTITY)"
else
    echo "WARNING: Entitlements file not found at $ENTITLEMENTS"
    echo "Widget requires com.apple.security.app-sandbox to be loaded by macOS!"
    codesign --force --options runtime --sign "$IDENTITY" \
             "$PLUGINS_DIR/${WIDGET_SCHEME}.appex"
fi

# Re-sign the app with its own entitlements (without --deep to preserve the widget's signature)
if [ -f "$APP_ENTITLEMENTS" ]; then
    codesign --force --options runtime --sign "$IDENTITY" \
             --entitlements "$APP_ENTITLEMENTS" \
             "$APP_BUNDLE"
    echo "[widget] Re-signed app with entitlements ($IDENTITY)"
else
    codesign --force --options runtime --sign "$IDENTITY" "$APP_BUNDLE"
    echo "[widget] Re-signed app ($IDENTITY) — no App.entitlements found"
fi

# ─── Rebuild DMG ──────────────────────────────────────────────────────────────

ARCH=$(uname -m)
case "$ARCH" in
    arm64)  ARCH_LABEL="aarch64" ;;
    x86_64) ARCH_LABEL="x86_64" ;;
    *)      ARCH_LABEL="$ARCH" ;;
esac

DMG_DIR="$(dirname "$APP_BUNDLE")/../dmg"
mkdir -p "$DMG_DIR"
DMG_NAME="${PRODUCT_NAME}_${APP_VERSION}_${ARCH_LABEL}.dmg"
DMG_PATH="$DMG_DIR/$DMG_NAME"

if [ -n "$APP_VERSION" ]; then
    echo "[widget] Rebuilding DMG..."

    DMG_STAGE=$(mktemp -d)
    cp -R "$APP_BUNDLE" "$DMG_STAGE/"
    ln -s /Applications "$DMG_STAGE/Applications"

    rm -f "$DMG_PATH"
    hdiutil create \
        -volname "$PRODUCT_NAME" \
        -srcfolder "$DMG_STAGE" \
        -ov -format UDZO \
        "$DMG_PATH" \
        2>&1 | tail -2

    rm -rf "$DMG_STAGE"

    echo "[widget] DMG ready: $DMG_PATH"
    open "$DMG_PATH"
else
    echo "[widget] Skipping DMG rebuild (version not found)"
fi

echo "[widget] Done!"
