# Reproducible visual stand — local hosts + case-driven goldens.

repo := justfile_directory()

android-up:
    bash {{repo}}/tools/android-up.sh

ios-up:
    bash {{repo}}/tools/ios-up.sh

# Level-1 geometry (Robolectric / Playwright tree) — unchanged bulk update OK.
test-geometry:
    pnpm test:geometry

test-desktop-geometry:
    pnpm test:desktop

# Level-2 visual (declarative cases → golden PNG).
test-desktop-visual:
    pnpm test:visual:desktop

test-android-visual:
    cd {{repo}}/android && ./gradlew :connectedDebugAndroidTest \
      -Pandroid.testInstrumentationRunnerArguments.class=git.s00d.widgets.WidgetRenderTest

test-ios-visual:
    cd {{repo}}/swift && swift test --filter RenderTests

# Level A — macOS AppKit DynamicElementView (no sim, seconds).
test-macos-visual:
    cd {{repo}}/swift && swift test --filter MacRenderTests

# Level C1 — Rust + Swift transports (config driver + file IO).
test-macos-transports:
    cargo test --lib transport::
    cargo test --test macos_transports -- --nocapture
    cd {{repo}}/swift && swift test --filter TransportTests

# Level C2 — init-macos → build .appex → entitlements → embed into stub .app.
test-macos-pipeline:
    bash {{repo}}/tests/macos/pipeline.sh

audit-sheets:
    pnpm audit:sheets

# Record exactly one case (required).
record-desktop case:
    CASE={{case}} GOLDEN_RECORD=1 pnpm test:visual:desktop

record-android case:
    cd {{repo}}/android && ./gradlew :connectedDebugAndroidTest \
      -Pgolden.record=true \
      -Pcase={{case}} \
      -Pandroid.testInstrumentationRunnerArguments.class=git.s00d.widgets.WidgetRenderTest
    @echo "Pull recorded PNGs:"
    @echo "  adb shell 'run-as git.s00d.widgets.test cat ...'  # or:"
    @echo "  adb pull /storage/emulated/0/Android/data/git.s00d.widgets.test/files/widgets-golden/ ./tests/golden/android/"

record-ios case:
    cd {{repo}}/swift && CASE={{case}} GOLDEN_RECORD=1 swift test --filter RenderTests

record-macos case:
    cd {{repo}}/swift && CASE={{case}} GOLDEN_RECORD=1 swift test --filter MacRenderTests

# ─── Windows / UTM (optional remote node) ───────────────────────────────────

win-up:
    bash {{repo}}/tools/win-up.sh

win-sync:
    bash {{repo}}/tools/win/sync.sh

win-bootstrap:
    bash {{repo}}/tools/win-up.sh
    scp {{repo}}/tools/win/bootstrap.ps1 utm-win:C:/work/bootstrap.ps1
    ssh utm-win 'powershell -NoProfile -ExecutionPolicy Bypass -File C:\work\bootstrap.ps1'

# Adaptive Card transpile tests on the Mac host (no VM required).
test-windows-adaptive:
    cargo test --lib adaptive_card -- --nocapture

# Remote smoke via ssh utm-win (after win-up + win-sync).
test-windows-remote:
    bash {{repo}}/tools/win-up.sh
    bash {{repo}}/tools/win/sync.sh
    ssh utm-win 'powershell -NoProfile -ExecutionPolicy Bypass -File C:\work\shot.ps1'

check-windows-xwin:
    rustup target add x86_64-pc-windows-msvc
    cargo xwin check --target x86_64-pc-windows-msvc --lib

build-windows-example-xwin:
    rustup target add x86_64-pc-windows-msvc
    cd {{repo}}/examples/tauri-plugin-widgets-example/src-tauri && cargo xwin build --target x86_64-pc-windows-msvc

record-windows case:
    bash {{repo}}/tools/win-up.sh
    bash {{repo}}/tools/win/sync.sh
    ssh utm-win "powershell -NoProfile -ExecutionPolicy Bypass -File C:\\work\\shot.ps1 -Mode visual -Case {{case}} -Record"
    mkdir -p {{repo}}/tests/golden/windows
    scp "utm-win:C:/work/tauri-plugin-widgets/out/windows/{{case}}.png" "{{repo}}/tests/golden/windows/{{case}}.png" || \
      scp "utm-win:C:/work/tauri-plugin-widgets/tests/golden/windows/{{case}}.png" "{{repo}}/tests/golden/windows/{{case}}.png"

# Dump Adaptive Card JSON snapshots (transpile). PNG goldens: record-windows via PreviewHost.
record-adaptive-snapshots:
    pnpm -C {{repo}}/scripts cli gen-adaptive-snapshots

# Alias kept for muscle memory — dumps AC JSON only (no fake Mac SVG PNGs).
record-windows-all:
    @echo "PNG goldens come from PreviewHost (just record-windows <case>). Dumping AC JSON…"
    pnpm -C {{repo}}/scripts cli gen-adaptive-snapshots

test-windows-visual:
    bash {{repo}}/tools/win-up.sh
    bash {{repo}}/tools/win/sync.sh
    ssh utm-win 'powershell -NoProfile -ExecutionPolicy Bypass -File C:\work\shot.ps1 -Mode visual'

win-pack:
    bash {{repo}}/tools/win-up.sh
    bash {{repo}}/tools/win/sync.sh
    scp {{repo}}/tools/win/pack.ps1 utm-win:C:/work/pack.ps1
    ssh utm-win 'powershell -NoProfile -ExecutionPolicy Bypass -File C:\work\pack.ps1'

win-sideload:
    scp {{repo}}/tools/win/sideload.ps1 utm-win:C:/work/sideload.ps1
    ssh utm-win 'powershell -NoProfile -ExecutionPolicy Bypass -File C:\work\sideload.ps1'

# ─── Linux / Docker (native arm64; X11 xprop + optional Wayland layer-shell) ─

linux-up:
    bash {{repo}}/tools/linux-up.sh

linux-up-wl:
    bash {{repo}}/tools/linux-up-wl.sh

linux-down:
    -docker rm -f wshot wshot-wl

# Property gate (xprop) — one-shot container (does not use wshot watch probe).
test-linux-x11:
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="{{repo}}"
    IMAGE="${WSHOT_IMAGE:-widgets-linux}"
    if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
      docker build -t "$IMAGE" "$ROOT/tests/linux"
    fi
    docker volume create widgets-cargo >/dev/null 2>&1 || true
    docker run --rm \
      -v "$ROOT:/work" \
      -v widgets-cargo:/root/.cargo/registry \
      -v widgets-cargo-git:/root/.cargo/git \
      "$IMAGE" bash tests/linux/run-x11.sh

test-linux-wayland:
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="{{repo}}"
    IMAGE="${WSHOT_IMAGE:-widgets-linux}"
    if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
      docker build -t "$IMAGE" "$ROOT/tests/linux"
    fi
    docker volume create widgets-cargo >/dev/null 2>&1 || true
    docker run --rm \
      -v "$ROOT:/work" \
      -v widgets-cargo:/root/.cargo/registry \
      -v widgets-cargo-git:/root/.cargo/git \
      "$IMAGE" bash tests/linux/run-wayland.sh

test-linux-fallback:
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="{{repo}}"
    IMAGE="${WSHOT_IMAGE:-widgets-linux}"
    if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
      docker build -t "$IMAGE" "$ROOT/tests/linux"
    fi
    docker volume create widgets-cargo >/dev/null 2>&1 || true
    docker run --rm \
      -v "$ROOT:/work" \
      -v widgets-cargo:/root/.cargo/registry \
      -v widgets-cargo-git:/root/.cargo/git \
      "$IMAGE" bash tests/linux/run-wayland-fallback.sh

shot-linux FX SIZE="small":
    bash {{repo}}/tools/shot-linux.sh {{FX}} {{SIZE}}

# Copy triage capture into golden corpus (explicit).
record-linux case:
    #!/usr/bin/env bash
    set -euo pipefail
    CASE={{case}}
    SIZE="$(jq -r '.size // "small"' "{{repo}}/tests/cases/${CASE}.json" 2>/dev/null || echo small)"
    FX="$(jq -r '.fixture // empty' "{{repo}}/tests/cases/${CASE}.json" 2>/dev/null || true)"
    bash {{repo}}/tools/linux-up.sh
    if [[ -n "$FX" ]]; then
      bash {{repo}}/tools/shot-linux.sh "$CASE" "$SIZE"
    else
      bash {{repo}}/tools/shot-linux.sh "$CASE" "$SIZE"
    fi
    mkdir -p "{{repo}}/tests/golden/linux"
    SRC="{{repo}}/out/linux/${CASE}-${SIZE}.png"
    [[ -f "$SRC" ]] || { echo "record-linux: expected PNG missing: $SRC"; exit 1; }
    cp "$SRC" "{{repo}}/tests/golden/linux/${CASE}.png"
    echo "record-linux: tests/golden/linux/${CASE}.png"

hosts:
    #!/usr/bin/env bash
    set -euo pipefail
    xcrun simctl list devices booted 2>/dev/null | grep -q Booted && echo "ios     ok" || echo "ios     DOWN"
    adb devices 2>/dev/null | grep -q emulator && echo "android ok" || echo "android DOWN"
    docker ps -q -f name=^/wshot$ | grep -q . && echo "linux   ok" || echo "linux   DOWN"
    docker ps -q -f name=^/wshot-wl$ | grep -q . && echo "linux-wl ok" || echo "linux-wl DOWN"
    ssh -o ConnectTimeout=2 -o BatchMode=yes utm-win exit 2>/dev/null && echo "windows ok" || echo "windows DOWN"
    pgrep -f widget-probe >/dev/null 2>&1 && echo "desktop ok" || echo "desktop DOWN"

