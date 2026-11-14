import AppIntents
import WidgetKit
import SwiftUI
import TauriWidgets

// Same shape as templates/macos-widget/MyWidget.swift.

// No `AppIntentsPackage` here. TauriWidgets is a static Swift package: its intents are
// already in this extension's App Intents metadata. Listing `TauriWidgetsIntentsPackage`
// adds an `extract.packagedata` include that linkd checks against a dynamic-library
// symbol; it finds none, drops ALL of the extension's intents, and every widget button /
// toggle then just opens the app ("aggregateMetadataIsEmpty" in the linkd log).

@main
struct Widgets: WidgetBundle {
    var body: some Widget {
        MyWidget()
    }
}

struct MyWidget: Widget {
    let kind = "ExampleWidget"
    let appGroup = "R6VC7JTLE8.com.s00d.tauriwidgets.example"
    let widgetId = "example"

    var body: some WidgetConfiguration {
        StaticConfiguration(
            kind: kind,
            provider: TauriWidgetProvider(appGroup: appGroup, widgetId: widgetId, refreshMinutes: 15, kind: kind)
        ) { entry in
            TauriWidgetView(entry: entry)
        }
        .configurationDisplayName("Example Widget")
        .description("Powered by Tauri")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
        .contentMarginsDisabled()
    }
}
