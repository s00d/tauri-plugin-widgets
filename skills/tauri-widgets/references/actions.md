# Actions

- `button` / `toggle` / list rows with `action` emit `widget-action`; `link` / button `url`
  open URLs.
- Subscribe with `onWidgetAction(cb)` — it also delivers a tap that launched the app (the
  host holds queued taps until the first `poll_pending_actions`, which `onWidgetAction` makes
  right after subscribing — the gate opens even if that poll fails; fallback 10 s after start:
  mobile schedules a drain for that moment after a cold start, the desktop macOS poller checks
  every 500 ms).
- Payload: `{ action, payload?, ts, widgetId, group }`. Activity buttons: `widgetId = "activity.<id>"`.
- iOS interactive buttons need iOS 17 (AppIntents); Widgets Board uses `Action.Execute`.
