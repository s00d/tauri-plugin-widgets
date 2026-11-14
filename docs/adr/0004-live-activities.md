# ADR 0004: One live-activity request for every OS

Status: Accepted

## Context

iOS has Live Activities (ActivityKit), Android ongoing notifications (Live Updates from
Android 16), desktops nothing native and comparable. Apps want one call.

## Decision

`startActivity / updateActivity / endActivity / listActivities` take one
`ActivityContent`: `title`, `body`, `progress`, `endsAt`, `icon`, `tint`, `actions` (≤ 3),
`url`, `staleAt`, `push`, and optional element trees `layout`, `compactLeading`,
`compactTrailing`, `minimal`.

- Rust fills `layout` from the plain fields when it is missing (`effective_layout`), so
  every surface draws the same default, and validates: id charset, title, actions,
  progress range, and **4 KiB** of JSON on every platform (the ActivityKit limit), so a
  request that works on desktop does not fail on an iPhone.
- iOS: the request JSON is the `ContentState`; `TauriLiveActivity` (Swift package) draws
  `layout` with the widget renderer. The host plugin keeps an identical
  `TauriActivityAttributes` type, which ActivityKit pairs by name.
- Android: `NotificationCompat` with progress, countdown chronometer, actions and
  `android.requestPromotedOngoing`. Notification ids are allocated and stored per activity
  id (no hash collisions).
- Desktop (macOS, Windows, Linux): an always-on-top widget window per activity, top-right,
  drawing `layout` through the existing renderer.
- Taps arrive as `widget-action` with `widgetId = "activity.<id>"`.

## Consequences

- Android ignores `layout` (notifications have fixed templates); desktop and iOS draw it.
- APNs push updates are limited to handing out the token (`pushToken`, `activity-token`
  event); push-to-start and a server SDK are left out.
- The app requests `POST_NOTIFICATIONS` itself on Android 13+.
