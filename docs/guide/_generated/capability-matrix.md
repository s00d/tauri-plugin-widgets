# Capability matrix (element × platform)

Generated from `tauri_plugin_widgets::capabilities`. Do not edit by hand.

## Core (strict snapshot contract)

| Element | iOS | macOS | Android | Desktop | Windows |
|---------|-----|-------|---------|---------|----------|
| `vstack` | full | full | full | full | full (Adaptive Cards 1.5) |
| `hstack` | full | full | full | full | full (Adaptive Cards 1.5) |
| `zstack` | full | full | full | full | degraded (flattened Container, no overlay) |
| `container` | full | full | full | full | full (Adaptive Cards 1.5) |
| `grid` | full | full | full | full | full (Adaptive Cards 1.5) |
| `text` | full | full | full | full | full (Adaptive Cards 1.5) |
| `image` | full | full | degraded (systemName via glyph map; url via localPath preprocess) | full | degraded (url/data URI; systemName unsupported) |
| `spacer` | full | full | full | full | full (Adaptive Cards 1.5) |
| `divider` | full | full | full | full | full (Adaptive Cards 1.5) |
| `progress` | full | full | full | full | full (Adaptive Cards 1.5) |
| `button` | full | full | full | full | full (Adaptive Cards 1.5) |
| `link` | full | full | full | full | full (Adaptive Cards 1.5) |
| `shape` | full | full | full | full | degraded (rasterized PNG) |

## Extended (best-effort, platform-dependent)

| Element | iOS | macOS | Android | Desktop | Windows |
|---------|-----|-------|---------|---------|----------|
| `gauge` | full | full | full | full | degraded (rasterized PNG) |
| `toggle` | full | full | full | full | full (Adaptive Cards 1.5) |
| `date` | full | full | full | full | full (Adaptive Cards 1.5) |
| `chart` | full | full | degraded (simplified bar/line rendering) | full (SVG) | degraded (rasterized PNG) |
| `list` | full | full | full (Glance LazyColumn; depth/children limited) | full | degraded (flattened TextBlocks) |
| `timer` | full | full | degraded (static snapshot, not live Chronometer in all hosts) | full (setInterval) | degraded (static TextBlock of targetDate) |
| `canvas` | full | full | degraded (bitmap canvas; path support limited) | full (SVG) | degraded (rasterized PNG) |
| `label` | full | full | full | full | full (Adaptive Cards 1.5) |

## Feature notes

| Feature | iOS | macOS | Android | Desktop | Windows |
|---------|-----|-------|---------|---------|----------|
| `image.url` | unsupported (prefetch into shared container not wired) | unsupported (prefetch into shared container not wired) | full (preprocess to localPath on setWidgetConfig) | full | full (Adaptive Cards Image.url) |
| `image.systemName` | full (SF Symbols) | full (SF Symbols) | degraded (glyph / drawable name map) | degraded (placeholder glyph) | unsupported (no SF Symbols on Adaptive Cards) |
| `background.gradient` | degraded (linear primary; radial/angular limited) | degraded (linear primary; radial/angular limited) | degraded (first color stop only (Glance)) | full (linear/radial/angular CSS/SVG) | unsupported (Container style=emphasis only) |
| `canvas.path` | degraded (M/L/H/V/Z subset) | degraded (M/L/H/V/Z subset) | degraded (limited path commands) | full (SVG path) | degraded (rasterized via SVG) |
| `timer.live` | full (Text(..., .timer)) | full (Text(..., .timer)) | unsupported (no live timer in Glance snapshot) | full (JS interval) | unsupported (static only) |
