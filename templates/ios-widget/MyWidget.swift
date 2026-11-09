import WidgetKit
import SwiftUI
import TauriWidgets

// ─── Widget entry point ──────────────────────────────────────────────────────
// `npx tauri-widgets init-ios` adds the WidgetExtension target for this folder to
// src-tauri/gen/apple/project.yml, so this file is yours: edit it freely, init-ios
// never overwrites it (unless --force).
//
// widgetId must match the JS side: setWidgetConfig(config) uses "default".

@main
struct MyWidget: Widget {
    let kind = "{{WIDGET_KIND}}"
    let appGroup = "{{APP_GROUP}}"
    let widgetId = "{{WIDGET_ID}}"

    var body: some WidgetConfiguration {
        StaticConfiguration(
            kind: kind,
            provider: TauriWidgetProvider(appGroup: appGroup, widgetId: widgetId, refreshMinutes: 15)
        ) { entry in
            TauriWidgetView(entry: entry)
        }
        .configurationDisplayName("My Widget")
        .description("Powered by Tauri")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
        .contentMarginsDisabled()
    }
}
