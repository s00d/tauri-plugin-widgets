---
title: Style
---

# Style

All elements support:

```json
{
  "padding": 12,
  "background": "#1a1a2e",
  "cornerRadius": 8,
  "opacity": 0.9,
  "frame": { "width": 100, "height": 50, "maxWidth": "infinity" },
  "border": { "color": "#333333", "width": 1 },
  "shadow": { "color": "#000000", "radius": 4, "x": 0, "y": 2 },
  "clipShape": "circle",
  "flex": 1
}
```

#### `clipShape` — Content Masking

Clips the element's content to a shape. Useful for circular avatars, pill-shaped badges, etc.

| Value | Description |
|-------|-------------|
| `"circle"` | Perfect circle (50% border-radius) |
| `"capsule"` | Pill shape (fully rounded ends) |
| `"rectangle"` | Rectangle with `cornerRadius` applied |

> **Platform mapping:** SwiftUI `.clipShape()`, CSS `border-radius` + `overflow: hidden`, Android Glance shape clipping.

#### `flex` — Proportional Layout

The `flex` property allows elements to occupy available space proportionally within stacks, analogous to CSS `flex`, Android `layout_weight`, or SwiftUI `layoutPriority`.

```json
{
  "type": "hstack",
  "children": [
    { "type": "text", "content": "Left", "flex": 1 },
    { "type": "text", "content": "Center (wider)", "flex": 2 },
    { "type": "text", "content": "Right", "flex": 1 }
  ]
}
```

