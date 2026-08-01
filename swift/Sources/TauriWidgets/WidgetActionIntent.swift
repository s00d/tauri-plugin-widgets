import Foundation
import AppIntents

/// Global configuration — set before using widget components
/// (typically in your Widget's `init()`).
public enum TauriWidgetsConfig {
    public static var appGroup: String = ""
    public static var widgetId: String = "default"
}

/// AppIntent that stores pending actions via multi-transport fan-out.
/// The host app polls these and emits Tauri `widget-action` events.
public struct WidgetActionIntent: AppIntent {
    public static var title: LocalizedStringResource = "Widget Action"

    @Parameter(title: "Action")
    public var actionName: String

    @Parameter(title: "Payload")
    public var payload: String?

    @Parameter(title: "Widget ID")
    public var widgetId: String?

    public init() {
        self.actionName = ""
        self.payload = nil
        self.widgetId = nil
    }

    public init(actionName: String, payload: String? = nil, widgetId: String? = nil) {
        self.actionName = actionName
        self.payload = payload
        self.widgetId = widgetId
    }

    public func perform() async throws -> some IntentResult {
        let group = TauriWidgetsConfig.appGroup
        guard !group.isEmpty else { return .result() }

        let wid = widgetId?.isEmpty == false ? widgetId! : TauriWidgetsConfig.widgetId
        var pending = TauriWidgetDataStore.readPendingActions(appGroup: group)
        let envelope = WidgetActionEnvelope(
            action: actionName,
            payload: payload,
            ts: UInt64(Date().timeIntervalSince1970 * 1000),
            widgetId: wid,
            group: group
        )
        pending.append(envelope)
        TauriWidgetDataStore.writePendingActions(pending, appGroup: group)
        return .result()
    }
}
