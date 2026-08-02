---
title: Actions
---

# Actions

`button` and `link` elements emit `widget-action` events back to the app.

<div class="doc-illust">

![Widget action event flowing back to the host app](/illustrations/actions.jpg)

</div>

```typescript
import { onWidgetAction } from "tauri-plugin-widgets-api";

await onWidgetAction((data) => {
  console.log("Action:", data.action, data.payload, data.widgetId, data.group);
});
```

Action payload shape: `{ action, payload?, ts, widgetId, group }`.

You can also emit from the host with `widgetAction(action, payload?)`.
