import AppIntents
import WidgetKit
import SwiftUI
import TauriWidgets

// ─── Widget entry point ──────────────────────────────────────────────────────
// `npx tauri-widgets init-ios` adds the WidgetExtension target for this folder to
// src-tauri/gen/apple/project.yml, so this file is yours: edit it freely, init-ios
// never overwrites it (unless --force).
//
// widgetId must match the JS side: setWidgetConfig(config) uses "default".

// Lets the host's getActiveWidgets map WidgetCenter kinds to widgetIds: providers get `kind:`.
// The Edit Widget item query may run before any provider; it then reads the App Group from
// the extension Info.plist key `TauriWidgetsAppGroup` (add it: <string>{{APP_GROUP}}</string>).

// No `AppIntentsPackage` here. TauriWidgets is a static Swift package: its intents are
// already in this extension's App Intents metadata. Listing `TauriWidgetsIntentsPackage`
// adds an `extract.packagedata` include that linkd checks against a dynamic-library
// symbol; it finds none, drops ALL of the extension's intents, and every widget button /
// toggle then just opens the app ("aggregateMetadataIsEmpty" in the linkd log).

@main
struct Widgets: WidgetBundle {
    var body: some Widget {
        MyWidget()
        // Live activities from `startActivity()` (Lock Screen + Dynamic Island).
        TauriLiveActivity()
        // Per-placement widget: uncomment, then `setWidgetInstances("{{WIDGET_ID}}", [{ id, title }])`.
        // MyInstanceWidget()
    }
}

struct MyWidget: Widget {
    let kind = "{{WIDGET_KIND}}"
    let appGroup = "{{APP_GROUP}}"
    let widgetId = "{{WIDGET_ID}}"

    var body: some WidgetConfiguration {
        StaticConfiguration(
            kind: kind,
            provider: TauriWidgetProvider(appGroup: appGroup, widgetId: widgetId, refreshMinutes: 15, kind: kind)
        ) { entry in
            TauriWidgetView(entry: entry)
        }
        .configurationDisplayName("My Widget")
        .description("Powered by Tauri")
        // Accessory families draw the config's `accessory` layout (else `small`) on the
        // Lock Screen; the system tints it, so backgrounds are dropped there.
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge,
                            .accessoryCircular, .accessoryRectangular, .accessoryInline])
        .contentMarginsDisabled()
        // iOS 26 widget push (your server reloads the widget over APNs; token from
        // getWidgetPushToken): with an iOS 26 deployment target and the Push Notifications
        // capability, add `.pushHandler(TauriWidgetPushHandler.self)`.
    }
}

/// The user picks an item in Edit Widget; it becomes this placement's `{{instance}}`.
struct MyInstanceWidget: Widget {
    let kind = "{{WIDGET_KIND}}.instance"
    let appGroup = "{{APP_GROUP}}"
    let widgetId = "{{WIDGET_ID}}"

    var body: some WidgetConfiguration {
        AppIntentConfiguration(
            kind: kind,
            intent: TauriInstanceIntent.self,
            provider: TauriInstanceProvider(appGroup: appGroup, widgetId: widgetId, refreshMinutes: 15, kind: kind)
        ) { entry in
            TauriWidgetView(entry: entry)
        }
        .configurationDisplayName("My Widget (choose)")
        .description("Powered by Tauri")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
        .contentMarginsDisabled()
    }
}
