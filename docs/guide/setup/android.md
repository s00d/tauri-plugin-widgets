---
title: Android setup
---

# Android setup

## Step 1: Initialize

```bash
pnpm tauri android init
```

The plugin automatically registers its Android bridge and Glance widget receiver.

**No Kotlin code required** — the built-in receiver reads widget config from shared storage and renders it via Jetpack Glance.

## Step 2: Configure the App Group (optional)

By default, the plugin uses the application package name as the `SharedPreferences` group. To use a custom group, add a `<meta-data>` tag in your app's `AndroidManifest.xml`:

```xml
<receiver
    android:name="git.s00d.widgets.TauriGlanceWidgetReceiver"
    tools:replace="android:label"
    android:label="My Widget">
    <meta-data
        android:name="tauri_widget_group"
        android:value="group.com.example.myapp" />
</receiver>
```

## Step 3: Run

```bash
pnpm tauri android dev
```

## Step 4: Add widget to home screen

Long-press the home screen → **Widgets** → find your app → drag to screen.

## Custom Widget Provider (advanced)

If you need custom behavior, create your own `GlanceAppWidget` / `GlanceAppWidgetReceiver` and keep the same shared group contract.

---

## When it fails

- Check `adb logcat | grep -i widget` for errors.
- Canvas bitmaps larger than 512px are automatically capped.
- Bitmap data is compressed to stay under the Binder IPC 500KB limit.
- Prefer flat `vstack` / `hstack`; always set `progress.label`.

More symptoms: [Troubleshooting index](/guide/troubleshooting).
