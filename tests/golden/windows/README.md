# Windows PNG goldens

Record via PreviewHost on a Windows VM (`AdaptiveCards.Rendering.Wpf`, `Smoke=false`), not the removed Mac SVG compositor:

```bash
just record-windows weather.small
# or: scripts/win/shot.ps1 -Mode visual -Record
```

Smoke walker (`Smoke=true`) is only for bootstrap / fallback — visual goldens must use the full WPF renderer.

JSON transpile snapshots live in `tests/snapshots/adaptive/`.
