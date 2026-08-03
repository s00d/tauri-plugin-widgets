#!/usr/bin/env bash
# Sideload the packed Windows widget package on the UTM VM.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
HOST="${UTM_WIN_HOST:-utm-win}"
scp "$ROOT/scripts/win/sideload.ps1" "${HOST}:C:/work/sideload.ps1"
ssh "$HOST" 'powershell -NoProfile -ExecutionPolicy Bypass -File C:\work\sideload.ps1'
