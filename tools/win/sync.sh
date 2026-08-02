#!/usr/bin/env bash
# Sync this repo to the UTM Windows VM working tree (ssh host: utm-win).
# Uses tar+scp — Windows OpenSSH typically has no rsync server.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HOST="${UTM_WIN_HOST:-utm-win}"

echo "==> ensuring remote dir on $HOST"
ssh "$HOST" powershell -NoProfile -Command \
  "New-Item -ItemType Directory -Force -Path 'C:\work\tauri-plugin-widgets' | Out-Null"

echo "==> tar+scp → ${HOST}:C:\\work\\tauri-plugin-widgets"
TMP="$(mktemp -t tpw-sync.XXXXXX.tgz)"
trap 'rm -f "$TMP"' EXIT
# GNU/BSD tar: exclude build artifacts
tar -C "$ROOT" \
  --exclude '.git' \
  --exclude 'target' \
  --exclude 'node_modules' \
  --exclude 'android/.gradle' \
  --exclude 'android/build' \
  --exclude '._*' \
  --exclude '.DS_Store' \
  -czf "$TMP" .

scp "$TMP" "${HOST}:C:/work/tpw-sync.tgz"

# Wipe remote tree before extract so deleted local files don't linger.
# Pass -Command as its own argv (no nested bash single-quotes) so PowerShell
# keeps the path string intact.
ssh "$HOST" powershell -NoProfile -Command \
  "\$dest = 'C:\work\tauri-plugin-widgets'; if (Test-Path \$dest) { Remove-Item -Recurse -Force \$dest }; New-Item -ItemType Directory -Force -Path \$dest | Out-Null; tar -xzf C:\work\tpw-sync.tgz -C \$dest"

echo "==> install C:\\work\\shot.ps1"
scp "$(cd "$(dirname "$0")" && pwd)/shot.ps1" "${HOST}:C:/work/shot.ps1"

echo "sync: OK → ${HOST}:C:\\work\\tauri-plugin-widgets"
