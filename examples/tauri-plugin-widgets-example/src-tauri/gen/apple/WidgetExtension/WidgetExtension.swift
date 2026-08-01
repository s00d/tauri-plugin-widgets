import WidgetKit
import SwiftUI
import TauriWidgets

struct WidgetExtension: Widget {
    let kind = "ExampleWidget"
    let appGroup = "group.com.s00d.tauri-plugin-widgets-example"
    let widgetId = "example"

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
