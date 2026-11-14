#!/usr/bin/env bash
# Record the docs demos (tests/demos/record.spec.ts: the real desktop renderer in headless
# Chromium — nothing touches your screen) and convert them for the site:
#   docs/public/media/<name>.mp4  (H.264, faststart: plays everywhere, starts before it loads)
#   docs/public/media/<name>.jpg  (poster, also shown with prefers-reduced-motion)
# Optional: DEMO=island-states (one recording, a Playwright -g filter).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
RAW="node_modules/.cache/demos"
OUT="docs/public/media"
LOG="node_modules/.cache/logs"
mkdir -p "$OUT" "$LOG"
rm -rf "$RAW"

pnpm build:widget > "$LOG/demos-build-widget.log" 2>&1
npx playwright test -c tests/demos/playwright.config.ts ${DEMO:+-g "$DEMO"} > "$LOG/demos-record.log" 2>&1 || {
  echo "record-demos: recording failed, see $LOG/demos-record.log" >&2; exit 1; }

for webm in "$RAW"/*.webm; do
  name="$(basename "$webm" .webm)"
  [[ "$name" == page@* ]] && continue   # Playwright's own copy (saved again under the scene name)
  dur="$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$webm")"
  ffmpeg -v error -y -i "$webm" -c:v libx264 -preset slow -crf 24 -pix_fmt yuv420p \
    -movflags +faststart -r 30 -an "$OUT/$name.mp4"
  ffmpeg -v error -y -ss "$(awk -v d="$dur" 'BEGIN{print d*0.45}')" -i "$webm" -frames:v 1 -q:v 3 "$OUT/$name.jpg"
  printf 'record-demos: %-18s %5.1fs  %s\n' "$name" "$dur" "$(du -h "$OUT/$name.mp4" | cut -f1)"
done
