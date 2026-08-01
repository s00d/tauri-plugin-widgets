# Capability matrix (element × platform)

Generated from `tauri_plugin_widgets::capabilities`. Do not edit by hand.

## Core (strict snapshot contract)

| Element | iOS | macOS | Android | Desktop |
|---------|-----|-------|---------|----------|
| `vstack` | full | full | full | full |
| `hstack` | full | full | full | full |
| `zstack` | full | full | full | full |
| `container` | full | full | full | full |
| `grid` | full | full | full | full |
| `text` | full | full | full | full |
| `image` | full | full | degraded (systemName via glyph map; url via localPath preprocess) | full |
| `spacer` | full | full | full | full |
| `divider` | full | full | full | full |
| `progress` | full | full | full | full |
| `button` | full | full | full | full |
| `link` | full | full | full | full |
| `shape` | full | full | full | full |

## Extended (best-effort, platform-dependent)

| Element | iOS | macOS | Android | Desktop |
|---------|-----|-------|---------|----------|
| `gauge` | full | full | full | full |
| `toggle` | full | full | full | full |
| `date` | full | full | full | full |
| `chart` | full | full | degraded (simplified bar/line rendering) | full (SVG) |
| `list` | full | full | full (Glance LazyColumn; depth/children limited) | full |
| `timer` | full | full | degraded (static snapshot, not live Chronometer in all hosts) | full (setInterval) |
| `canvas` | full | full | degraded (bitmap canvas; path support limited) | full (SVG) |
| `label` | full | full | full | full |

## Feature notes

| Feature | iOS | macOS | Android | Desktop |
|---------|-----|-------|---------|----------|
| `image.url` | unsupported (prefetch into shared container not wired) | unsupported (prefetch into shared container not wired) | full (preprocess to localPath on setWidgetConfig) | full |
| `image.systemName` | full (SF Symbols) | full (SF Symbols) | degraded (glyph / drawable name map) | degraded (placeholder glyph) |
| `background.gradient` | degraded (linear primary; radial/angular limited) | degraded (linear primary; radial/angular limited) | degraded (first color stop only (Glance)) | full (linear/radial/angular CSS/SVG) |
| `canvas.path` | degraded (M/L/H/V/Z subset) | degraded (M/L/H/V/Z subset) | degraded (limited path commands) | full (SVG path) |
| `timer.live` | full (Text(..., .timer)) | full (Text(..., .timer)) | unsupported (no live timer in Glance snapshot) | full (JS interval) |
