# Android: Samsung Now Bar, lock screen, cover screen, scrolling lists, November 2026

Research for the "later" Android row of [references-2026-11.md](./references-2026-11.md):
Samsung Now Bar extras, keyguard category, cover screen, scrollable list and the update
timestamp guard. Docs and code read, with links. What landed is listed at the end.

## Samsung Live Notifications and Now Bar

Samsung's own pipeline is older than AOSP Live Updates (One UI 7, Android 15). Samsung has
not published a developer page for it. The keys below come from apps that ship them and
from decompiled Samsung apps.

- Reference write-up: [akexorcist.dev, Live Notifications and Now Bar in One UI 7](https://akexorcist.dev/live-notifications-and-now-bar-in-samsung-one-ui-7-as-developer-en/).
  It covers the manifest flag `com.samsung.android.support.ongoing_activity = true`, the
  `android.ongoingActivityNoti.*` bundle (`style` = 1 required; `primaryInfo`,
  `secondaryInfo`, `chipIcon`, `chipBgColor`, `chipExpandedText`, `actionType` = 1 with
  `actionPrimarySet`, `progress` / `progressMax`, `progressSegments[]` with
  `segmentStart` / `segmentColor`, `progressSegments.progressColor`, `nowbarPrimaryInfo`,
  `nowbarSecondaryInfo`, `chronometerRemoteView*`). The Now Bar reuses `primaryInfo` /
  `secondaryInfo` unless the `nowbar*` keys are set. The Now Bar has no progress style.
- Access: on One UI 7, only media sessions and **allow-listed packages** get these surfaces.
  Other apps' extras are ignored.
- Code that ships it:
  - [nightscout/AndroidAPS `PersistentNotificationPlugin.applySamsungLiveUpdate`](https://github.com/nightscout/AndroidAPS/blob/master/app/src/main/kotlin/app/aaps/persistentNotification/PersistentNotificationPlugin.kt):
    Samsung-only (`Build.MANUFACTURER`), style 1 plus primary / secondary / chip / nowbar
    texts, alongside `setRequestPromotedOngoing` on API 36. Its
    [manifest](https://github.com/nightscout/AndroidAPS/blob/master/app/src/main/AndroidManifest.xml)
    declares the `ongoing_activity` meta-data.
  - [`OngoingExtrasBuilder`](https://github.com/YetAnotherBusDeveloper/yetanotherbusapp/blob/main/android/app/src/main/kotlin/com/nowbar/api/notification/OngoingExtrasBuilder.kt):
    the full key list with types (Icon parcelables, int colors, a Bundle array for segments).
  - [tigerduck `LiveActivityNotifier.samsungNowBarExtras`](https://github.com/tigerduck-app/tigerduck-app-android/blob/main/app/src/main/java/org/ntust/app/tigerduck/liveactivity/LiveActivityNotifier.kt):
    from decompiled **One UI 8.5** SystemUI. One UI 8+ feeds the Now Bar from AOSP promoted
    notifications, behind Samsung's allowlist (or the developer option "Live notifications
    for all apps"). **`style >= 1` moves the notification to Samsung's private card path and
    cancels the AOSP promotion.** tigerduck instead sets a private `automation` extra that
    skips the allowlist check. We do not use that key: it bypasses a vendor gate.
- One UI 8 news: [Android Authority, One UI 8 Live Updates](https://www.androidauthority.com/one-ui-8-live-updates-support-3573794),
  [SamMobile, One UI 9 Now Bar categories](https://www.sammobile.com/news/now-bar-one-ui-9-support-three-more-types-third-party-apps/).

**Decision.** On Samsung devices below API 36 (One UI 7 and older) the plugin adds the
`android.ongoingActivityNoti.*` extras. From API 36 (One UI 8+) it adds none: Samsung
reads the AOSP Live Update there (`setRequestPromotedOngoing`, `setShortCriticalText`,
`ProgressStyle`, already posted), and `style` would cancel it. The
`com.samsung.android.support.ongoing_activity` flag is merged from the plugin manifest.
Other devices never get the extras.

Mapping: `title` → `primaryInfo`, `body` → `secondaryInfo`, `shortCriticalText` (else
`title`) → `chipExpandedText`, `tint` → `chipBgColor` and `progressSegments.progressColor`,
`progress` → `progress` / `progressMax` 100. The status-bar icon (`tauri_activity_icon`) →
`chipIcon`. Actions → `actionType` 1 / `actionPrimarySet` 0 (the buttons themselves are the
notification actions). `endsAt` / `startedAt` stay on the notification chronometer
(`when` + `usesChronometer`); the custom chronometer RemoteViews are not used.

## Lock screen (keyguard)

- Notifications: [`setVisibility` / `setPublicVersion`](https://developer.android.com/develop/ui/views/notifications/build-notification#lockscreenNotification).
  `VISIBILITY_PRIVATE` shows the public version on a secure lock screen (by default only
  the app name and "Contents hidden"). `VISIBILITY_SECRET` shows nothing.
- Promoted Live Updates follow the same visibility rules ([Live Updates](https://developer.android.com/develop/ui/views/notifications/live-update)).
- Widgets: `android:widgetCategory="keyguard"` makes a widget eligible for lock-screen
  hosts. Android 16 QPR2 brings back lock-screen widgets on phones, in a hub mode
  ([9to5Google](https://9to5google.com/2025/08/20/android-qpr2-lock-screen-widgets/),
  [Android Authority](https://www.androidauthority.com/lock-screen-widgets-on-phones-android-16-qpr2-3589668/)).
  The Samsung cover screen also requires `keyguard`. The host reports
  `OPTION_APPWIDGET_HOST_CATEGORY = WIDGET_CATEGORY_KEYGUARD`.

**Decision.** `visibility: private` gets a redacted public version: the app name, the
chronometer, progress, tint and category, with no title, body, sub text or actions.
`secret` stays hidden, and on Samsung gets no Now Bar extras. With `private`, the Now Bar
texts are redacted too (`nowbarPrimaryInfo` = app name, empty `nowbarSecondaryInfo`). The
widget info XMLs now declare `home_screen|keyguard`.

## Galaxy Z Flip cover screen (Flex Window)

- [Samsung Flex Window guide](https://developer.samsung.com/galaxy-z/flex_window.html),
  [codelab "Develop a widget for Flex Window"](https://developer.samsung.com/codelab/galaxy-z/widget-flex-window.html),
  [Good Lock best practices](https://developer.samsung.com/sdp/blog/en/2024/01/09/best-practices-of-app-development-for-various-screen-sizespowered-by-good-lock).
  - The widget `<receiver>` adds meta-data `com.samsung.android.appwidget.provider` →
    `@xml/…` containing `<samsung-appwidget-provider display="sub_screen" />`.
  - Recommended `appwidget-provider`: `minWidth 352dp`, `minHeight 339dp`,
    `resizeMode horizontal|vertical`, `widgetCategory keyguard`. The pages swap width and
    height, and neither says whether these sizes are required for the widget to appear.
  - The user adds the widget in Settings > Cover screen > Widgets. Activities can open on
    the cover with `ActivityOptions.setLaunchDisplayId(1)`.

**Decision.** The plugin receiver carries the Samsung meta-data (`@xml/tauri_widget_samsung`)
and `keyguard`. The min sizes stay at the home-screen 110 dp. The renderer picks the
**small** layout for a keyguard host (cover screen, lock-screen hub) whatever cell size it
reports. Taps still open the app on the main display.

## Scrolling lists in Glance

- [Glance `LazyColumn`](https://developer.android.com/develop/ui/compose/glance/build-ui#scrollable-layouts):
  it becomes a `ListView` backed by `RemoteCollectionItems` (API 31+, compat service below),
  so RemoteViews collection limits apply: one Binder parcel for all items, and no nesting
  ([`LazyListTranslator`](https://github.com/androidx/androidx/blob/androidx-main/glance/glance-appwidget/src/main/java/androidx/glance/appwidget/translators/LazyListTranslator.kt):
  "Glance does not support nested list views"). Each item is translated with a single root
  child, and the view type count is `TopLevelLayoutsCount`.
- The IR has no `scrollable` flag on stacks, so only `list` scrolls.

**Decision.** `list` with `items` stays a chunked `Column` while its rows fit, so existing
goldens and baselines are unchanged. It becomes a `LazyColumn` once
`rows × fontSize × 1.3 + spacing + padding` is taller than the widget height. The 50-row
cap and the "+N more" row remain, to stay within the Binder budget. A `list` with custom
`children` keeps the Column (arbitrary rows could nest lists).

## Update timestamp guard

The iOS port (`ios/Sources/WidgetPlugin.swift` `remember` / `forget`) drops an update whose
`timestamp` is older than the last applied one for that id. Android now does the same. The
last timestamp is stored per id in the activity prefs (`ts:{id}`). A fresh start always
applies, an update without `timestamp` applies and keeps the old value, and `end` (or a
user dismissal seen by `list`) clears it. The bridge returns `applied` on start and update.

Gap found: the Rust mobile wrapper (`src/mobile.rs` `update_activity`) maps any native
response to `true`, so `applied: false` from iOS or Android does not reach JS yet.

## Not verifiable without Samsung hardware

These need a Samsung device: the `ongoingActivityNoti` extras on One UI 7 (allow-list
only), Now Bar behaviour on One UI 8+ (AOSP path, allow-list or developer option), whether
the cover-screen list shows a widget with 110 dp minimums, and the host category the cover
screen reports.
