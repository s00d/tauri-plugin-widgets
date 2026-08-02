---
title: Multi-widget
---

# Multi-widget

One app can drive many widgets by giving each a distinct `widgetId` inside the same (or different) `group`.

<div class="doc-illust">

![One host driving multiple widgets](/illustrations/multi-widget.jpg)

</div>

```typescript
await setWidgetConfig(weatherConfig, group, "weather");
await setWidgetConfig(tasksConfig, group, "tasks");
```

## Actions

`onWidgetAction` payloads include `widgetId` and `group`. Filter in the handler:

```typescript
await onWidgetAction((data) => {
  if (data.widgetId !== "tasks") return;
  // …
});
```

`startWidgetUpdater` is per `(group, widgetId)` — start one updater per logical widget.

## Android instances

Each pinned AppWidget instance can bind to a different logical id via meta `widgetId:{appWidgetId}`. After the user pins a second instance, call `setWidgetConfig` for that id so two tiles do not share one config by accident.

## Desktop windows

`createWidgetWindow` needs `group`, `widgetId`, and `size` so the **embedded** renderer (no local `widget.html` copy) loads the right config. See [Desktop webview](/guide/setup/desktop).
