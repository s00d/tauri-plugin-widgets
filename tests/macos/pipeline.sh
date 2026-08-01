#!/usr/bin/env bash
# Level C2 — macOS widget build + entitlements + embed smoke test.
# Slow (xcodebuild), but answers whether the macOS packaging path still works.
set -euo pipefail

REPO="$(cd "$(dirname "$0")/../.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "[pipeline] staging example → $TMP/app"
cp -R "$REPO/examples/tauri-plugin-widgets-example" "$TMP/app"
cd "$TMP/app"

# Drop pre-existing macos-widget so init-macos --force regenerates from templates.
rm -rf src-tauri/macos-widget

node "$REPO/bin/cli.mjs" init-macos \
  --force \
  --app-group group.test.app \
  --bundle-id test.app.widgetkit

test -x src-tauri/macos-widget/build-widget.sh \
  || { echo "FAIL: build-widget.sh not executable"; exit 1; }
test -x src-tauri/macos-widget/embed-widget.sh \
  || { echo "FAIL: embed-widget.sh not executable"; exit 1; }

echo "[pipeline] building .appex…"
./src-tauri/macos-widget/build-widget.sh

APPEX="$(find src-tauri/macos-widget -name '*.appex' -type d | head -1)"
[ -n "$APPEX" ] || { echo "FAIL: .appex was not built"; exit 1; }
echo "[pipeline] appex: $APPEX"

# Ad-hoc sign so codesign -dv --entitlements can dump the embedded entitlements.
codesign --force --sign - \
  --entitlements src-tauri/macos-widget/TauriWidgetExtension.entitlements \
  "$APPEX" 2>/dev/null || true

ENT_OUT="$TMP/ent.txt"
codesign -dv --entitlements - "$APPEX" 2>"$ENT_OUT" || true
# entitlements dump goes to stdout as XML for -dv --entitlements -
codesign -d --entitlements :- "$APPEX" >"$TMP/ent.xml" 2>/dev/null \
  || codesign -dv --entitlements - "$APPEX" >"$TMP/ent.xml" 2>&1

grep -q 'group\.test\.app' "$TMP/ent.xml" \
  || { echo "FAIL: App Group missing in entitlements"; cat "$TMP/ent.xml"; exit 1; }

plutil -extract NSExtension.NSExtensionPointIdentifier raw \
  "$APPEX/Contents/Info.plist" \
  | grep -q widgetkit-extension \
  || { echo "FAIL: NSExtensionPointIdentifier is not widgetkit-extension"; exit 1; }

# Stub .app so embed can run without a full `tauri build`.
PRODUCT_NAME="$(python3 -c "import json; print(json.load(open('src-tauri/tauri.conf.json'))['productName'])")"
APP_DIR="src-tauri/target/release/bundle/macos/${PRODUCT_NAME}.app"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"
printf '%s\n' \
  '<?xml version="1.0" encoding="UTF-8"?>' \
  '<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">' \
  '<plist version="1.0"><dict>' \
  '  <key>CFBundleIdentifier</key><string>test.app</string>' \
  '  <key>CFBundleName</key><string>'"$PRODUCT_NAME"'</string>' \
  '  <key>CFBundleExecutable</key><string>stub</string>' \
  '  <key>CFBundlePackageType</key><string>APPL</string>' \
  '</dict></plist>' >"$APP_DIR/Contents/Info.plist"
printf '#!/bin/sh\necho stub\n' >"$APP_DIR/Contents/MacOS/stub"
chmod +x "$APP_DIR/Contents/MacOS/stub"
# Minimal Mach-O stub is not required for embed path check; codesign may warn.

echo "[pipeline] embedding into stub .app…"
WIDGET_SKIP_DMG=1 ./src-tauri/macos-widget/embed-widget.sh

find . -path '*.app/Contents/PlugIns/*.appex' -print -quit | grep -q . \
  || { echo "FAIL: .appex not embedded under Contents/PlugIns"; exit 1; }

echo "[pipeline] OK"
