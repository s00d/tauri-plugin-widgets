#!/usr/bin/env bash
# Level C2 — macOS widget build + entitlements + macOS.files smoke test.
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

# Conf must wire PlugIns via bundle.macOS.files (no || true, no forced targets=["app"]).
python3 - <<'PY'
import json, sys
c = json.load(open("src-tauri/tauri.conf.json"))
bb = c.get("build", {}).get("beforeBundleCommand", "")
if "|| true" in bb:
    print("FAIL: beforeBundleCommand still has || true:", bb); sys.exit(1)
if "build-widget.sh" not in bb:
    print("FAIL: beforeBundleCommand missing build-widget.sh:", bb); sys.exit(1)
mac = c.get("bundle", {}).get("macOS", {})
files = mac.get("files") or {}
key = "PlugIns/TauriWidgetExtension.appex"
if key not in files:
    print("FAIL: bundle.macOS.files missing", key, files); sys.exit(1)
ent = mac.get("entitlements", "")
if "App.entitlements" not in str(ent):
    print("FAIL: bundle.macOS.entitlements missing App.entitlements:", ent); sys.exit(1)
print("[pipeline] conf OK:", bb, files[key], ent)
PY

echo "[pipeline] building .appex…"
./src-tauri/macos-widget/build-widget.sh

APPEX="$(find src-tauri/macos-widget -name '*.appex' -type d | head -1)"
[ -n "$APPEX" ] || { echo "FAIL: .appex was not built"; exit 1; }
echo "[pipeline] appex: $APPEX"

# build-widget.sh already signs; dump entitlements
codesign -d --entitlements :- "$APPEX" >"$TMP/ent.xml" 2>/dev/null \
  || codesign -dv --entitlements - "$APPEX" >"$TMP/ent.xml" 2>&1

grep -q 'group\.test\.app' "$TMP/ent.xml" \
  || { echo "FAIL: App Group missing in entitlements"; cat "$TMP/ent.xml"; exit 1; }

plutil -extract NSExtension.NSExtensionPointIdentifier raw \
  "$APPEX/Contents/Info.plist" \
  | grep -q widgetkit-extension \
  || { echo "FAIL: NSExtensionPointIdentifier is not widgetkit-extension"; exit 1; }

# Simulate Tauri bundle.macOS.files: copy .appex into Contents/PlugIns/
PRODUCT_NAME="$(python3 -c "import json; print(json.load(open('src-tauri/tauri.conf.json'))['productName'])")"
APP_DIR="src-tauri/target/release/bundle/macos/${PRODUCT_NAME}.app"
PLUG_KEY="PlugIns/TauriWidgetExtension.appex"
APPEX_SRC="$(python3 -c "import json; print(json.load(open('src-tauri/tauri.conf.json'))['bundle']['macOS']['files']['$PLUG_KEY'])")"
# Path in conf is relative to src-tauri/
APPEX_ABS="src-tauri/${APPEX_SRC#./}"
[ -d "$APPEX_ABS" ] || { echo "FAIL: files source missing: $APPEX_ABS"; exit 1; }

mkdir -p "$APP_DIR/Contents/$(dirname "$PLUG_KEY")"
rm -rf "$APP_DIR/Contents/$PLUG_KEY"
cp -R "$APPEX_ABS" "$APP_DIR/Contents/$PLUG_KEY"

find . -path '*.app/Contents/PlugIns/*.appex' -print -quit | grep -q . \
  || { echo "FAIL: .appex not under Contents/PlugIns after files-copy sim"; exit 1; }

echo "[pipeline] OK"
