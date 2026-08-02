import Foundation
import AppIntents

/// Global configuration — set before using widget components
/// (typically in your Widget's `init()` / provider).
public enum TauriWidgetsConfig {
    public static var appGroup: String = ""
    public static var widgetId: String = "default"
}

/// AppIntent that stores pending actions via multi-transport fan-out.
/// The host app polls these and emits Tauri `widget-action` events.
///
/// `appGroup` / `widgetId` must be passed as intent parameters — AppIntents
/// often run out-of-process where `TauriWidgetsConfig` statics are empty.
public struct WidgetActionIntent: AppIntent {
    public static var title: LocalizedStringResource = "Widget Action"

    @Parameter(title: "Action")
    public var actionName: String

    @Parameter(title: "Payload")
    public var payload: String?

    @Parameter(title: "Widget ID")
    public var widgetId: String?

    @Parameter(title: "App Group")
    public var appGroup: String?

    public init() {
        self.actionName = ""
        self.payload = nil
        self.widgetId = nil
        self.appGroup = nil
    }

    public init(
        actionName: String,
        payload: String? = nil,
        widgetId: String? = nil,
        appGroup: String? = nil
    ) {
        self.actionName = actionName
        self.payload = payload
        self.widgetId = widgetId ?? TauriWidgetsConfig.widgetId
        self.appGroup = appGroup ?? TauriWidgetsConfig.appGroup
    }

    public func perform() async throws -> some IntentResult {
        let group = (appGroup?.isEmpty == false ? appGroup! : TauriWidgetsConfig.appGroup)
        guard !group.isEmpty else { return .result() }

        let wid = (widgetId?.isEmpty == false ? widgetId! : TauriWidgetsConfig.widgetId)
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
