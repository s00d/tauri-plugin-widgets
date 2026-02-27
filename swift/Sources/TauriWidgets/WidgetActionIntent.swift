import Foundation
import AppIntents

/// Global configuration — set `TauriWidgetsConfig.appGroup` before using
/// any widget components (typically in your Widget's `init()`).
public enum TauriWidgetsConfig {
    public static var appGroup: String = ""
}

/// AppIntent that stores pending actions in the shared container.
/// The host app polls these and emits Tauri `widget-action` events.
public struct WidgetActionIntent: AppIntent {
    public static var title: LocalizedStringResource = "Widget Action"

    @Parameter(title: "Action")
    public var actionName: String

    public init() { self.actionName = "" }
    public init(actionName: String) { self.actionName = actionName }

    public func perform() async throws -> some IntentResult {
        let group = TauriWidgetsConfig.appGroup
        guard !group.isEmpty else { return .result() }

        var pending: [String] = []
        if let raw = TauriWidgetDataStore.readValue(forKey: "__widget_pending_actions__", appGroup: group),
           let data = raw.data(using: .utf8),
           let arr = try? JSONDecoder().decode([String].self, from: data) {
            pending = arr
        }
        pending.append(actionName)
        if let data = try? JSONEncoder().encode(pending),
           let str = String(data: data, encoding: .utf8) {
            TauriWidgetDataStore.writeValue(str, forKey: "__widget_pending_actions__", appGroup: group)
        }
        return .result()
    }
}
