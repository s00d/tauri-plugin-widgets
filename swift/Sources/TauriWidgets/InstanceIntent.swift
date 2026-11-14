import AppIntents
import Foundation
import SwiftUI
import WidgetKit

// Per-placement configuration: the user picks an item (a city, an account) in the Edit
// Widget sheet; the choice is the placement's `instance`. The app publishes the items with
// `setWidgetInstances(widgetId, [{id, title}])` and can give one placement its own layout
// with `setWidgetConfig(config, group, widgetId, false, instance)` or its own data with
// `setWidgetData(widgetId, data, { instance })`. `{{instance}}` also fills `source.url`.

/// One choice of the Edit Widget sheet.
public struct TauriInstanceEntity: AppEntity, Hashable {
    public let id: String
    public let title: String

    public init(id: String, title: String) {
        self.id = id
        self.title = title
    }

    public static var typeDisplayRepresentation: TypeDisplayRepresentation = "Item"
    public var displayRepresentation: DisplayRepresentation { DisplayRepresentation(title: "\(title)") }
    public static var defaultQuery = TauriInstanceQuery()
}

/// Items from every `instances:{widgetId}` list in the App Group.
///
/// ponytail: one merged list per App Group — an EntityQuery cannot see which widget kind
/// asks. Give items of different widgets distinct ids (or one widget per extension) if
/// they must not mix; a typed intent per kind is the upgrade path.
public struct TauriInstanceQuery: EntityQuery {
    public init() {}

    static func all() -> [TauriInstanceEntity] {
        TauriWidgetDataStore.loadInstanceOptions(appGroup: TauriWidgetsConfig.resolvedAppGroup)
            .map { TauriInstanceEntity(id: $0.id, title: $0.title) }
    }

    public func entities(for identifiers: [String]) async throws -> [TauriInstanceEntity] {
        let known = Self.all()
        // A choice the app no longer lists stays selected (shown by its id).
        return identifiers.map { id in known.first { $0.id == id } ?? TauriInstanceEntity(id: id, title: id) }
    }

    public func suggestedEntities() async throws -> [TauriInstanceEntity] {
        Self.all()
    }

    /// A new placement starts on the first listed item instead of an empty choice.
    public func defaultResult() async -> TauriInstanceEntity? {
        Self.all().first
    }
}

/// Configuration intent of a per-placement widget.
public struct TauriInstanceIntent: WidgetConfigurationIntent {
    public static var title: LocalizedStringResource = "Choose"
    public static var description = IntentDescription("Pick what this widget shows.")

    @Parameter(title: "Item")
    public var instance: TauriInstanceEntity?

    public init() {}

    public init(instance: TauriInstanceEntity?) {
        self.instance = instance
    }
}

/// Timeline provider for `AppIntentConfiguration(kind:intent: TauriInstanceIntent.self, …)`.
public struct TauriInstanceProvider: AppIntentTimelineProvider {
    public typealias Entry = TauriWidgetEntry
    public typealias Intent = TauriInstanceIntent

    public let appGroup: String
    public let widgetId: String
    public let refreshMinutes: Int

    /// Pass the widget's `kind` so the host's `getActiveWidgets` can map it to `widgetId`.
    public init(appGroup: String, widgetId: String = "default", refreshMinutes: Int = 15, kind: String? = nil) {
        self.appGroup = appGroup
        self.widgetId = widgetId
        self.refreshMinutes = max(1, refreshMinutes)
        TauriWidgetsConfig.register(appGroup: appGroup, widgetId: widgetId, kind: kind)
    }

    public func placeholder(in context: Context) -> TauriWidgetEntry {
        TauriWidgetEntry(date: Date(), config: nil, family: context.family, appGroup: appGroup, widgetId: widgetId)
    }

    public func snapshot(for configuration: TauriInstanceIntent, in context: Context) async -> TauriWidgetEntry {
        if context.isPreview {
            return TauriTimeline.preview(appGroup: appGroup, widgetId: widgetId, instance: configuration.instance?.id,
                                         family: context.family) ?? placeholder(in: context)
        }
        let t = await timeline(configuration, context, trigger: "snapshot")
        return t.entries.first ?? placeholder(in: context)
    }

    public func timeline(for configuration: TauriInstanceIntent, in context: Context) async -> Timeline<TauriWidgetEntry> {
        await timeline(configuration, context, trigger: "timeline")
    }

    private func timeline(_ intent: TauriInstanceIntent, _ context: Context, trigger: String) async -> Timeline<TauriWidgetEntry> {
        await withCheckedContinuation { cont in
            TauriTimeline.timeline(
                appGroup: appGroup, widgetId: widgetId, instance: intent.instance?.id, family: context.family,
                refreshMinutes: refreshMinutes, trigger: trigger
            ) { cont.resume(returning: $0) }
        }
    }
}
