import WidgetKit
import SwiftUI
import TauriWidgets

// ─── Entry Point ─────────────────────────────────────────────────────────────
// Adjust `appGroup`, `kind`, and `widgetId` to match your app.
// Placeholders: {{APP_GROUP}}, {{WIDGET_KIND}}, {{WIDGET_ID}}

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
