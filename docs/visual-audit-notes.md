# Visual audit notes (2026-08-01)

Generated via `pnpm audit:sheets` → `out/audit/index.html` (panels: Desktop | iOS | macOS | Android | **Windows**).

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

## Windows Adaptive Cards

Corpus: Adaptive Card **JSON** in `tests/snapshots/adaptive/` (`bash tools/gen-adaptive-snapshots.sh`). Windows **PNG** goldens: PreviewHost on UTM (`just record-windows <case>`), not a Mac SVG compositor.

| Issue | Fix |
|-------|-----|
| shape / zstack empty | shape → rasterized PNG; zstack → flattened Container |
| Toggle looked like blue ActionSet buttons | Toggle → `✓` / `○` TextBlock |
| Progress always green | Hex tint → `id=fill:#…` + hue style |
| SF Symbol images empty | `gear`/`person.fill` → emoji TextBlock |
| Audit sheets missing Windows | `tools/audit-sheets.ts` 5th panel |

### Intentionally Degraded (Windows)

- No true zstack overlay (children stacked)
- Hex text colors often omitted in AC (semantic tokens only)
- Chart/canvas/gauge are rasterized PNGs when `rasterize` is enabled
- Pixel goldens must come from WinUI3 PreviewHost, not a Mac-side SVG stand-in

## Still open / needs emulator

- Android AppWidgetHost re-record (no device attached this run) — Robolectric PNGs in `tests/golden/android` remain stale
- Engine packing drift (lists, spacers) — Level-1 allowlist, not pixel bugs
- SF Symbol fidelity vs emoji glyphs — expected
- Windows Widgets Board / Win+W pixel goldens (PreviewHost UTM) — optional follow-up

## Commands

```bash
pnpm audit:sheets && open out/audit/index.html
bash tools/gen-adaptive-snapshots.sh   # AC JSON; FEATURES=rasterize for chart URIs
just record-windows weather.small      # real WinUI3 PNG via PreviewHost
just test-macos-visual
pnpm test:visual:desktop
```
