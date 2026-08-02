---
title: Images
---

# Images

`image` supports:

<div class="doc-illust">

![Image sources: symbol, base64, and URL into a widget](/illustrations/images.jpg)

</div>

| Source | Notes |
| --- | --- |
| `systemName` | SF Symbols on Apple; drawable / glyph map on Android; placeholder on desktop |
| `data` | Base64 (with or without `data:image/...;base64,` prefix) |
| `url` | Remote URL — full on Android/Desktop/Windows; unsupported on iOS/macOS host prefetch today |

See [Core vs extended](/guide/tiers) for per-platform support.
