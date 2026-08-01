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

Corpus: **36/36** `tests/golden/windows/` via `bash tools/gen-windows-goldens.sh` (AC transpile + SVG composite + resvg). Source of truth is Mac-side; UTM PreviewHost is spot-check only.

| Issue | Fix |
|-------|-----|
| Blank goldens (no text) | `usvg` `fontdb.load_system_fonts()` in rasterize + gen |
| Only 3 goldens | Full case walk in `gen-windows-goldens` |
| shape / zstack empty | shape → rasterized PNG; zstack → flattened Container |
| Progress bar invisible in composite | ColumnSet empty+emphasis → painted bar rects |
| Progress label not tinted | AC TextBlock `color=Good` from progress `tint` |
| Toggle looked like blue ActionSet buttons | Toggle → `✓` / `○` TextBlock |
| Flat gray composite bg | Always `#0f172a` navy |
| Canvas Stretch image tiny / top-left | Fill frame + `xMidYMid meet` |
| Dim/invisible text (`Dark`, slate→Accent) | `label`→Default; low-sat hex→Light; brighter composite fills |
| Button labels blue-on-blue / missing glyphs | White fg; ASCII media (`\|\|`, `>\|`); larger font |
| Progress always green | Hex tint → `id=fill:#…` + hue style |
| Gauge “Steps” / % unreadable | Rasterize: white current + white label |
| SF Symbol images empty | `gear`/`person.fill` → emoji TextBlock |
| Link chip no background | `apply_container_style` + composite pill |
| Empty/zstack off-center | Center badges; expand spacers; inherit Center |
| Image+label false overlay (“Card”) | Overlay only short glyphs (≤2) / `badge:overlay` |
| Audit sheets missing Windows | `tools/audit-sheets.ts` 5th panel |

### Intentionally Degraded (Windows)

- No true zstack overlay (children stacked)
- Hex text colors often omitted in AC (semantic tokens only); composite approximates
- Chart/canvas/gauge are rasterized PNGs, not live Adaptive Cards primitives
- Composite is Adaptive Card fidelity stand-in, not WinUI Widgets Board pixels

## Still open / needs emulator

- Android AppWidgetHost re-record (no device attached this run) — Robolectric PNGs in `tests/golden/android` remain stale
- Engine packing drift (lists, spacers) — Level-1 allowlist, not pixel bugs
- SF Symbol fidelity vs emoji glyphs — expected
- Windows Widgets Board / Win+W pixel goldens (PreviewHost UTM) — optional follow-up

## Commands

```bash
pnpm audit:sheets && open out/audit/index.html
bash tools/gen-windows-goldens.sh   # or: just record-windows-all
just test-macos-visual
pnpm test:visual:desktop
```
