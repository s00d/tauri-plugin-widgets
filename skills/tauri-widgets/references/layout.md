# Layout JSON

`WidgetConfig = { version?, small?, medium?, large?, entries?, strings?, defaultLocale?, data?, source? }`.
A missing size family falls back to another (small → medium → large, per platform order).
JSON Schema for IDE checks: `"$schema": "https://s00d.github.io/tauri-plugin-widgets/schemas/widget-config.v1.json"`.

Elements (`type`): layout `vstack`, `hstack`, `zstack`, `grid`, `container`; content `text`,
`label`, `image`, `shape`, `canvas`, `date`, `timer`; data `progress`, `gauge`, `chart`,
`list`; interactive `button`, `toggle`, `link`; spacing `spacer`, `divider`.

Every element takes the shared style: `padding`, `background` (hex, `{light, dark}`,
gradient), `cornerRadius`, `opacity`, `frame`, `border`, `shadow`, `clipShape`, `flex`,
`accessibilityLabel`, `privacySensitive` (redacted on the Apple locked screen).

```json
{
  "small": {
    "type": "vstack", "padding": 12, "background": "#1a1a2e",
    "children": [
      { "type": "text", "content": "72°", "fontSize": 36, "fontWeight": "bold", "color": "#fff" },
      { "type": "progress", "value": 0.7, "tint": "#4CAF50", "label": "Humidity" }
    ]
  }
}
```

Images: `systemName` (SF Symbol; mapped to Material / emoji elsewhere), `url` (prefetched
by the host), `data` (base64). Field-level docs: https://s00d.github.io/tauri-plugin-widgets/elements
