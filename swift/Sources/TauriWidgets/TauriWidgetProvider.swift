import Foundation
import SwiftUI
import WidgetKit

// MARK: - Timeline Entry

public struct TauriWidgetEntry: TimelineEntry {
    public let date: Date
    public let config: WidgetUIConfig?
    public let family: WidgetFamily
    /// Which widget produced this entry: its actions carry these ids, so several
    /// widgets in one `WidgetBundle` do not report each other's widgetId.
    public let appGroup: String
    public let widgetId: String
    /// Placement id chosen in the Edit Widget sheet (`TauriInstanceProvider`).
    public let instance: String?

    public init(
        date: Date,
        config: WidgetUIConfig?,
        family: WidgetFamily,
        appGroup: String = TauriWidgetsConfig.appGroup,
        widgetId: String = TauriWidgetsConfig.widgetId,
        instance: String? = nil
    ) {
        self.date = date; self.config = config; self.family = family
        self.appGroup = appGroup; self.widgetId = widgetId; self.instance = instance
    }
}

/// Environment for intents built deep in the element tree.
struct TauriWidgetIdentityKey: EnvironmentKey {
    static let defaultValue: (appGroup: String, widgetId: String, instance: String?)? = nil
}

extension EnvironmentValues {
    var tauriWidgetIdentity: (appGroup: String, widgetId: String, instance: String?)? {
        get { self[TauriWidgetIdentityKey.self] }
        set { self[TauriWidgetIdentityKey.self] = newValue }
    }
}

// MARK: - Timeline building (shared by the static and the instance provider)

/// Builds timelines: one entry now plus one per scheduled `entries[].at` (at most
/// `maxEntries`), with `{{…}}` filled, after an optional `source` fetch.
public enum TauriTimeline {
    /// WidgetKit keeps a bounded timeline; later entries arrive on the next reload.
    public static let maxEntries = 30
    /// A widget extension has a few seconds per timeline request.
    public static let fetchTimeout: TimeInterval = 8

    public static func size(_ family: WidgetFamily) -> String {
        switch family {
        case .systemSmall: return "small"
        case .systemLarge, .systemExtraLarge: return "large"
        default: return isAccessory(family) ? "accessory" : "medium"
        }
    }

    /// Lock Screen / watch-style families (iOS only; macOS has none).
    public static func isAccessory(_ family: WidgetFamily) -> Bool {
        #if os(iOS) || os(watchOS)
        return [.accessoryCircular, .accessoryRectangular, .accessoryInline].contains(family)
        #else
        return false
        #endif
    }

    /// Entries for `now` and each upcoming change, resolved with `data`.
    public static func entries(
        raw: String?,
        data: Any?,
        state: Any? = nil,
        now: Date,
        family: WidgetFamily,
        appGroup: String,
        widgetId: String,
        instance: String?
    ) -> [TauriWidgetEntry] {
        let nowMs = UInt64(max(0, now.timeIntervalSince1970 * 1000))
        func entry(_ ms: UInt64) -> TauriWidgetEntry {
            var ctx = TauriResolve.Context.current(instance: instance, data: data, state: state)
            ctx.nowMs = ms
            let cfg = raw.flatMap { TauriResolve.decode($0, ctx) }
            return TauriWidgetEntry(
                date: Date(timeIntervalSince1970: TimeInterval(ms) / 1000),
                config: cfg, family: family, appGroup: appGroup, widgetId: widgetId, instance: instance
            )
        }
        var out = [entry(nowMs)]
        if let raw, let obj = TauriWidgetSnapshot.object(raw) {
            for at in TauriResolve.upcomingChanges(obj, nowMs: nowMs, limit: maxEntries - 1) {
                out.append(entry(at))
            }
        }
        return out
    }

    /// When WidgetKit asks for the next timeline: after `minutes`, or at the last scheduled
    /// entry when that comes first, so a schedule longer than `maxEntries` (or than the
    /// pushed window) keeps going from the stored config instead of freezing on its last entry.
    public static func refreshDate(now: Date, minutes: Int, entries: [TauriWidgetEntry]) -> Date {
        let regular = Calendar.current.date(byAdding: .minute, value: max(1, minutes), to: now) ?? now
        guard entries.count > 1, let last = entries.last?.date else { return regular }
        return min(regular, last)
    }

    /// Fetch the config's `source` when the cached response is older than its refresh
    /// interval; calls back with the freshest data available (cache on failure).
    public static func withFetched(
        raw: String?,
        appGroup: String,
        widgetId: String,
        instance: String?,
        session: URLSession = .shared,
        completion: @escaping (Any?) -> Void
    ) {
        let scoped = TauriWidgetStoreKeys.scopedId(widgetId, instance)
        let cached = TauriFetchedCache.read(appGroup: appGroup, scopedId: scoped)
        let cachedJson = TauriWidgetSnapshot.json(cached?.raw)
        guard let raw, let obj = TauriWidgetSnapshot.object(raw),
              let req = TauriResolve.sourceRequest(obj, .current(instance: instance)),
              let url = URL(string: req.url) else {
            completion(cachedJson)
            return
        }
        let nowMs = UInt64(Date().timeIntervalSince1970 * 1000)
        if let cached, nowMs &- cached.ts < UInt64(req.refreshMinutes) * 60_000 {
            completion(cachedJson)
            return
        }
        var request = URLRequest(url: url, timeoutInterval: fetchTimeout)
        request.httpMethod = req.method
        for (k, v) in req.headers { request.setValue(v, forHTTPHeaderField: k) }
        if let body = req.body { request.httpBody = body.data(using: .utf8) }
        session.dataTask(with: request) { data, response, _ in
            let ok = (response as? HTTPURLResponse).map { (200..<300).contains($0.statusCode) } ?? false
            guard ok, let data, data.count <= TauriResolve.maxSourceBytes,
                  let text = String(data: data, encoding: .utf8),
                  let json = TauriWidgetSnapshot.json(text) else {
                completion(cachedJson)
                return
            }
            TauriFetchedCache.write(appGroup: appGroup, scopedId: scoped, raw: text, ts: nowMs)
            completion(json)
        }.resume()
    }

    /// Load, fetch, resolve; refresh after `refreshMinutes` (or the source interval).
    public static func timeline(
        appGroup: String,
        widgetId: String,
        instance: String?,
        family: WidgetFamily,
        refreshMinutes: Int,
        trigger: String,
        completion: @escaping (Timeline<TauriWidgetEntry>) -> Void
    ) {
        let snap = TauriWidgetDataStore.loadSnapshot(appGroup: appGroup, widgetId: widgetId, instance: instance)
        withFetched(raw: snap.rawConfig, appGroup: appGroup, widgetId: widgetId, instance: instance) { fetched in
            let list = entries(
                raw: snap.rawConfig, data: snap.data(fetched: fetched), state: snap.state, now: Date(), family: family,
                appGroup: appGroup, widgetId: widgetId, instance: instance
            )
            writeReceipt(snap: snap, config: list.first?.config, widgetId: widgetId, appGroup: appGroup,
                         instance: instance, family: family, trigger: trigger)
            var minutes = refreshMinutes
            if let raw = snap.rawConfig, let obj = TauriWidgetSnapshot.object(raw),
               let req = TauriResolve.sourceRequest(obj, .current(instance: instance)) {
                minutes = min(minutes, req.refreshMinutes)
            }
            completion(Timeline(entries: list, policy: .after(refreshDate(now: Date(), minutes: minutes, entries: list))))
        }
    }

    static func writeReceipt(
        snap: TauriWidgetSnapshot, config: WidgetUIConfig?, widgetId: String, appGroup: String,
        instance: String?, family: WidgetFamily, trigger: String
    ) {
        var rendered: [String] = []
        func collect(_ el: WidgetElement) {
            rendered.append(el.type)
            el.children?.forEach(collect)
        }
        if let el = config?.small ?? config?.medium ?? config?.large { collect(el) }
        let receipt = WidgetTransportReceipt(
            source: snap.source,
            widgetId: widgetId,
            group: appGroup,
            instance: instance ?? "",
            nonce: snap.nonce,
            size: size(family),
            schema: 1,
            rendered: rendered,
            skipped: [],
            trigger: trigger
        )
        TauriWidgetDataStore.writeReceiptEverywhere(receipt, appGroup: appGroup)
    }

    /// Widget gallery: the stored config with the cached `source` response, no network
    /// (the gallery wants an answer at once). A fresh install with no config shows the placeholder.
    public static func preview(appGroup: String, widgetId: String, instance: String?, family: WidgetFamily) -> TauriWidgetEntry? {
        let snap = TauriWidgetDataStore.loadSnapshot(appGroup: appGroup, widgetId: widgetId, instance: instance)
        guard snap.rawConfig != nil else { return nil }
        let cached = TauriFetchedCache.read(appGroup: appGroup, scopedId: TauriWidgetStoreKeys.scopedId(widgetId, instance))
        return entries(
            raw: snap.rawConfig, data: snap.data(fetched: TauriWidgetSnapshot.json(cached?.raw)), state: snap.state,
            now: Date(), family: family, appGroup: appGroup, widgetId: widgetId, instance: instance
        ).first
    }
}

// MARK: - Timeline Provider

public struct TauriWidgetProvider: TimelineProvider {
    public let appGroup: String
    public let widgetId: String
    /// Timeline refresh interval in minutes (default 15 to respect WidgetKit budget).
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

    public func getSnapshot(in context: Context, completion: @escaping (TauriWidgetEntry) -> Void) {
        if context.isPreview {
            completion(TauriTimeline.preview(appGroup: appGroup, widgetId: widgetId, instance: nil, family: context.family)
                ?? placeholder(in: context))
            return
        }
        TauriTimeline.timeline(
            appGroup: appGroup, widgetId: widgetId, instance: nil, family: context.family,
            refreshMinutes: refreshMinutes, trigger: "snapshot"
        ) { completion($0.entries.first ?? placeholder(in: context)) }
    }

    public func getTimeline(in context: Context, completion: @escaping (Timeline<TauriWidgetEntry>) -> Void) {
        TauriTimeline.timeline(
            appGroup: appGroup, widgetId: widgetId, instance: nil, family: context.family,
            refreshMinutes: refreshMinutes, trigger: "timeline", completion: completion
        )
    }
}

// MARK: - Widget View

public struct TauriWidgetView: View {
    public var entry: TauriWidgetEntry
    @Environment(\.widgetRenderingMode) private var renderingMode

    public init(entry: TauriWidgetEntry) { self.entry = entry }

    public var body: some View {
        if let el = layoutForFamily() {
            DynamicElementView(element: el, isWidgetRoot: true)
                .environment(\.tauriWidgetIdentity, (entry.appGroup, entry.widgetId, entry.instance))
                .modifier(TransitionMod(kind: entry.config?.transition, swapId: entry.date))
                // Pin to top — default center left a fake “top padding” when content is shorter than the canvas.
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                .containerBackground(for: .widget) {
                    // Lock Screen / tinted / StandBy-vibrant: the system draws the backdrop;
                    // a full-color fill would be flattened to a grey slab.
                    if renderingMode == .fullColor && !TauriTimeline.isAccessory(entry.family) {
                        WidgetChrome.background(for: el)
                    } else {
                        Color.clear
                    }
                }
                .widgetURL(TauriWidgetView.widgetURL(el, family: entry.family))
        } else {
            placeholderView()
                .containerBackground(for: .widget) {
                    LinearGradient(colors: [Color.indigo, Color.purple],
                                   startPoint: .topLeading, endPoint: .bottomTrailing)
                }
        }
    }

    /// Tap target for the whole widget: the root's `url`; small and accessory families
    /// ignore `Link`, so there the first link / button `url` in the tree opens instead.
    static func widgetURL(_ el: WidgetElement, family: WidgetFamily) -> URL? {
        if let u = el.url.flatMap(URL.init(string:)) { return u }
        guard family != .systemMedium, family != .systemLarge, family != .systemExtraLarge else { return nil }
        func first(_ e: WidgetElement) -> URL? {
            if e.type == "link" || e.type == "button", let u = e.url.flatMap(URL.init(string:)) { return u }
            return e.children?.lazy.compactMap(first).first
        }
        return first(el)
    }

    private func layoutForFamily() -> WidgetElement? {
        guard let cfg = entry.config else { return nil }
        switch entry.family {
        case .systemSmall:  return cfg.small ?? cfg.medium ?? cfg.large
        case .systemMedium: return cfg.medium ?? cfg.large ?? cfg.small
        case .systemLarge:  return cfg.large ?? cfg.medium ?? cfg.small
        default:
            if TauriTimeline.isAccessory(entry.family) { return cfg.accessory ?? cfg.small ?? cfg.medium }
            return cfg.medium ?? cfg.small ?? cfg.large
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

// MARK: - Widget push (iOS 26 / macOS 26)

#if compiler(>=6.2)
/// Records the widget push token (hex) in the App Group (`tauri_widget_push_token`); the
/// host's `getWidgetPushToken` returns it for your server, which then reloads the widget
/// with an APNs `widgets` push. Attach with `.pushHandler(TauriWidgetPushHandler.self)`
/// (see the init-ios template) and enable the Push Notifications capability.
@available(iOS 26.0, macOS 26.0, *)
public struct TauriWidgetPushHandler: WidgetPushHandler {
    public static let tokenKey = "tauri_widget_push_token"

    public init() {}

    public func pushTokenDidChange(_ pushInfo: WidgetPushInfo, widgets: [WidgetInfo]) {
        let group = TauriWidgetsConfig.resolvedAppGroup
        guard !group.isEmpty else { return }
        UserDefaults(suiteName: group)?.set(pushInfo.token.map { String(format: "%02x", $0) }.joined(), forKey: Self.tokenKey)
    }
}
#endif
