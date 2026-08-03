---
title: Core vs extended
---

# Core vs extended

Not every element is held to the same cross-platform bar. **Pixel-identical Core+Extended everywhere is not possible** — Adaptive Cards, Glance, and WidgetKit each miss overlays, live timers, or network fetch. Aim for a **portable Core** and treat Extended as best-effort.

<div class="doc-illust">

![Core elements versus extended best-effort elements](/illustrations/tiers.jpg)

</div>

## Core

**Core** means the element is present on every matrix platform and keeps its IR semantics (layout role, text, tap target, bitmap image, …). It does **not** mean identical pixels. Degraded cells (for example Windows `zstack` / `shape`, or `image.systemName` outside Apple) are documented in the matrix below — check them before assuming Board/Glance look like WidgetKit.

The authoritative list is `CORE_ELEMENTS` in `src/snapshot.rs` (also mirrored under [Capability matrix](#capability-matrix)). Prefer that set when you need broad coverage.

### Portable Core checklist

- Prefer solid backgrounds (or accept gradient bake / first-stop degradation).
- Prefer `image.data` / `image.url` over `image.systemName` for Board/Android parity — SF Symbols exist only on Apple; elsewhere the feature row is degraded (glyph/emoji).
- Remote `image.url` is prefetched on the host where wired — do not rely on live network inside the extension process.
- Prefer Desktop webview when you need true overlays, live timers, or full SVG canvas.

## Extended

**Extended** (`EXTENDED_ELEMENTS`) is best-effort and platform-dependent. Expect `degraded` or `unsupported` cells (live `timer` on Windows Board, chart rasterization on Adaptive Cards). The live list is in the matrix section.

## How to read the matrix

| Level | Meaning |
| --- | --- |
| full | Supported with the intended semantics |
| degraded | Renders with known limitations (see footnotes under the matrix) |
| unsupported | Do not rely on this feature on that platform |

**Desktop column:** one IR path shared by the desktop webview hosts — WKWebView (macOS window), WebView2 (Windows window), and webkit2gtk (Linux). Semantics match; differences are system fonts and CSS engine version. Linux also has its own golden stand (`tests/golden/linux/`) for pinning / layer-shell, but it is not a sixth matrix column. Windows **Widgets Board** (Adaptive Cards) is the separate **Windows** column.

Showcase presets exercise both tiers; docs build fails if a schema element has **zero** fixture coverage.

## Runtime behavior

### Capability warnings on `setWidgetConfig`

Each `setWidgetConfig` runs `validate_config` for the current host platform and emits `log::warn!` via `log_capabilities` when a used element or feature is `degraded` / `unsupported` (for example `image.systemName` outside Apple). Warnings are advisory — the write still proceeds. There is no public guest API to dry-run validation yet; treat host logs (and the matrix) as the source of truth before shipping a config.

### Skipped elements in render receipts

When a renderer cannot express a node (common on Adaptive Cards without the `rasterize` feature, or after a flatten), the element is recorded on the render receipt as `skipped: [{ type, reason }]`, not as a silent hole you must guess about.

Inspect recent receipts with [`getWidgetDiagnostics(group)`](/api/js):

```ts
import { getWidgetDiagnostics } from "tauri-plugin-widgets-api";

const receipts = await getWidgetDiagnostics("com.example.widgets");
for (const r of receipts) {
  if (r.skipped?.length) {
    console.warn(r.widgetId, r.skipped);
    // e.g. [{ type: "canvas", reason: "adaptive-cards: enable crate feature `rasterize`" }]
  }
}
```

Desktop / debug builds can also use `getWidgetTrace(group)` for the host journal plus receipt history.

## Capability matrix

Table is authored in `src/capabilities.rs`; this page embeds it automatically. Refresh with `cargo test --lib capabilities::write_docs::capability_matrix_doc_matches` (delete `docs/guide/_generated/capability-matrix.md` first if drifted) then `pnpm docs:generate`.

<!-- generated:capability-matrix — do not edit; run `pnpm docs:generate` -->

### Element lists

**Core** (`CORE_ELEMENTS` in `src/snapshot.rs`): [`vstack`](/elements/layout#el-vstack), [`hstack`](/elements/layout#el-hstack), [`zstack`](/elements/layout#el-zstack), [`container`](/elements/layout#el-container), [`grid`](/elements/layout#el-grid), [`text`](/elements/text#el-text), [`image`](/elements/media#el-image), [`spacer`](/elements/spacing#el-spacer), [`divider`](/elements/spacing#el-divider), [`progress`](/elements/data#el-progress), [`button`](/elements/interactive#el-button), [`link`](/elements/interactive#el-link), [`shape`](/elements/media#el-shape)

**Extended** (`EXTENDED_ELEMENTS`): [`gauge`](/elements/data#el-gauge), [`toggle`](/elements/interactive#el-toggle), [`date`](/elements/text#el-date), [`chart`](/elements/data#el-chart), [`list`](/elements/data#el-list), [`timer`](/elements/text#el-timer), [`canvas`](/elements/media#el-canvas), [`label`](/elements/text#el-label)

### Core

| Element | iOS | macOS | Android | Desktop | Windows |
|---------|-----|-------|---------|---------|----------|
| [`vstack`](/elements/layout#el-vstack) | full | full | full | full | full<sup>1</sup> |
| [`hstack`](/elements/layout#el-hstack) | full | full | full | full | full<sup>1</sup> |
| [`zstack`](/elements/layout#el-zstack) | full | full | full | full | degraded<sup>2</sup> |
| [`container`](/elements/layout#el-container) | full | full | full | full | full<sup>1</sup> |
| [`grid`](/elements/layout#el-grid) | full | full | full | full | full<sup>1</sup> |
| [`text`](/elements/text#el-text) | full | full | full | full | full<sup>1</sup> |
| [`image`](/elements/media#el-image) | full | full | full<sup>3</sup> | full | full<sup>4</sup> |
| [`spacer`](/elements/spacing#el-spacer) | full | full | full | full | full<sup>1</sup> |
| [`divider`](/elements/spacing#el-divider) | full | full | full | full | full<sup>1</sup> |
| [`progress`](/elements/data#el-progress) | full | full | full | full | full<sup>1</sup> |
| [`button`](/elements/interactive#el-button) | full | full | full | full | full<sup>1</sup> |
| [`link`](/elements/interactive#el-link) | full | full | full | full | full<sup>1</sup> |
| [`shape`](/elements/media#el-shape) | full | full | full | full | degraded<sup>5</sup> |

### Extended

| Element | iOS | macOS | Android | Desktop | Windows |
|---------|-----|-------|---------|---------|----------|
| [`gauge`](/elements/data#el-gauge) | full | full | full | full | degraded<sup>5</sup> |
| [`toggle`](/elements/interactive#el-toggle) | full | full | full | full | full<sup>1</sup> |
| [`date`](/elements/text#el-date) | full | full | full | full | full<sup>1</sup> |
| [`chart`](/elements/data#el-chart) | full | full | full<sup>6</sup> | full<sup>7</sup> | degraded<sup>5</sup> |
| [`list`](/elements/data#el-list) | full | full | full<sup>8</sup> | full | degraded<sup>9</sup> |
| [`timer`](/elements/text#el-timer) | full | full | full<sup>10</sup> | full<sup>11</sup> | degraded<sup>12</sup> |
| [`canvas`](/elements/media#el-canvas) | full | full | degraded<sup>13</sup> | full<sup>7</sup> | degraded<sup>5</sup> |
| [`label`](/elements/text#el-label) | full | full | full | full | full<sup>1</sup> |

### Feature notes

| Feature | iOS | macOS | Android | Desktop | Windows |
|---------|-----|-------|---------|---------|----------|
| [`image.url`](/elements/media#el-image) | full<sup>14</sup> | full<sup>14</sup> | full<sup>15</sup> | full | full<sup>16</sup> |
| [`image.systemName`](/elements/media#el-image) | full<sup>17</sup> | full<sup>17</sup> | degraded<sup>18</sup> | degraded<sup>18</sup> | degraded<sup>19</sup> |
| [`background.gradient`](/elements/style) | full<sup>20</sup> | full<sup>20</sup> | full<sup>21</sup> | full<sup>22</sup> | degraded<sup>23</sup> |
| [`canvas.path`](/elements/media#el-canvas) | full<sup>24</sup> | full<sup>24</sup> | full<sup>25</sup> | full<sup>26</sup> | degraded<sup>27</sup> |
| [`timer.live`](/elements/text#el-timer) | full<sup>28</sup> | full<sup>28</sup> | full<sup>10</sup> | full<sup>29</sup> | degraded<sup>30</sup> |

### Choosing a surface set

Pick the platforms you ship, then stay in the **full** set for that profile. Degraded cells still render, but check the [Notes](#notes) and element pages.

#### Apple only

Platforms: **iOS + macOS**.

- **Full core:** [`vstack`](/elements/layout#el-vstack), [`hstack`](/elements/layout#el-hstack), [`zstack`](/elements/layout#el-zstack), [`container`](/elements/layout#el-container), [`grid`](/elements/layout#el-grid), [`text`](/elements/text#el-text), [`image`](/elements/media#el-image), [`spacer`](/elements/spacing#el-spacer), [`divider`](/elements/spacing#el-divider), [`progress`](/elements/data#el-progress), [`button`](/elements/interactive#el-button), [`link`](/elements/interactive#el-link), [`shape`](/elements/media#el-shape)
- **Full extended:** [`gauge`](/elements/data#el-gauge), [`toggle`](/elements/interactive#el-toggle), [`date`](/elements/text#el-date), [`chart`](/elements/data#el-chart), [`list`](/elements/data#el-list), [`timer`](/elements/text#el-timer), [`canvas`](/elements/media#el-canvas), [`label`](/elements/text#el-label)

#### Apple + Android

Platforms: **iOS + macOS + Android**.

- **Full core:** [`vstack`](/elements/layout#el-vstack), [`hstack`](/elements/layout#el-hstack), [`zstack`](/elements/layout#el-zstack), [`container`](/elements/layout#el-container), [`grid`](/elements/layout#el-grid), [`text`](/elements/text#el-text), [`image`](/elements/media#el-image), [`spacer`](/elements/spacing#el-spacer), [`divider`](/elements/spacing#el-divider), [`progress`](/elements/data#el-progress), [`button`](/elements/interactive#el-button), [`link`](/elements/interactive#el-link), [`shape`](/elements/media#el-shape)
- **Full extended:** [`gauge`](/elements/data#el-gauge), [`toggle`](/elements/interactive#el-toggle), [`date`](/elements/text#el-date), [`chart`](/elements/data#el-chart), [`list`](/elements/data#el-list), [`timer`](/elements/text#el-timer), [`label`](/elements/text#el-label)
- **Extended with degraded/unsupported cells:** [`canvas`](/elements/media#el-canvas)

#### All five matrix columns

Platforms: **iOS + macOS + Android + Desktop + Windows**.

- **Full core:** [`vstack`](/elements/layout#el-vstack), [`hstack`](/elements/layout#el-hstack), [`container`](/elements/layout#el-container), [`grid`](/elements/layout#el-grid), [`text`](/elements/text#el-text), [`image`](/elements/media#el-image), [`spacer`](/elements/spacing#el-spacer), [`divider`](/elements/spacing#el-divider), [`progress`](/elements/data#el-progress), [`button`](/elements/interactive#el-button), [`link`](/elements/interactive#el-link)
- **Core with degraded/unsupported cells:** [`zstack`](/elements/layout#el-zstack), [`shape`](/elements/media#el-shape)
- **Full extended:** [`toggle`](/elements/interactive#el-toggle), [`date`](/elements/text#el-date), [`label`](/elements/text#el-label)
- **Extended with degraded/unsupported cells:** [`gauge`](/elements/data#el-gauge), [`chart`](/elements/data#el-chart), [`list`](/elements/data#el-list), [`timer`](/elements/text#el-timer), [`canvas`](/elements/media#el-canvas)


### Notes {#notes}

1. Adaptive Cards 1.5
2. rasterized PNG overlay when possible; else flattened Container
3. url via localPath preprocess; see image.systemName
4. url/data URI; see image.systemName
5. rasterized PNG
6. bitmap bar/line/area/pie
7. SVG
8. Column chunking; soft cap ~50 items
9. Adaptive Cards Table
10. Chronometer via AndroidRemoteViews
11. setInterval
12. provider minute push + static TextBlock
13. bitmap canvas (full SVG path via PathParser)
14. host prefetch to data URI on setWidgetConfig
15. preprocess to localPath on setWidgetConfig
16. Adaptive Cards Image.url
17. SF Symbols
18. SF→Material / emoji map (not SF Symbols)
19. emoji TextBlock by default; glyph PNG Image with feature rasterize
20. linear/radial/angular SwiftUI
21. baked bitmap at LocalSize / frame
22. linear/radial/angular CSS/SVG
23. rasterized PNG backgroundImage when rasterize enabled
24. SVG path grammar
25. PathParser full SVG path
26. SVG path
27. rasterized via SVG
28. Text(..., .timer)
29. JS interval
30. provider pushes UpdateWidget ~1/min

<!-- /generated:capability-matrix -->
