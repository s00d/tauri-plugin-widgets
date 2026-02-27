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

    public init(appGroup: String) {
        self.appGroup = appGroup
        TauriWidgetsConfig.appGroup = appGroup
    }

    public func placeholder(in context: Context) -> TauriWidgetEntry {
        TauriWidgetEntry(date: Date(), config: nil, family: context.family)
    }

    public func getSnapshot(in context: Context, completion: @escaping (TauriWidgetEntry) -> Void) {
        let cfg = context.isPreview ? nil : TauriWidgetDataStore.loadConfig(appGroup: appGroup)
        completion(TauriWidgetEntry(date: Date(), config: cfg, family: context.family))
    }

    public func getTimeline(in context: Context, completion: @escaping (Timeline<TauriWidgetEntry>) -> Void) {
        let cfg = TauriWidgetDataStore.loadConfig(appGroup: appGroup)
        let entry = TauriWidgetEntry(date: Date(), config: cfg, family: context.family)
        let nextUpdate = Calendar.current.date(byAdding: .minute, value: 5, to: Date()) ?? Date()
        completion(Timeline(entries: [entry], policy: .after(nextUpdate)))
    }
}

// MARK: - Widget View

public struct TauriWidgetView: View {
    public var entry: TauriWidgetEntry

    public init(entry: TauriWidgetEntry) { self.entry = entry }

    public var body: some View {
        if let el = layoutForFamily() {
            DynamicElementView(element: el)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .containerBackground(for: .widget) { backgroundView(for: el) }
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
    private func backgroundView(for el: WidgetElement) -> some View {
        switch el.background {
        case .solid(let hex):
            if let sem = Color.semantic(hex) { sem } else { Color(hex: hex) }
        case .gradient(let g):
            let colors = g.colors.map { Color(hex: $0) }
            let (s, e): (UnitPoint, UnitPoint) = {
                switch g.direction {
                case "bottomToTop": return (.bottom, .top)
                case "leadingToTrailing": return (.leading, .trailing)
                case "trailingToLeading": return (.trailing, .leading)
                case "topLeadingToBottomTrailing": return (.topLeading, .bottomTrailing)
                case "topTrailingToBottomLeading": return (.topTrailing, .bottomLeading)
                default: return (.top, .bottom)
                }
            }()
            LinearGradient(colors: colors, startPoint: s, endPoint: e)
        case .adaptive(let light, let dark):
            Color.adaptive(light: light, dark: dark)
        case nil:
            #if os(macOS)
            Color(.windowBackgroundColor)
            #else
            Color(.systemBackground)
            #endif
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
