---
title: Colors
---

# Colors

## Adaptive colors

 (Dark Mode Support)

All color properties (`color`, `tint`, `fill`, `stroke`, `backgroundColor`, `iconColor`, `background`) support three formats:

**1. Hex string** (static):
```json
{ "color": "#FF5733" }
```

**2. Semantic color name** (auto-adapts to system theme):
```json
{ "color": "label" }
```

Available semantic colors: `label`, `secondaryLabel`, `tertiaryLabel`, `systemBackground`, `secondarySystemBackground`, `separator`, `accent`, `systemRed`, `systemGreen`, `systemBlue`, `systemOrange`, `systemYellow`, `systemPurple`, `systemPink`, `systemGray`

**3. Adaptive object** (explicit light/dark values):
```json
{ "color": { "light": "#000000", "dark": "#FFFFFF" } }
```

The `background` property additionally supports gradients:

```json
{
  "background": { "light": "#FFFFFF", "dark": "#1C1C1E" }
}
```

## Gradient Backgrounds

```json
{
  "background": {
    "gradientType": "linear",
    "colors": ["#667eea", "#764ba2"],
    "direction": "topToBottom"
  }
}
```

Types: `linear`, `radial`, `angular`.

---
