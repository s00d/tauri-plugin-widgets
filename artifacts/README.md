# Built example app binaries (local only — do not commit large APK/IPA/DMG).
#
# Generate with:
#   pnpm build:example
#   # or: bash tools/build-example-artifacts.sh macos ios android linux
#
# Layout:
#   artifacts/macos/    .app (+ raw binary)
#   artifacts/ios/      .app (device + sim), .xcarchive (IPA needs App Groups profile)
#   artifacts/android/  .apk (unsigned release)
#   artifacts/linux/    release binary (Docker image `widgets-linux`)
#   artifacts/windows/  build on a Windows host
#   artifacts/logs/     build logs
#   artifacts/STATUS.md last local build notes
#
# Prerequisites:
#   - macOS/iOS: Xcode + rustup ios targets
#   - Android: ANDROID_HOME + NDK + rustup android targets
#   - Linux: `docker build -t widgets-linux tests/linux` (or `bash tools/linux-up.sh`)
