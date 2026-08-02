import Foundation
import SwiftUI
import WidgetKit

// MARK: - Timeline Entry

public struct TauriWidgetEntry: TimelineEntry {
    public let date: Date
    public let config: WidgetUIConfig?
    public let family: WidgetFamily

    public init(date: Date, config: WidgetUIConfig?, family: WidgetFamily) {
        self.date = date; self.config = config; self.family = family
    }
}

// MARK: - Timeline Provider

public struct TauriWidgetProvider: TimelineProvider {
    public let appGroup: String
    public let widgetId: String
    /// Timeline refresh interval in minutes (default 15 to respect WidgetKit budget).
    public let refreshMinutes: Int

    public init(appGroup: String, widgetId: String = "default", refreshMinutes: Int = 15) {
        self.appGroup = appGroup
        self.widgetId = widgetId
        self.refreshMinutes = max(1, refreshMinutes)
        TauriWidgetsConfig.appGroup = appGroup
        TauriWidgetsConfig.widgetId = widgetId
    }

    public func placeholder(in context: Context) -> TauriWidgetEntry {
        TauriWidgetEntry(date: Date(), config: nil, family: context.family)
    }

    public func getSnapshot(in context: Context, completion: @escaping (TauriWidgetEntry) -> Void) {
        let cfg: WidgetUIConfig?
        if context.isPreview {
            cfg = nil
        } else {
            let (loaded, source, nonce) = TauriWidgetDataStore.loadConfigWithSource(
                appGroup: appGroup, widgetId: widgetId
            )
            cfg = loaded
            writeProviderReceipt(context: context, source: source, nonce: nonce, config: loaded, trigger: context.isPreview ? "added" : "snapshot")
        }
        completion(TauriWidgetEntry(date: Date(), config: cfg, family: context.family))
    }

    public func getTimeline(in context: Context, completion: @escaping (Timeline<TauriWidgetEntry>) -> Void) {
        let (cfg, source, nonce) = TauriWidgetDataStore.loadConfigWithSource(
            appGroup: appGroup, widgetId: widgetId
        )
        writeProviderReceipt(context: context, source: source, nonce: nonce, config: cfg, trigger: "timeline")
        let entry = TauriWidgetEntry(date: Date(), config: cfg, family: context.family)
        let nextUpdate = Calendar.current.date(byAdding: .minute, value: refreshMinutes, to: Date()) ?? Date()
        completion(Timeline(entries: [entry], policy: .after(nextUpdate)))
    }

    private func writeProviderReceipt(
        context: Context,
        source: String,
        nonce: UInt64,
        config: WidgetUIConfig?,
        trigger: String
    ) {
        let size: String = {
            switch context.family {
            case .systemSmall: return "small"
            case .systemMedium: return "medium"
            case .systemLarge: return "large"
            default: return "medium"
            }
        }()
        var rendered: [String] = []
        if let el = config?.small ?? config?.medium ?? config?.large {
            collectTypes(el, into: &rendered)
        }
        let receipt = WidgetTransportReceipt(
            source: source,
            widgetId: widgetId,
            group: appGroup,
            instance: "\(context.family)",
            nonce: nonce,
            size: size,
            schema: 1,
            rendered: rendered,
            skipped: [],
            trigger: trigger
        )
        TauriWidgetDataStore.writeReceiptEverywhere(receipt, appGroup: appGroup)
    }

    private func collectTypes(_ el: WidgetElement, into out: inout [String]) {
        out.append(el.type)
        el.children?.forEach { collectTypes($0, into: &out) }
    }
}

// MARK: - Widget View

public struct TauriWidgetView: View {
    public var entry: TauriWidgetEntry

    public init(entry: TauriWidgetEntry) { self.entry = entry }

    public var body: some View {
        if let el = layoutForFamily() {
            DynamicElementView(element: el, isWidgetRoot: true)
                // Pin to top — default center left a fake “top padding” when content is shorter than the canvas.
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                .containerBackground(for: .widget) { WidgetChrome.background(for: el) }
        } else {
            placeholderView()
                .containerBackground(for: .widget) {
                    LinearGradient(colors: [Color.indigo, Color.purple],
                                   startPoint: .topLeading, endPoint: .bottomTrailing)
                }
        }
    }

    private func layoutForFamily() -> WidgetElement? {
        guard let cfg = entry.config else { return nil }
        switch entry.family {
        case .systemSmall:  return cfg.small ?? cfg.medium ?? cfg.large
        case .systemMedium: return cfg.medium ?? cfg.large ?? cfg.small
        case .systemLarge:  return cfg.large ?? cfg.medium ?? cfg.small
        default:            return cfg.medium ?? cfg.small ?? cfg.large
        }
    }

    @ViewBuilder
    private func placeholderView() -> some View {
        VStack(spacing: 6) {
            Image(systemName: "app.fill").font(.title2).foregroundColor(.white.opacity(0.8))
            Text("No configuration").font(.caption2).foregroundColor(.white.opacity(0.6))
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity).padding()
    }
}
