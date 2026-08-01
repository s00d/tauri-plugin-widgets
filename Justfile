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

# Level C1 — Rust + Swift multi-transport (fake TransportSet + file IO).
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

# Mac-side: regenerate all 36 Windows Adaptive Card goldens (no UTM).
record-windows-all:
    bash {{repo}}/tools/gen-windows-goldens.sh

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
