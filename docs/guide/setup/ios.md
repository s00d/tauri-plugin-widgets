---
title: iOS setup
---

# iOS setup

## Step 1: Initialize the iOS project

```bash
pnpm tauri ios init
```

## Step 2: Add a Widget Extension target

Open the generated Xcode project:

```bash
open src-tauri/gen/apple/*.xcodeproj
```

In Xcode: **File → New → Target → Widget Extension**. Name it (e.g. `WidgetExtension`), language: Swift.

In the **Choose options for your new target** dialog:
- Set `Product Name` to `WidgetExtension` (or your widget name)
- Select your Apple `Team` (e.g. Personal Team)
- Keep `Project` as your current iOS project (e.g. `myapp`)
- Set `Embed in Application` to your iOS app target (e.g. `myapp_iOS`)
- For the basic plugin setup, disable:
  - `Include Live Activity`
  - `Include Control`
  - `Include Configuration App Intent`

## Step 3: Add the TauriWidgets Swift Package

In Xcode:
1. **File → Add Package Dependencies...**
2. Click **Add Local...**
3. Select the package folder:
   - for typical app projects: `node_modules/tauri-plugin-widgets-api/swift/`
   - for this repository example: `swift/` (repository root)
4. In **Add to Target**, choose your widget target (`WidgetExtension` / `WidgetExtensionExtension`).

Important: `TauriWidgets` must be linked to the widget target itself. If it is linked only to the main iOS app target, `import TauriWidgets` fails with `no such module`.

## Step 4: Sync the widget code via CLI (recommended)

After creating the Widget Extension target, run:

```bash
npx tauri-plugin-widgets-api init-ios
```

This updates the generated `src-tauri/gen/apple/*/*.swift` widget entry file from the plugin template using the auto-generated App Group.
It also adapts the widget struct name to match Xcode-generated `*Bundle.swift` references, preventing `cannot find 'WidgetExtension' in scope`.

Manual fallback (if you prefer to edit/copy by hand):

```swift
import SwiftUI
import WidgetKit
import TauriWidgets

struct MyWidgetEntryView: View {
    var entry: TauriWidgetEntry
    var body: some View {
        TauriWidgetView(entry: entry)
    }
}

@main
struct MyWidget: Widget {
    let kind = "ExampleWidget"
    var body: some WidgetConfiguration {
        StaticConfiguration(
            kind: kind,
            // Use the same App Group that `init-ios` generated for your project.
            provider: TauriWidgetProvider(appGroup: "group.<your-tauri-identifier>")
        ) { entry in
            MyWidgetEntryView(entry: entry)
        }
        .configurationDisplayName("My Widget")
        .description("Powered by TauriWidgets")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
    }
}
```

Or copy from `templates/ios-widget/MyWidget.swift`.

## Step 5: Configure App Groups

1. Select the **main app target** (your iOS app, e.g. `myapp_iOS`) → **Signing & Capabilities**.
2. Click **+ Capability** → add **App Groups**.
3. In the App Groups block click `+` and add your group (use the value printed by `init-ios`).
4. Repeat the same for the **WidgetExtension** target.
5. Verify the App Group value is **exactly the same** in both targets.
6. If **+ Capability** is disabled, set a valid **Team** in Signing for that target first.

## Step 6: Run

```bash
pnpm tauri ios dev
```

---

## When it fails

### Widget shows "No configuration"

- Ensure the App Group identifier is **identical** in the main app and widget extension (Xcode → Signing & Capabilities → App Groups).
- Call `setWidgetConfig(...)` from your app before adding the widget.

### Widget doesn't update

- Apple limits widget refreshes to ~40–70 per day.
- Check plugin throttle: `TAURI_WIDGET_MIN_RELOAD_SECS` (Debug default `0`, Release default `900`). Inspect `ApplyOutcome.reload` from `setWidgetConfig` — throttled reloads no longer look like success-only `true`.
- For testing: in Xcode, use **Debug → Simulate Timeline → After Refresh**.
- For live counters, use `{ type: "timer" }` instead of frequent reloads.

More symptoms: [Troubleshooting index](/guide/troubleshooting). Transport: [Apple data transport](/guide/transport).
