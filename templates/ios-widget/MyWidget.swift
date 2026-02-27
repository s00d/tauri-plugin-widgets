import WidgetKit
import SwiftUI
import TauriWidgets

// ─── Entry Point ─────────────────────────────────────────────────────────────
// Add this file to your iOS Widget Extension target.
// 1. Open gen/apple/*.xcodeproj in Xcode
// 2. File → New → Target → Widget Extension
// 3. Add TauriWidgets Swift Package as a dependency
// 4. Replace the generated Swift code with this file
// 5. Enable App Groups in BOTH targets (App and Widget Extension)

@main
struct MyWidget: Widget {
    let kind = "MyTauriWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(
            kind: kind,
            provider: TauriWidgetProvider(appGroup: "group.com.example.myapp")
        ) { entry in
            TauriWidgetView(entry: entry)
        }
        .configurationDisplayName("My Widget")
        .description("Powered by Tauri")
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
        .contentMarginsDisabled()
    }
}
