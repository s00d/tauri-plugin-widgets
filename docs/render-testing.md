# Render testing (config → geometry / pixels)

Headless **Level-1** gate: pure `JSON → layout tree`. Pixel goldens are **Level-2** — see [visual-stand.md](./visual-stand.md).

## Who is the etalon?

| Level | Contract | Etalon |
|-------|----------|--------|
| 1 | Geometry JSON trees | **Desktop** (`*.desktop.json`) |
| 1 | Cross-check | Android vs Desktop (scaled centers, allowlist) |
| 2 | Pixel PNGs | `tests/golden/{android,ios,desktop}/` via declarative `tests/cases/` |

**Baseline order (required):**

1. Run the harness and inspect the diff.
2. Decide **who is right** (do not freeze a known bug).
3. Update that platform’s baseline (`UPDATE_SNAPSHOTS` for geometry; `GOLDEN_RECORD=1 CASE=…` for pixels).
4. Fix the other platforms or add an allowlist entry with a reason.

## Commands

```bash
# Desktop geometry (Playwright) + visual cases
pnpm test:desktop
pnpm test:desktop:update          # writes tests/expected/geometry/*.desktop.json
pnpm test:visual:desktop          # asserts tests/golden/desktop/<case>.png
CASE=weather.small GOLDEN_RECORD=1 pnpm test:visual:desktop

# Android geometry (Robolectric) — JDK 17 — Level-1 only
export JAVA_HOME="$(/usr/libexec/java_home -v 17)"
pnpm test:android
pnpm test:android:update

# Android visual (emulator + AppWidgetHost) — see visual-stand.md
just android-up
just test-android-visual

# Cross-platform geometry gate (local; not wired into GitHub Actions)
pnpm test:geometry

# iOS / macOS visual (cases → tests/golden/ios)
cd swift && swift test --filter RenderTests
CASE=weather.small GOLDEN_RECORD=1 swift test --filter RenderTests

# Vision triage (manual, not CI)
node --experimental-strip-types tools/triage.ts \
  --android tests/golden/android/null-fields.small.png \
  --ios tests/golden/ios/null-fields.small.png \
  --desktop tests/golden/desktop/null-fields.small.png \
  --config tests/fixtures/bugs/null-fields.json \
  --out /tmp/contact.png
```

## Layout

```
tests/cases/*.json                      # declarative visual cases
tests/fixtures/{core,bugs,presets}/     # widget config JSON
tests/expected/geometry/
  *.desktop.json
  *.android.json
  allowlist.json
tests/golden/{android,ios,desktop}/     # Level-2 PNG goldens (<case>.png)
out/                                    # actual + diff (gitignored)
```

Event scenarios (desktop): `tests/fixtures/events/*.json` → before/after PNG in
`tests/expected/pixels/desktop/events/`. Run `pnpm test:events` / `pnpm test:events:update`.

## Allowlist

`tests/expected/geometry/allowlist.json` entries:

```json
{ "fixture": "bugs/list-20-items", "size": "large", "path": "*", "reason": "…" }
```

- `path: "*"` or `waive: true` — skip the whole fixture/size.
- Specific paths (`center:Label`, `overlap`, `literal-null:…`) — waive one check.

Never use the allowlist to hide a literal `"null"` text regression.

## Tolerances

- **Same platform** (Playwright / Robolectric vs own baseline): root rect **±2px**.
- **Cross platform** (`pnpm test:geometry`): text-leaf centers after density normalize, default **±8px** (`GEOMETRY_TOL`), overlap ≥ 0.5 (`GEOMETRY_MIN_OVERLAP`). Font metrics and Glance vs HTML layout make ±2 unrealistic across engines.
- **Pixels** (Level-2): ~2% mismatched pixels (channel delta > 8), or iOS similarity ≥ 0.98.

## Fixture notes

- `bugs/null-fields.json` — explicit JSON `null`s; renderers must not paint the string `"null"`.
- Desktop dump skips `#close-btn` / `#drag-handle` chrome.
- Android geometry harness uses `WidgetRootDirect` (no Glance Preferences state).
- Android pixel path uses AppWidgetHost + SharedPreferences (production path).
