# ADR 0002: Per-placement instances

Status: Accepted

## Context

Placing a widget twice showed the same content twice; Android already told each placement
apart (`appWidgetId`), Apple only through a configuration intent, desktop by window label,
the Widgets Board by its widget id. The app could not list placements at all.

## Decision

- An **instance** id names a placement. Store keys gain a scoped form
  `config:{widgetId}#{instance}`, `data:{widgetId}#{instance}`; `#` is outside the widget-id
  charset, so a scoped key can never collide with another widget. Renderers read the
  scoped key first, then the widget-wide one (`store::lookup_keys`).
- Sources of the id: Android `appWidgetId`; Apple `TauriInstanceIntent` (the user picks
  from `instances:{widgetId}` lists published by `setWidgetInstances`); desktop window
  `instance` (default: label); Widgets Board id.
- `getActiveWidgets()` merges a native list (exact on Android; WidgetKit kinds on iOS) with
  render receipts. On iOS only the extension knows the Edit Widget choice, so a kind's
  receipts with an instance replace its bare entries (`store::merge_native_active`).
- Host-side key sanitizers (iOS plugin, Android) now allow `#`.

## Consequences

- Apple's picker lists choices of all widgets in the App Group: an `EntityQuery` cannot see
  the asking widget kind. Distinct ids per widget avoid mixing; a typed intent per kind is
  the upgrade path.
- Android placements are numbered by the system; the app learns them from
  `getActiveWidgets()` before it can address one.
- Widget-wide config pushed into Glance state skips placements that have their own config.
