import WidgetKit
import SwiftUI
import TauriWidgets

// ─── Entry Point ─────────────────────────────────────────────────────────────
// This is the only file you need in your Widget Extension target.
// Adjust `appGroup` and `kind` to match your app's configuration.

@main
struct MyWidget: Widget {
    let kind = "MyTauriWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(
            kind: kind,
            provider: TauriWidgetProvider(appGroup: "group.com.s00d.tauri-plugin-widgets-example")
        ) { entry in
            TauriWidgetView(entry: entry)
        }
        .configurationDisplayName("My Widget")
        .description("Powered by Tauri")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
        .contentMarginsDisabled()
    }
}
