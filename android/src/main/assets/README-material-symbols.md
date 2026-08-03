# Optional Material Symbols font

Drop Google’s **Material Symbols Outlined** TTF into the host app (or this library) as:

- `assets/material_symbols_outlined.ttf`, or
- `assets/fonts/MaterialSymbolsOutlined.ttf`

When present, Android Glance renders `image.systemName` via Material ligatures from the shared SF→Material map (`src/icons.rs`). Without the font, the same map falls back to emoji/unicode glyphs (never a random first letter).

Desktop / Windows use the emoji (and Windows Board rasterizes to PNG when the `rasterize` feature is on).
