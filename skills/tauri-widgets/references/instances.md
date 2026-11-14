# Instances (same widget placed twice)

Instance id: Android `appWidgetId`; Apple Edit Widget choice; desktop window `instance`
(default label); Widgets Board id.

- List: `await getActiveWidgets()` → `[{ widgetId, instance?, size?, lastSeenMs? }]`.
  iOS maps WidgetKit kinds to widget ids only when providers get `kind:` (template does).
- Own config: `setWidgetConfig(config, undefined, "weather", false, instance)`.
- Own data: `setWidgetData("weather", data, { instance })`.
- Without per-placement configs: use `{{instance}}` in `source.url` and text.
- Apple picker: `setWidgetInstances("weather", [{ id: "nyc", title: "New York" }])` and an
  `AppIntentConfiguration(kind:intent: TauriInstanceIntent.self, provider: TauriInstanceProvider(...))`
  widget in the extension (template: `MyInstanceWidget`). Choices of all widgets in the
  App Group are listed together — keep ids distinct.
