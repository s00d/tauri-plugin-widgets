# Scheduled states, strings, data

Resolved by every renderer right before drawing; works while the app is closed.

- `entries: [{ at: <unix ms>, small?, medium?, large? }]` — last `at <= now` wins per family.
  Use for schedules, countdown end states, "event started" switches.
- `strings: { "en": {...}, "ru": {...} }` + `defaultLocale` — `{{t.key}}` in any string.
  Matching: exact tag (`pt_BR` = `pt-BR`; `zh-Hant-TW-u-…` matches `zh-TW`), same language, `defaultLocale`, `en`, first table.
  Missing key → default table → the key itself.
- `{{data.path}}` (`{{data.days.0.hi}}`): layers `config.data` < `setWidgetData(widgetId, data)`
  < fetched `source`. Missing → empty string. Integers as integers, whole floats as integers up to 1e21
  (`21.0` → `21`), other numbers shortest decimal without exponent. `{{data.obj}}` prints JSON
  whose key order/escaping differ per platform — don't rely on its text.
- `{{locale}}`, `{{instance}}` also available.
- `source: { url, method?, headers?, body?, refreshMinutes? }` — `https://` only (`http://localhost` / `127.0.0.1` exactly for dev), JSON ≤ 256 KiB,
  15–1440 min (default 30). `url` / headers / body may use only `{{locale}}` and `{{instance}}`
  (not `{{data…}}`, every platform). Cache kept on failure.

Prefer `setWidgetData` for value updates: it does not resend the layout.
Docs: https://s00d.github.io/tauri-plugin-widgets/features/timelines, /features/remote-data, /features/dynamic-content, /features/interactive
