#!/bin/bash
set -euo pipefail

# ─── macOS Widget Extension Builder ──────────────────────────────────────────
# Called automatically via tauri.conf.json → build.beforeBundleCommand.
#
# Builds and signs the .appex so `bundle.macOS.files` can copy it into
# Contents/PlugIns/ during the normal `tauri build` (Tauri nested-codesigns PlugIns).

# The hook lives in tauri.macos.conf.json; this guard covers older tauri.conf.json setups.
if [[ "$(uname)" != "Darwin" ]]; then
  echo "[widget] not macOS — skipping the WidgetKit extension build"
  exit 0
fi

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
  # Same rule as the plugin: plugins.widgets.appGroup (tauri.macos.conf.json wins), else group.<identifier>.
  GROUP="$(node -e "
    const fs = require('fs'), path = require('path');
    const read = (p) => (fs.existsSync(p) ? JSON.parse(fs.readFileSync(p, 'utf8')) : {});
    const conf = read(process.argv[1]);
    const mac = read(path.join(path.dirname(process.argv[1]), 'tauri.macos.conf.json'));
    const pick = (c) => c.plugins && c.plugins.widgets && c.plugins.widgets.appGroup;
    const g = pick(mac) || pick(conf) || (conf.identifier && 'group.' + conf.identifier);
    if (!g) { console.error('ERROR: no plugins.widgets.appGroup and no identifier in', process.argv[1]); process.exit(1); }
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

# ─── Point project.yml at the installed TauriWidgets Swift package ───────────
# Resolved on every build, so upgrades and other machines never see a stale path.

if command -v node >/dev/null 2>&1; then
  PKG_SWIFT="$(cd "$SCRIPT_DIR" && node -e "
    const path = require('path');
    try {
      const entry = require.resolve('tauri-plugin-widgets-api', { paths: [process.cwd()] });
      process.stdout.write(path.join(path.dirname(path.dirname(entry)), 'swift'));
    } catch { process.exit(1); }
  " 2>/dev/null || true)"
  if [[ -n "$PKG_SWIFT" && -d "$PKG_SWIFT" ]]; then
    sed -i '' -E "/^packages:/,/^targets:/ s|^( +path: ).*|\1$PKG_SWIFT|" "$SCRIPT_DIR/project.yml"
    echo "[widget] TauriWidgets package: $PKG_SWIFT"
  fi
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

# Extension version = app version, build number new on every build: WidgetKit caches an
# extension's widget list by version, so a fixed "1" hides new widgets and changes.
APP_VERSION="$(node -e "
  const fs = require('fs'), path = require('path');
  const conf = JSON.parse(fs.readFileSync(process.argv[1], 'utf8'));
  let v = conf.version || '1.0';
  if (v.endsWith('.json')) v = JSON.parse(fs.readFileSync(path.resolve(path.dirname(process.argv[1]), v), 'utf8')).version;
  process.stdout.write(String(v));
" "$CONF" 2>/dev/null || echo 1.0)"
BUILD_NUMBER="$(date +%s)"

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

# Version stamped into the built Info.plist (xcodegen writes literals, so build settings
# cannot do it). Before signing: the signature covers Info.plist.
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $APP_VERSION" \
    -c "Set :CFBundleVersion $BUILD_NUMBER" "$APPEX_PATH/Contents/Info.plist"

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
