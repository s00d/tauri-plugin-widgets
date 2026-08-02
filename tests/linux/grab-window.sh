#!/usr/bin/env bash
# Grab the live widget-probe window (not root — DESKTOP windows are often invisible on root).
# Usage: grab-window.sh /work/out/linux/tmp.png
set -euo pipefail

OUT="${1:?usage: grab-window.sh <out.png>}"
export DISPLAY="${DISPLAY:-:99}"

pick_wid() {
  local best="" best_area=0 w geom width height
  while read -r w; do
    [[ -n "$w" ]] || continue
    geom=$(xdotool getwindowgeometry --shell "$w" 2>/dev/null || true)
    [[ -n "$geom" ]] || continue
    eval "$geom"
    width="${WIDTH:-0}"
    height="${HEIGHT:-0}"
    if [[ "$width" -gt 40 && "$height" -gt 40 ]]; then
      area=$((width * height))
      if [[ "$area" -gt "$best_area" ]]; then
        best_area=$area
        best=$w
      fi
    fi
  done < <(xdotool search --name widget-probe 2>/dev/null || true)
  echo "$best"
}

unique_colors() {
  identify -format '%k' "$1" 2>/dev/null || echo 0
}

# True if frame still looks like .w-empty Loading gradient (indigo→violet).
is_loading_purple() {
  local r g b
  # Mean channel intensities 0–100.
  IFS=',' read -r r g b < <(identify -format '%[fx:int(100*mean.r)],%[fx:int(100*mean.g)],%[fx:int(100*mean.b)]' "$1" 2>/dev/null || echo '0,0,0')
  r=${r:-0}; g=${g:-0}; b=${b:-0}
  # Loading purple sits mid-high on R/B with G lower; real dark widgets have low means.
  if [[ "$b" -ge 35 && "$r" -ge 28 && "$g" -le 55 && "$b" -ge "$g" && "$r" -ge $((g - 5)) ]]; then
    return 0
  fi
  return 1
}

for attempt in $(seq 1 50); do
  WID=$(pick_wid)
  if [[ -z "$WID" ]]; then
    sleep 0.2
    continue
  fi
  xdotool windowmap "$WID" 2>/dev/null || true
  xdotool windowactivate "$WID" 2>/dev/null || true
  sleep 0.15

  if xwd -id "$WID" -silent 2>/dev/null | convert xwd:- "png:${OUT}.tmp" 2>/dev/null; then
    mv -f "${OUT}.tmp" "$OUT"
  else
    import -window "$WID" "$OUT" 2>/dev/null || true
  fi

  if [[ -f "$OUT" ]]; then
    colors=$(unique_colors "$OUT")
    bytes=$(wc -c < "$OUT" | tr -d ' ')
    if is_loading_purple "$OUT"; then
      sleep 0.25
      continue
    fi
    if [[ "${colors:-0}" -ge 4 && "${bytes:-0}" -gt 200 ]]; then
      echo "grab-window: wid=$WID colors=$colors bytes=$bytes → $OUT"
      exit 0
    fi
  fi
  sleep 0.25
done

echo "grab-window: FAILED (blank, loading purple, or missing widget window)" >&2
exit 1
