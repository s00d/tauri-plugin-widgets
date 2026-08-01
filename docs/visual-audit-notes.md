# Visual audit notes (2026-08-01)

Generated via `pnpm audit:sheets` → `out/audit/index.html`.

## Fixed in this pass

| Issue | Fix |
|-------|-----|
| `person.fill` → bullseye glyph (Desktop/Android) | Map to bust silhouette emoji |
| Progress Humidity label gray on iOS/macOS | Label uses `tint` when `color` absent |
| Linear gauge looks like slider (iOS/macOS) | Custom capsule bar like HTML |
| Countdown wraps on iOS/macOS | `lineLimit(1)` + `minimumScaleFactor` |
| Container content not centered (Swift) | ZStack `frame(maxWidth/Height: .infinity)` |
| Android button full-bleed in vstack | Hug width for button/image/shape |
| Android container ignores center | Box alignment + no full-width Column |
| Android `clipShape: circle` on sized images | Include `size` in clip radius |
| Android time `19:30` vs 12h | `h:mm a` Locale.US |
| Desktop circular gauge too thin | Larger ring (56px, stroke 4) |
| Swift golden flaky compare | RGBA normalize + Mac 2× bitmap |

## Still open / needs emulator

- Android AppWidgetHost re-record (no device attached this run) — Robolectric PNGs in `tests/golden/android` remain stale
- Engine packing drift (lists, spacers) — Level-1 allowlist, not pixel bugs
- SF Symbol fidelity vs emoji glyphs — expected

## Commands

```bash
pnpm audit:sheets && open out/audit/index.html
just test-macos-visual
pnpm test:visual:desktop
```
