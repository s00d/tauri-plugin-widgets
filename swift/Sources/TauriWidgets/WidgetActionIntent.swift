import Foundation
import AppIntents
import WidgetKit

/// Global configuration — set before using widget components
/// (typically in your Widget's `init()` / provider).
public enum TauriWidgetsConfig {
    public static var appGroup: String = ""
    public static var widgetId: String = "default"

    /// App Group UserDefaults key: WidgetKit `kind` → `widgetId`, read by the host
    /// plugin's `getActiveWidgets` (WidgetCenter only knows kinds).
    public static let kindsKey = "tauri_widget_kinds"

    /// `appGroup`, or Info.plist `TauriWidgetsAppGroup` when no provider ran in this
    /// process yet (an entity query or intent can run first).
    static var resolvedAppGroup: String {
        appGroup.isEmpty ? (Bundle.main.object(forInfoDictionaryKey: "TauriWidgetsAppGroup") as? String ?? "") : appGroup
    }

    /// Called by the providers' init; records `kind` → `widgetId` (written only on change).
    static func register(appGroup: String, widgetId: String, kind: String?) {
        self.appGroup = appGroup
        self.widgetId = widgetId
        guard let kind, !appGroup.isEmpty, let d = UserDefaults(suiteName: appGroup) else { return }
        var map = d.dictionary(forKey: kindsKey) as? [String: String] ?? [:]
        guard map[kind] != widgetId else { return }
        map[kind] = widgetId
        d.set(map, forKey: kindsKey)
    }
}

/// AppIntents package of this library. Only for a DYNAMIC link (a framework): then the
/// extension lists it in its own `AppIntentsPackage.includedPackages`. The usual static
/// Swift package link must NOT list it: the intents are already in the extension's
/// metadata, and linkd rejects an include it cannot match to an imported symbol, which
/// drops every intent of the extension (widget taps just open the app).
public struct TauriWidgetsIntentsPackage: AppIntentsPackage {}

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
        let group = (appGroup?.isEmpty == false ? appGroup! : TauriWidgetsConfig.resolvedAppGroup)
        guard !group.isEmpty else { return .result() }

        let wid = (widgetId?.isEmpty == false ? widgetId! : TauriWidgetsConfig.widgetId)
        let envelope = WidgetActionEnvelope(
            action: actionName,
            payload: payload,
            ts: UInt64(Date().timeIntervalSince1970 * 1000),
            widgetId: wid,
            group: group
        )
        TauriWidgetDataStore.appendPendingAction(envelope, appGroup: group)
        // The system asks for a new timeline as soon as this returns; a reload the app sends
        // while that request runs is held back for seconds. So give a running app a moment to
        // answer (it writes the widget's state or config) and return when it did: the
        // timeline then already shows the answer (calculator "=").
        await Self.awaitHostAnswer(appGroup: group, widgetId: wid, timeout: Self.hostAnswerTimeout)
        return .result()
    }

    /// Longest wait for the app's answer to an action (an app that is not running costs this).
    static let hostAnswerTimeout: TimeInterval = 1.0

    /// Wait until the widget's `state:` or `config:` entry changes in the store, or `timeout`.
    /// Returns whether the app answered.
    @discardableResult
    static func awaitHostAnswer(appGroup: String, widgetId: String, timeout: TimeInterval,
                                poll: TimeInterval = 0.05) async -> Bool {
        let keys = [TauriWidgetStoreKeys.statePrefix + widgetId, TauriWidgetStoreKeys.configKey(widgetId)]
        func current() -> [String?] {
            let map = TauriWidgetDataStore.loadFreshestMap(appGroup: appGroup)
            return keys.map { map[$0] }
        }
        let before = current()
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            try? await Task.sleep(nanoseconds: UInt64(poll * 1_000_000_000))
            if current() != before { return true }
        }
        return false
    }
}

/// `set` on a button / toggle / link (contract v3): writes the placement's state in the
/// extension, without the app, then redraws. An `action` on the same element is still
/// queued for the app (after the write), and a `$state` envelope tells a running app the
/// new state (`widget-state-changed`).
public struct WidgetStateIntent: AppIntent {
    public static var title: LocalizedStringResource = "Widget State"
    /// Taps only; never offered in Shortcuts.
    public static var isDiscoverable: Bool = false

    /// The `set` object as JSON.
    @Parameter(title: "Patch")
    public var patch: String

    @Parameter(title: "Action")
    public var actionName: String?

    @Parameter(title: "Payload")
    public var payload: String?

    @Parameter(title: "Widget ID")
    public var widgetId: String?

    @Parameter(title: "Instance")
    public var instance: String?

    @Parameter(title: "App Group")
    public var appGroup: String?

    public init() {
        patch = "{}"; actionName = nil; payload = nil; widgetId = nil; instance = nil; appGroup = nil
    }

    public init(patch: [String: Any], actionName: String? = nil, payload: String? = nil,
                widgetId: String?, instance: String?, appGroup: String?) {
        let data = (try? JSONSerialization.data(withJSONObject: patch, options: [.sortedKeys])) ?? Data("{}".utf8)
        self.patch = String(decoding: data, as: UTF8.self)
        self.actionName = actionName
        self.payload = payload
        self.widgetId = widgetId ?? TauriWidgetsConfig.widgetId
        self.instance = instance
        self.appGroup = appGroup ?? TauriWidgetsConfig.appGroup
    }

    /// Store write + envelopes; split from `perform` for tests.
    @discardableResult
    static func run(patch raw: String, actionName: String?, payload: String?, widgetId: String,
                    instance: String?, group: String, now: UInt64 = UInt64(Date().timeIntervalSince1970 * 1000)) -> [String: Any] {
        let patch = (TauriWidgetSnapshot.json(raw) as? [String: Any]) ?? [:]
        let inst = instance?.isEmpty == false ? instance : nil
        let state = TauriWidgetDataStore.applyStatePatch(patch, appGroup: group, widgetId: widgetId, instance: inst)
        if let actionName, !actionName.isEmpty {
            TauriWidgetDataStore.appendPendingAction(
                WidgetActionEnvelope(action: actionName, payload: payload, ts: now, widgetId: widgetId, group: group),
                appGroup: group)
        }
        let event: [String: Any] = ["widgetId": widgetId, "instance": inst ?? NSNull(), "state": state]
        if let data = try? JSONSerialization.data(withJSONObject: event, options: [.sortedKeys]) {
            TauriWidgetDataStore.appendPendingAction(
                WidgetActionEnvelope(action: stateEnvelopeAction, payload: String(decoding: data, as: UTF8.self),
                                     ts: now, widgetId: widgetId, group: group),
                appGroup: group)
        }
        return state
    }

    /// Action name of the envelope the host turns into `widget-state-changed`.
    public static let stateEnvelopeAction = "$state"

    public func perform() async throws -> some IntentResult {
        let group = (appGroup?.isEmpty == false ? appGroup! : TauriWidgetsConfig.resolvedAppGroup)
        guard !group.isEmpty else { return .result() }
        let wid = (widgetId?.isEmpty == false ? widgetId! : TauriWidgetsConfig.widgetId)
        Self.run(patch: patch, actionName: actionName, payload: payload, widgetId: wid, instance: instance, group: group)
        WidgetCenter.shared.reloadAllTimelines()
        return .result()
    }
}
