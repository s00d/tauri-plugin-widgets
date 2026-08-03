#!/usr/bin/env bash
# Bring up the UTM Windows VM and run bootstrap.ps1 remotely.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HOST="${UTM_WIN_HOST:-utm-win}"
bash "$ROOT/scripts/sh/win-up.sh"
scp "$ROOT/scripts/win/bootstrap.ps1" "${HOST}:C:/work/bootstrap.ps1"
ssh "$HOST" 'powershell -NoProfile -ExecutionPolicy Bypass -File C:\work\bootstrap.ps1'
