# Example build status (local)

Rebuilt: `2026-08-03T08:36Z` — release/0.5.0 mega-refactor.

| Platform | Artifact | Notes |
|----------|----------|-------|
| macOS | `macos/widget-example.app`, binary, `widget-example_0.5.0_aarch64.dmg` | OK |
| iOS | `ios/widget-example-iphoneos.app`, `iphonesimulator.app`, `.ipa` | OK |
| Android | `android/app-universal-release-unsigned.apk` | OK (unsigned, aarch64) |
| Linux | `linux/widget-example` (+ `tauri-plugin-widgets-example`) | OK (Docker `widgets-linux`, aarch64 ELF) |
| Windows | — | SKIP — UTM `utm-win` SSH unreachable |

Logs under `artifacts/logs/rebuild-all-*`, `rebuild-linux-*`, `rebuild-macos-*`.
