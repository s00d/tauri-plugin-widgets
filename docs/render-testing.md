# Render testing (config → geometry / pixels)

Headless Level-1 gate: pure `JSON → layout tree`, no emulator / WidgetKit / AppWidgetManager.

## Who is the etalon?

| Level | Contract | Etalon |
|-------|----------|--------|
| 1 | Geometry JSON trees | **Desktop** (`*.desktop.json`) |
| 1 | Cross-check | Android vs Desktop (scaled centers, allowlist) |
| 2 | Pixel PNGs | Per-platform goldens (`ios` snapshots, optional desktop screenshots) |

**Baseline order (required):**

1. Run the harness and inspect the diff.
2. Decide **who is right** (do not freeze a known bug).
3. Update that platform’s baseline (`--update` / `-PupdateSnapshots`).
4. Fix the other platforms or add an allowlist entry with a reason.

## Commands

```bash
# Desktop geometry (Playwright)
pnpm test:desktop
pnpm test:desktop:update          # writes tests/expected/geometry/*.desktop.json

# Android geometry (Robolectric + Glance RemoteViews) — JDK 17
export JAVA_HOME="$(/usr/libexec/java_home -v 17)"
cd android && ./gradlew :testDebugUnitTest
cd android && ./gradlew :testDebugUnitTest -PupdateSnapshots=true

# Cross-platform geometry gate (local; not wired into GitHub Actions)
pnpm test:geometry                # Android↔Desktop, center ±8px after density scale

# iOS / macOS ImageRenderer (Level 2)
cd swift && swift test
# Record pixel snapshots:
SNAPSHOT_TESTING_RECORD=all swift test

# Vision triage (manual, not CI)
node --experimental-strip-types tools/triage.ts \
  --android a.png --ios i.png --desktop d.png \
  --config tests/fixtures/bugs/null-fields.json \
  --out /tmp/contact.png
```

## Layout

```
tests/fixtures/{core,bugs,presets}/   # widget config JSON
tests/expected/geometry/
  *.desktop.json
  *.android.json
  allowlist.json                      # known platform quirks
tests/expected/pixels/{android,ios,desktop}/   # Level 2 PNG goldens
```

PNG filenames: `<folder>__<name>.<size>.png` (e.g. `bugs__null-fields.small.png`).

Event scenarios (desktop): `tests/fixtures/events/*.json` → before/after PNG in
`tests/expected/pixels/desktop/events/`. Run `pnpm test:events` / `pnpm test:events:update`.

Update pixels together with geometry:

```bash
pnpm test:desktop:update
pnpm test:android:update
cd swift && UPDATE_SNAPSHOTS=1 swift test
```

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

## Fixture notes

- `bugs/null-fields.json` — explicit JSON `null`s; renderers must not paint the string `"null"`.
- Desktop dump skips `#close-btn` / `#drag-handle` chrome.
- Android harness uses `WidgetRootDirect` (no Glance Preferences state).
