---
title: Android setup
---

# Android setup

## Step 1: Initialize

```bash
pnpm tauri android init
```

The plugin automatically registers its Android bridge and Glance widget receiver from the plugin AAR — you do **not** write a Glance `AppWidget` in Kotlin for the default path.

**No Kotlin code required** — the built-in receiver reads widget config from shared storage and renders it via Jetpack Glance.

## Step 2: Match the SharedPreferences group

The Glance receiver resolves the store name in this order (`WidgetStoreKeys.resolveGroup`):

1. Per-instance meta `widgetId:{appWidgetId}` binding (after pin)
2. Manifest `<meta-data android:name="tauri_widget_group" …/>` on the Glance receiver
3. Fallback candidates: app **package name**, `group.<package>`, and hyphenated variants

**Your JS `group` argument to `setWidgetConfig` must be one of those names.** A random `group.com.example.myapp` from the desktop tutorial will **not** match unless you set the same value in meta-data.

Simplest path — use the package name:

```typescript
// e.g. identifier / applicationId is com.example.myapp
await setWidgetConfig(config, "com.example.myapp", widgetId);
```

Optional custom group — add meta-data on the **merged** application (keep the plugin’s `APPWIDGET_UPDATE` receiver intact). Prefer only injecting meta-data rather than replacing the whole `<receiver>`:

```xml
<!-- In your app AndroidManifest <application>, after merger with the plugin: -->
<meta-data
    android:name="tauri_widget_group"
    android:value="group.com.example.myapp" />
```

If you override the Glance `<receiver>` yourself, keep the plugin class, provider meta-data, and intent-filter, and declare `xmlns:tools` when using `tools:replace`:

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    xmlns:tools="http://schemas.android.com/tools">
  <application>
    <receiver
        android:name="git.s00d.widgets.TauriGlanceWidgetReceiver"
        android:exported="true"
        android:label="My Widget"
        tools:replace="android:label">
      <intent-filter>
        <action android:name="android.appwidget.action.APPWIDGET_UPDATE" />
      </intent-filter>
      <meta-data
          android:name="android.appwidget.provider"
          android:resource="@xml/tauri_widget_info" />
      <meta-data
          android:name="tauri_widget_group"
          android:value="group.com.example.myapp" />
    </receiver>
  </application>
</manifest>
```

Then call `setWidgetConfig(..., "group.com.example.myapp", widgetId)` with the **same** string.

## Step 3: Run

```bash
pnpm tauri android dev
```

## Step 4: Add widget to home screen

Long-press the home screen → **Widgets** → find your app → drag to screen.

After pinning, call `setWidgetConfig` for the logical `widgetId` bound to that instance (see [Multi-widget](/guide/recipes/multi-widget)).

## Custom Widget Provider (advanced)

If you need custom behavior, create your own `GlanceAppWidget` / `GlanceAppWidgetReceiver` and keep the same shared-group + `config:{widgetId}` contract.

---

## When it fails

- Empty tile: JS `group` ≠ what Glance resolves (log package name / meta-data).
- Check `adb logcat | grep -i widget` for errors.
- Canvas bitmaps larger than 512px are automatically capped.
- Bitmap data is compressed to stay under the Binder IPC 500KB limit.
- Prefer flat `vstack` / `hstack`; always set `progress.label`.

More symptoms: [Troubleshooting index](/guide/troubleshooting).
