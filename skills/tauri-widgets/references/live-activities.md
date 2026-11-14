# Live activities

```ts
await startActivity({ id: "order.42", title: "Order #42", body: "Driver en route",
  icon: "bicycle", progress: 0.3, endsAt: Date.now() + 12 * 60_000,
  actions: [{ id: "call", label: "Call driver" }] });
await updateActivity({ id: "order.42", title: "Order #42", progress: 0.9 });
await endActivity("order.42", 5_000);
```

- iOS 16.2+: Lock Screen + Dynamic Island. Extension bundle has `TauriLiveActivity()`;
  app Info.plist `NSSupportsLiveActivities = YES` (`init-ios` adds both). `push: true` →
  `pushToken` + `activity-token` events for APNs.
- Android: ongoing notification (progress, countdown, ≤ 3 actions, Live Update on 16).
  App must hold `POST_NOTIFICATIONS` (request it at runtime on 13+).
- Desktop: always-on-top window, top-right. An id restarted during `dismissAfterMs` keeps its window.
- Custom look: `layout` (any element tree), `compactLeading`, `compactTrailing`, `minimal`.
  Android ignores `layout`.
- ≤ 4 KiB JSON, counted including the layout the plugin generates when `layout` is omitted. Taps: `onWidgetAction` with `widgetId: "activity.<id>"`.
- Push: `push: true` → `pushToken`; `onActivityTokens(cb)` for rotations and the iOS 17.2+
  push-to-start token (`getActivityPushToStartToken()`); APNs `attributes-type:
  "TauriActivityAttributes"`, `content-state: {json}`.
