---
title: Testing your widget
---

# Testing your widget

## Visual goldens (this repo)

Cases live in `tests/cases/*.json`, fixtures in `tests/fixtures/**`, screenshots in `tests/golden/{platform}/*.png`.

| Surface | Command |
| --- | --- |
| Desktop HTML | `pnpm test:desktop` / `pnpm test:desktop:update` |
| Android unit | `pnpm test:android` |
| macOS Swift | `pnpm test:macos:visual` |
| Geometry | `pnpm test:geometry` |

Contributor deep-dive: [Render testing](/contributing/render-testing), [Visual stand](/contributing/visual-stand).

## In your app

1. Keep fixture JSON next to the feature that builds `WidgetConfig`
2. Use the [showcase playground](/showcase) (same desktop renderer) to iterate on layout
3. On device, prefer `{ type: "timer" }` / date styles for live UI instead of burning WidgetKit reload budget
4. Call `getWidgetDiagnostics(group)` when a native surface looks empty — check receipts and skipped capabilities

## Docs coverage gate

`pnpm docs:audit` (also run from `docs:generate`) fails if:

- a schema element never appears in any fixture
- a case is missing a golden on desktop / ios / macos / android / linux

Windows goldens are optional until PreviewHost recording is restored.
