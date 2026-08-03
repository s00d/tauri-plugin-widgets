import WidgetKit
import SwiftUI
import TauriWidgets

// Enable App Groups in BOTH targets with:
// group.com.s00d.tauriwidgets.example

@main
struct MyWidget: Widget {
    let kind = "ExampleWidget"
    let appGroup = "group.com.s00d.tauriwidgets.example"
    let widgetId = "example"

    var body: some WidgetConfiguration {
        StaticConfiguration(
            kind: kind,
            provider: TauriWidgetProvider(appGroup: appGroup, widgetId: widgetId, refreshMinutes: 15)
        ) { entry in
            TauriWidgetView(entry: entry)
        }
        .configurationDisplayName("Example Widget")
        .description("Powered by Tauri")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
        .contentMarginsDisabled()
    }
}
