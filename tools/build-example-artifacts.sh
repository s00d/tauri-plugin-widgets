#!/usr/bin/env bash
# Build the example app for available platforms and copy artifacts to <repo>/artifacts/.
# Usage:
#   bash tools/build-example-artifacts.sh            # all (skip unavailable)
#   bash tools/build-example-artifacts.sh macos ios android
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXAMPLE="$ROOT/examples/tauri-plugin-widgets-example"
OUT="$ROOT/artifacts"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"

mkdir -p "$OUT"/{macos,ios,android,linux,windows,logs}
cd "$EXAMPLE"

WANT=("$@")
if [[ ${#WANT[@]} -eq 0 ]]; then
  WANT=(macos ios android linux windows)
fi

want() {
  local t="$1"
  for w in "${WANT[@]}"; do [[ "$w" == "$t" ]] && return 0; done
  return 1
}

log() { echo "==> $*"; }
fail_log() {
  local platform="$1"
  echo "$2" | tee "$OUT/logs/${platform}-${STAMP}.log" >&2
}

copy_glob() {
  local dest="$1"; shift
  mkdir -p "$dest"
  local found=0
  for pattern in "$@"; do
    # shellcheck disable=SC2086
    for f in $pattern; do
      if [[ -e "$f" ]]; then
        cp -R "$f" "$dest/"
        found=1
        log "copied $(basename "$f") → ${dest#"$ROOT"/}"
      fi
    done
  done
  [[ "$found" -eq 1 ]]
}

# Ensure JS API is built for workspace link.
log "build plugin JS (root)"
(cd "$ROOT" && pnpm build)

# ── macOS ──────────────────────────────────────────────────────────────────
if want macos; then
  log "macOS: tauri build (PlugIns via bundle.macOS.files)"
  if pnpm tauri build 2>&1 | tee "$OUT/logs/macos-${STAMP}.log"; then
    APPEX="src-tauri/target/release/bundle/macos/widget-example.app/Contents/PlugIns/TauriWidgetExtension.appex"
    if [[ ! -d "$APPEX" ]]; then
      fail_log macos "macOS build OK but PlugIns/.appex missing — check beforeBundleCommand + macOS.files"
    else
      log "macOS: found $APPEX"
      copy_glob "$OUT/macos" \
        "src-tauri/target/release/bundle/macos/*.app" \
        "src-tauri/target/release/bundle/dmg/*.dmg" \
        "src-tauri/target/release/widget-example" \
        "src-tauri/target/release/tauri-plugin-widgets-example" \
        || log "macOS: build ok but no bundle files found (check logs)"
    fi
  else
    fail_log macos "macOS build FAILED"
  fi
fi

# ── iOS ────────────────────────────────────────────────────────────────────
if want ios; then
  log "iOS: tauri ios build (device aarch64 + sim) — real codesign"
  # Xcodegen baked Externals/arm64/debug/libapp.a into Resources; release
  # builds write to Externals/.../release/. Symlink so CpResource finds it.
  for arch in arm64 x86_64; do
    mkdir -p "src-tauri/gen/apple/Externals/$arch/"{debug,release}
    ln -sfn "../release/libapp.a" "src-tauri/gen/apple/Externals/$arch/debug/libapp.a"
  done
  # Align Widget Extension team with the app (pbxproj can drift).
  PBX="src-tauri/gen/apple/tauri-plugin-widgets-example.xcodeproj/project.pbxproj"
  if [[ -f "$PBX" ]]; then
    sed -i '' 's/DEVELOPMENT_TEAM = WXS9GV3T26;/DEVELOPMENT_TEAM = R6VC7JTLE8;/g' "$PBX" || true
  fi
  # CRITICAL: if APPLE_API_KEY* are set, tauri-cli skips real codesign and stamps
  # a dummy "Apple Distribution: Tauri (unset)" identity. Unset for local signing.
  # Keep APPLE_DEVELOPMENT_TEAM for Automatic signing.
  export APPLE_DEVELOPMENT_TEAM="${APPLE_DEVELOPMENT_TEAM:-R6VC7JTLE8}"
  unset APPLE_API_KEY APPLE_API_ISSUER APPLE_API_KEY_PATH || true

  ios_ok=0
  if pnpm tauri ios build --target aarch64 --export-method debugging --ci \
      2>&1 | tee "$OUT/logs/ios-${STAMP}.log"; then
    ios_ok=1
  else
    log "iOS: device IPA export failed — harvesting .app/.xcarchive if present"
  fi
  if pnpm tauri ios build --target aarch64-sim --ci \
      2>&1 | tee -a "$OUT/logs/ios-${STAMP}.log"; then
    ios_ok=1
  else
    log "iOS: simulator build failed — see log"
  fi

  # Harvest whatever Xcode / tauri produced
  copy_glob "$OUT/ios" \
    "src-tauri/gen/apple/build/arm64/*.ipa" \
    "src-tauri/gen/apple/build/arm64-sim/*.app" \
    "src-tauri/gen/apple/build/**/*.ipa" \
    "src-tauri/gen/apple/build/**/*.xcarchive" \
    "src-tauri/gen/apple/**/*.ipa" \
    || true
  archive_app="src-tauri/gen/apple/build/tauri-plugin-widgets-example_iOS.xcarchive/Products/Applications/widget-example.app"
  if [[ -d "$archive_app" ]]; then
    rm -rf "$OUT/ios/widget-example-iphoneos.app"
    cp -R "$archive_app" "$OUT/ios/widget-example-iphoneos.app"
    log "copied widget-example-iphoneos.app → artifacts/ios/"
    ios_ok=1
  fi
  find "$HOME/Library/Developer/Xcode/DerivedData" -path '*tauri-plugin-widgets-example*' \
    -path '*iphonesimulator*' -name 'widget-example.app' -type d 2>/dev/null \
    | head -2 \
    | while read -r f; do
        rm -rf "$OUT/ios/widget-example-iphonesimulator.app"
        cp -R "$f" "$OUT/ios/widget-example-iphonesimulator.app"
        log "copied widget-example-iphonesimulator.app → artifacts/ios/"
      done
  [[ -d "$OUT/ios/widget-example-iphonesimulator.app" ]] && ios_ok=1

  if [[ "$ios_ok" -ne 1 ]]; then
    fail_log ios "iOS build FAILED — see artifacts/logs/ios-*.log"
  fi
fi

# ── Android ────────────────────────────────────────────────────────────────
if want android; then
  log "Android: tauri android build --apk (aarch64)"
  export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
  export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-}"
  if [[ -z "${ANDROID_NDK_HOME}" && -d "$ANDROID_HOME/ndk" ]]; then
    ANDROID_NDK_HOME="$(ls -d "$ANDROID_HOME/ndk"/* 2>/dev/null | sort -V | tail -1 || true)"
    export ANDROID_NDK_HOME
  fi
  if pnpm tauri android build --apk --target aarch64 2>&1 | tee "$OUT/logs/android-${STAMP}.log"; then
    copy_glob "$OUT/android" \
      "src-tauri/gen/android/app/build/outputs/apk/**/*.apk" \
      "src-tauri/gen/android/app/build/outputs/bundle/**/*.aab" \
      || true
    find src-tauri/gen/android -name '*.apk' -o -name '*.aab' 2>/dev/null \
      | while read -r f; do
          cp -R "$f" "$OUT/android/" 2>/dev/null || true
          log "copied $(basename "$f") → artifacts/android/"
        done
  else
    fail_log android "Android build FAILED — see artifacts/logs/android-*.log"
  fi
fi

# ── Linux (Docker, native arch) ────────────────────────────────────────────
if want linux; then
  if command -v docker >/dev/null 2>&1 && docker image inspect widgets-linux >/dev/null 2>&1; then
    log "Linux: Docker cargo build --release (binary only; no .deb)"
    # Host must provide dist/ (image has no Node).
    if [[ ! -d "$EXAMPLE/dist" ]]; then
      log "Linux: building frontend on host"
      (cd "$EXAMPLE" && pnpm build)
    fi
    linux_script="$(mktemp)"
    cat >"$linux_script" <<'EOS'
set -euo pipefail
cd /work/examples/tauri-plugin-widgets-example
if [[ ! -d dist ]]; then
  echo "ERROR: dist/ missing — build frontend on host first" >&2
  exit 1
fi
cargo build --release --manifest-path src-tauri/Cargo.toml
mkdir -p /work/artifacts/linux
cp -f /tmp/example-target/release/widget-example /work/artifacts/linux/ 2>/dev/null \
  || cp -f /tmp/example-target/release/tauri-plugin-widgets-example /work/artifacts/linux/ 2>/dev/null \
  || find /tmp/example-target/release -maxdepth 1 -type f -executable -exec cp -f {} /work/artifacts/linux/ \;
ls -lah /work/artifacts/linux/
EOS
    if docker run --rm \
      -v "$ROOT:/work" \
      -v "$linux_script:/tmp/build-linux.sh:ro" \
      -v widgets-cargo:/root/.cargo/registry \
      -v widgets-cargo-git:/root/.cargo/git \
      -e CARGO_TARGET_DIR=/tmp/example-target \
      -e CARGO_INCREMENTAL=0 \
      -e CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}" \
      -e RUSTFLAGS="${RUSTFLAGS:--C link-arg=-fuse-ld=lld}" \
      widgets-linux bash /tmp/build-linux.sh \
      2>&1 | tee "$OUT/logs/linux-${STAMP}.log"
    then
      log "Linux: ok"
    else
      fail_log linux "Linux Docker build FAILED"
    fi
    rm -f "$linux_script"
  else
    log "Linux: SKIP (need docker image widgets-linux — run: bash tools/linux-up.sh once / docker build -t widgets-linux tests/linux)"
  fi
fi

# ── Windows ────────────────────────────────────────────────────────────────
if want windows; then
  log "Windows: SKIP on this host (needs Windows / cargo-xwin with full WebView2 toolchain)"
  echo "Build on Windows: cd examples/tauri-plugin-widgets-example && pnpm tauri build" \
    | tee "$OUT/logs/windows-${STAMP}.log"
fi

log "done — tree:"
find "$OUT" -maxdepth 2 \( -type f -o -type d \) | sort | sed "s|$ROOT/||"
