# ADR 0003: Widgets fetch their own data

Status: Accepted

## Context

Fresh data needed the app to run (`startWidgetUpdater`) or a server that speaks the
widget JSON. WidgetKit and Android allow the widget itself to make short requests.

## Decision

`WidgetConfig.source = { url, method, headers, body, refreshMinutes }`. The response (any
JSON up to 256 KiB) becomes the top data layer for `{{data.…}}`:
`config.data` < `setWidgetData` < fetched. `url`, headers and body may use only `{{locale}}`
and `{{instance}}` (never `{{data.…}}`, on every platform, so a response cannot change its own
request), so each placement asks for its own data without its own config.

- Only `https://` (and `http://localhost` / `127.0.0.1` for development).
- `refreshMinutes` clamped to 15–1440, default 30; a fetch runs only when the cache is
  older than that.
- The last good response is cached apart from the widget config (Apple: a UserDefaults key;
  Android: its own prefs file). On desktop the renderer writes it with `set_items` under
  `fetched:{id}` in the shared store, so it does write the map, but only that key — a fetch
  never overwrites a newer config.
- A failed fetch keeps the cache.

## Consequences

- iOS/macOS fetch inside the timeline request (8 s timeout); Android on redraw, with an
  inexact alarm when the next fetch is due; desktop while the window is open (each time a window opens, then every `refreshMinutes`).
- The Widgets Board provider does not fetch; the app pushes `setWidgetData` there.
- Credentials sit in the config (headers). A keychain-backed secret store is the upgrade
  path if apps need tokens that rotate without rewriting the config.
