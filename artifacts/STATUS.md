# Example build status (local)

Rebuilt: `2026-08-02` via `bash tools/build-example-artifacts.sh`  
macOS: plain `pnpm tauri build` — `Contents/PlugIns/TauriWidgetExtension.appex` present (no `embed-widget.sh`).

| Platform | Artifact | Notes |
|----------|----------|-------|
| macOS | `macos/widget-example.app`, `.dmg`, binary | OK — PlugIns + notarized |
| iOS device | `ios/widget-example-iphoneos.app`, `.ipa` | OK |
| iOS sim | `ios/widget-example-iphonesimulator.app` | OK |
| Android | `android/app-universal-release-unsigned.apk` | OK (unsigned) |
| Linux | — | SKIP — need `widgets-linux` Docker image |
| Windows | — | SKIP — build on Windows host |

Rebuild: `bash tools/build-example-artifacts.sh` (or `pnpm build:example` if wired).  
Binaries under `artifacts/{macos,ios,…}` are gitignored; only this file + `README.md` are tracked.
