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
