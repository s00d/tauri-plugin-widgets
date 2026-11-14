#!/usr/bin/env bash
# Playwright desktop tests in the pinned image CI uses (ci.yml `playwright` job).
# Linux Chromium goldens live in tests/golden/linux-chromium (not tests/golden/linux:
# that is the WebKitGTK host, `just record-linux`).
#
#   scripts/sh/playwright-linux.sh             # render + events specs, CI=1 (missing golden fails)
#   RECORD=1 scripts/sh/playwright-linux.sh    # (re)record every linux-chromium golden
#   CASE=weather.small RECORD=1 scripts/sh/playwright-linux.sh
#
# node_modules is an anonymous volume: the host's (macOS) native packages never mix in.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="$(node -p 'require(process.argv[1]).version' "$ROOT/node_modules/@playwright/test/package.json")"
IMAGE="mcr.microsoft.com/playwright:v${VERSION}-noble"

ENV=(-e CI=1 -e HOME=/tmp/home)
SPECS=""
if [ "${RECORD:-}" = "1" ]; then
  ENV+=(-e GOLDEN_RECORD=1)
  if [ -n "${CASE:-}" ]; then ENV+=(-e "CASE=${CASE}"); else ENV+=(-e GOLDEN_RECORD_ALL=1); fi
  SPECS="tests/desktop/render.spec.ts"
elif [ -n "${CASE:-}" ]; then
  ENV+=(-e "CASE=${CASE}")
fi

USER_ARGS=()
[ "$(uname -s)" = "Linux" ] && USER_ARGS=(--user "$(id -u):$(id -g)")

docker run --rm --ipc=host ${USER_ARGS[@]+"${USER_ARGS[@]}"} "${ENV[@]}" \
  -v "$ROOT:/work" -v /work/node_modules -w /work "$IMAGE" \
  bash -c "npx -y pnpm@9 install --frozen-lockfile --ignore-scripts >/dev/null \
    && npx -y pnpm@9 exec playwright test -c tests/desktop/playwright.config.ts ${SPECS}"
