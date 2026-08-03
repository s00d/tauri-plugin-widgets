#!/usr/bin/env bash
# Sync repo to the UTM Windows VM and run pack.ps1.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HOST="${UTM_WIN_HOST:-utm-win}"
bash "$ROOT/scripts/sh/win-up.sh"
bash "$ROOT/scripts/win/sync.sh"
scp "$ROOT/scripts/win/pack.ps1" "${HOST}:C:/work/pack.ps1"
ssh "$HOST" 'powershell -NoProfile -ExecutionPolicy Bypass -File C:\work\pack.ps1'
