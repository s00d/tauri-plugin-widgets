#!/bin/bash
set -euo pipefail

# ─── macOS Widget Extension Builder ──────────────────────────────────────────
# Called automatically via tauri.conf.json → build.beforeBundleCommand.
#
# Builds the .appex WITHOUT code-signing (signing is deferred to
# embed-widget.sh so the whole bundle is signed once, correctly).
#
# After `tauri build` finishes, run embed-widget.sh to inject the .appex
# into the .app bundle and re-sign everything.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CONFIGURATION="${1:-Release}"
DERIVED_DATA="$SCRIPT_DIR/build"

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

# ─── Build (no code signing — deferred to embed step) ────────────────────────

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

echo "[widget] Built: $APPEX_PATH"
