#if os(iOS) && canImport(ActivityKit)
import ActivityKit
import SwiftUI
import WidgetKit

/// Shared with the host plugin (`ios/Sources/WidgetPlugin.swift`): ActivityKit pairs the
/// app's `Activity.request` with this extension's `ActivityConfiguration` by this type.
public struct TauriActivityAttributes: ActivityAttributes {
    public struct ContentState: Codable, Hashable {
        /// The `ActivityContent` request as JSON; `layout` is always filled by the plugin.
        public var json: String
        public init(json: String) { self.json = json }
    }
    public var id: String
    public var group: String
    public init(id: String, group: String) { self.id = id; self.group = group }
}

/// Parsed activity request; every slot falls back to something sensible.
struct TauriActivityContent {
    let raw: [String: Any]

    /// With `template` set, the stored `activity-template:{name}` (App Group) is filled
    /// with the content's `data` first (contract v3 §3).
    init(json: String, group: String? = nil) {
        let content = (json.data(using: .utf8).flatMap { try? JSONSerialization.jsonObject(with: $0) } as? [String: Any]) ?? [:]
        guard let name = content["template"] as? String, !name.isEmpty, let group,
              let tpl = TauriWidgetDataStore.readValue(forKey: TauriWidgetStoreKeys.activityTemplatePrefix + name, appGroup: group)
                .flatMap(TauriWidgetSnapshot.object) else {
            raw = content
            return
        }
        raw = TauriResolve.applyActivityTemplate(content, template: tpl)
    }

    func element(_ key: String) -> WidgetElement? {
        guard let obj = raw[key], !(obj is NSNull),
              let data = try? JSONSerialization.data(withJSONObject: obj) else { return nil }
        return try? JSONDecoder().decode(WidgetElement.self, from: data)
    }

    var title: String { raw["title"] as? String ?? "" }
    var icon: String? { raw["icon"] as? String }
    var tint: Color { (raw["tint"] as? String).map { Color(hex: $0) } ?? .accentColor }
    var progress: Double? { (raw["progress"] as? NSNumber)?.doubleValue }
    var endsAt: Date? { (raw["endsAt"] as? NSNumber).map { Date(timeIntervalSince1970: $0.doubleValue / 1000) } }
    var url: URL? { (raw["url"] as? String).flatMap(URL.init(string:)) }
    var keylineTint: Color? { (raw["keylineTint"] as? String).map { Color(hex: $0) } }
    var backgroundTint: Color? { (raw["backgroundTint"] as? String).map { Color(hex: $0) } }
    /// Lock Screen slot: `lockScreen` when set, else the shared `layout`.
    var lockScreenKey: String { element("lockScreen") != nil ? "lockScreen" : "layout" }
}

/// Add to your `WidgetBundle` next to the home-screen widgets:
///
/// ```swift
/// @main struct Widgets: WidgetBundle {
///     var body: some Widget { MyWidget(); TauriLiveActivity() }
/// }
/// ```
/// and set `NSSupportsLiveActivities = YES` in the **app's** Info.plist.
public struct TauriLiveActivity: Widget {
    public init() {}

    public var body: some WidgetConfiguration {
        ActivityConfiguration(for: TauriActivityAttributes.self) { ctx in
            let c = TauriActivityContent(json: ctx.state.json, group: ctx.attributes.group)
            // `isWidgetRoot` leaves the root background to the chrome (as `TauriWidgetView`
            // does with containerBackground); here the chrome is the Lock Screen banner.
            let key = c.lockScreenKey
            let bg = c.element(key).flatMap { $0.background == nil ? nil : $0 }
            slot(c, key, ctx.attributes)
                .frame(maxWidth: .infinity, alignment: .topLeading)
                .background { if let bg { WidgetChrome.background(for: bg) } }
                .activityBackgroundTint(bg == nil ? c.backgroundTint : Color.clear)
                .widgetURL(c.url)
        } dynamicIsland: { ctx in
            let c = TauriActivityContent(json: ctx.state.json, group: ctx.attributes.group)
            return DynamicIsland {
                DynamicIslandExpandedRegion(.leading) { optionalSlot(c, "islandLeading", ctx.attributes) }
                DynamicIslandExpandedRegion(.trailing) { optionalSlot(c, "islandTrailing", ctx.attributes) }
                DynamicIslandExpandedRegion(.center) { optionalSlot(c, "islandCenter", ctx.attributes) }
                DynamicIslandExpandedRegion(.bottom) { slot(c, "layout", ctx.attributes) }
            } compactLeading: {
                if c.element("compactLeading") != nil { slot(c, "compactLeading", ctx.attributes) } else { iconView(c) }
            } compactTrailing: {
                if c.element("compactTrailing") != nil { slot(c, "compactTrailing", ctx.attributes) } else { statusView(c) }
            } minimal: {
                if c.element("minimal") != nil { slot(c, "minimal", ctx.attributes) } else { iconView(c) }
            }
            .keylineTint(c.keylineTint)
            .widgetURL(c.url)
        }
    }

    /// Expanded-island region: drawn only when the request carries it.
    @ViewBuilder
    private func optionalSlot(_ c: TauriActivityContent, _ key: String, _ attrs: TauriActivityAttributes) -> some View {
        if c.element(key) != nil { slot(c, key, attrs) }
    }

    @ViewBuilder
    private func slot(_ c: TauriActivityContent, _ key: String, _ attrs: TauriActivityAttributes) -> some View {
        if let el = c.element(key) {
            DynamicElementView(element: el, isWidgetRoot: key == "layout" || key == "lockScreen")
                // Buttons in the layout report `widgetId = "activity.<id>"` (Rust `activity_widget_id`).
                .environment(\.tauriWidgetIdentity, (attrs.group, "activity." + attrs.id, nil))
        } else {
            Text(c.title).font(.headline).padding()
        }
    }

    @ViewBuilder
    private func iconView(_ c: TauriActivityContent) -> some View {
        Image(systemName: c.icon ?? "circle.fill").foregroundStyle(c.tint)
    }

    @ViewBuilder
    private func statusView(_ c: TauriActivityContent) -> some View {
        if let end = c.endsAt {
            Text(timerInterval: Date()...max(end, Date()), countsDown: true)
                .monospacedDigit().frame(maxWidth: 52)
        } else if let p = c.progress {
            Text("\(Int((p * 100).rounded()))%").monospacedDigit()
        } else {
            EmptyView()
        }
    }
}
#endif
