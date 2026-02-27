# Widget Plugin Example

A demo app for [tauri-plugin-widgets](https://github.com/s00d/tauri-plugin-widgets).

Showcases all 20 element types across 18 widget presets with a live JSON editor, desktop widget windows, and native widget support on iOS/macOS/Android.

## Quick Start (Desktop)

```bash
pnpm install
pnpm tauri dev
```

## Platform-specific Setup

See [GUIDE.md](../../GUIDE.md) in the repository root for detailed instructions on running the example on **macOS**, **iOS**, **Android**, **Windows**, and **Linux**.

> **macOS note:** widgets appear in the system widget gallery only after a full build/install flow.  
> Build the app and move the resulting `.app` to **`/Applications`** (or install from the generated DMG), then add the widget.

## Project Structure

```
src/              — React frontend (widget builder UI)
  presets/        — 18 widget config presets
  App.tsx         — Main UI: preset selector, JSON editor, controls
src-tauri/        — Rust backend
  src/lib.rs      — Plugin registration
  tauri.conf.json — Tauri config
```

## Presets

| Preset | Description | New Features Used |
|--------|-------------|-------------------|
| Weather | 5-day forecast with conditions | Adaptive colors, `textStyle`, `flex`, flattened card rows |
| Calendar | Dynamic date display with events | Adaptive colors, styled `vstack` date badge, `textStyle`, `flex` |
| Tasks | Todo list with toggles and progress | Adaptive colors, `list`, progress labels, capsule styling |
| Fitness | Activity rings and health metrics | Adaptive colors, `textStyle`, `flex`, direct gauge layout |
| Music | Now-playing with track queue | Adaptive colors, circular clipped artwork, `textStyle`, `flex` |
| Crypto | Portfolio tracker with charts | Adaptive colors, styled `vstack` cards, `textStyle`, `flex` |
| Quote | Rotating inspirational quotes | Adaptive `{ light, dark }` colors throughout, `textStyle` |
| Social | Social stats dashboard | Adaptive colors, semantic `"label"` / `"secondaryLabel"`, `textStyle` |
| Battery | Device battery levels | Adaptive colors, `textStyle`, `flex`, direct gauge layout |
| System | System monitor (CPU/RAM/Disk) | Adaptive colors, `textStyle`, `flex`, direct gauge layout |
| Revenue | Revenue dashboard with pie chart | Adaptive colors, styled metric cards, `textStyle`, `flex` |
| Countdown | Countdown timer to a target date | `container`, `textStyle`, `flex` |
| Analog Clock | Canvas-drawn analog clock face | Canvas draw commands, gradient background |
| Traffic Light | Traffic signal simulation | Canvas, shapes, border, gradient background |
| Live Metrics | Real-time server monitoring | Dynamic data, gauges, charts, shapes |
| Calculator | Functional calculator with history | Button grid, state management |
| **User Profile** | Profile card with avatar | `clipShape: "circle"`, `flex`, adaptive colors, flattened metric cards |
| **Dark Mode Demo** | All new features showcase | Semantic colors, `textStyle`, `clipShape`, `flex`, `container` |

## New Features Demonstrated

### Adaptive Colors (Dark Mode)

All presets now use adaptive colors that auto-adjust to the system theme:

```typescript
// Semantic color name — adapts automatically
{ color: "label" }
{ color: "secondaryLabel" }
{ color: "systemBackground" }

// Explicit light/dark pair
{ color: { light: "#000000", dark: "#FFFFFF" } }

// Background also supports adaptive
{ background: { light: "#F8F9FA", dark: "#1C1C1E" } }
```

### Semantic Text Styles

Text elements can use `textStyle` for platform-aware sizing that respects Dynamic Type:

```typescript
{ type: "text", content: "Title", textStyle: "title3", color: "label" }
{ type: "text", content: "Body text", textStyle: "body", color: "label" }
{ type: "text", content: "Small note", textStyle: "caption", color: "secondaryLabel" }
```

Available styles: `largeTitle`, `title`, `title2`, `title3`, `headline`, `subheadline`, `body`, `callout`, `footnote`, `caption`, `caption2`

### Container Element

A single-child wrapper for cards, badges, and avatars:

```typescript
{
  type: "container",
  contentAlignment: "center",
  clipShape: "circle",
  background: { light: "#DBEAFE", dark: "#1E3A5F" },
  frame: { width: 48, height: 48 },
  children: [
    { type: "image", systemName: "person.fill", color: "#2563EB", size: 20 }
  ]
}
```

### Android-safe layout examples

The example presets were updated to avoid overly complex trees. Prefer these patterns:

```typescript
// Good: flat metric card
{
  type: "vstack",
  spacing: 2,
  padding: 8,
  background: { light: "#EDE9FE", dark: "#313244" },
  cornerRadius: 8,
  children: [
    { type: "text", content: "Revenue", textStyle: "caption2", color: "secondaryLabel" },
    { type: "text", content: "$12,450", fontSize: 18, fontWeight: "bold", color: "#a6e3a1" }
  ]
}

// Avoid: unnecessary wrapper depth
{
  type: "container",
  children: [{ type: "vstack", children: [{ type: "text", content: "..." }] }]
}
```

Additional notes:
- Keep branches relatively shallow (about 3-4 levels in most cases).
- In dense rows, prefer direct children + `flex`.
- Always set `progress.label` to avoid host-level `null` rendering text.

### Clip Shape

Content masking for any element:

```typescript
{ clipShape: "circle" }   // Circular avatars
{ clipShape: "capsule" }  // Pill-shaped badges
{ clipShape: "rectangle" } // Rounded rectangles (uses cornerRadius)
```

### Flex Layout

Proportional space distribution within stacks:

```typescript
{
  type: "hstack",
  children: [
    { type: "text", content: "1/4", flex: 1 },
    { type: "text", content: "1/2", flex: 2 },
    { type: "text", content: "1/4", flex: 1 }
  ]
}
```

## App Group

This example uses `group.com.s00d.tauri-plugin-widgets-example` as the App Group identifier. When setting up native widgets on iOS/macOS, ensure both the main app and widget extension use this same identifier.
