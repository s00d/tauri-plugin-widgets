import Foundation
import os.log

private let logger = Logger(subsystem: "com.tauri.widgets", category: "DataStore")

/// Storage contract (0.4+):
/// - `config:{widgetId}` — widget UI JSON
/// - `pending_actions` — JSON array of action envelopes
/// - `__meta_nonce__` / `__meta_updated_at__` — freshness for multi-transport pick
/// - Receipts live in a **sibling** file/key (`widget_receipt.json` / `widget_receipt`) —
///   never inside the config map (must not bump nonce).
public enum TauriWidgetStoreKeys {
    public static let configPrefix = "config:"
    public static let pendingActions = "pending_actions"
    public static let metaNonce = "__meta_nonce__"
    public static let metaUpdatedAt = "__meta_updated_at__"
    public static let receiptDefaultsKey = "widget_receipt"

    public static func configKey(_ widgetId: String) -> String {
        configPrefix + widgetId
    }
}

/// Transport names — must match Rust `transport::NAME_*`.
public enum TauriWidgetTransportName {
    public static let appGroup = "appgroup"
    public static let defaults = "defaults"
    public static let container = "container"
}

public struct WidgetActionEnvelope: Codable {
    public let action: String
    public let payload: String?
    public let ts: UInt64
    public let widgetId: String
    public let group: String

    public init(action: String, payload: String? = nil, ts: UInt64, widgetId: String, group: String) {
        self.action = action
        self.payload = payload
        self.ts = ts
        self.widgetId = widgetId
        self.group = group
    }

    enum CodingKeys: String, CodingKey {
        case action, payload, ts, widgetId, group
    }
}

/// Async delivery / render receipt (outside config nonce space).
public struct WidgetTransportReceipt: Codable {
    /// Transport / channel that supplied the config (`appgroup`|`defaults`|`container`|…).
    public let source: String
    /// Legacy alias kept in-memory for older callers; not encoded (Rust serde rejects duplicate fields).
    public let readFrom: String
    public let widgetId: String
    public let group: String
    public let instance: String
    public let nonce: UInt64
    public let size: String?
    public let theme: String?
    public let schema: UInt32
    public let rendered: [String]
    public let skipped: [SkippedElement]
    public let ts: UInt64
    /// Why this paint ran: `reload` | `timeline` | `action` | `added` | `resize` | `snapshot`.
    public let trigger: String?

    public struct SkippedElement: Codable {
        public let type: String
        public let reason: String
        public init(type: String, reason: String) {
            self.type = type
            self.reason = reason
        }
    }

    enum CodingKeys: String, CodingKey {
        case source, readFrom, widgetId, group, instance, nonce, size, theme, schema, rendered, skipped, ts, trigger
    }

    public init(
        source: String,
        widgetId: String,
        group: String,
        instance: String,
        nonce: UInt64,
        size: String? = nil,
        theme: String? = nil,
        schema: UInt32 = 1,
        rendered: [String] = [],
        skipped: [SkippedElement] = [],
        ts: UInt64 = UInt64(Date().timeIntervalSince1970 * 1000),
        trigger: String? = nil
    ) {
        self.source = source
        self.readFrom = source
        self.widgetId = widgetId
        self.group = group
        self.instance = instance
        self.nonce = nonce
        self.size = size
        self.theme = theme
        self.schema = schema
        self.rendered = rendered
        self.skipped = skipped
        self.ts = ts
        self.trigger = trigger
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let src = try c.decodeIfPresent(String.self, forKey: .source)
            ?? c.decode(String.self, forKey: .readFrom)
        self.source = src
        self.readFrom = try c.decodeIfPresent(String.self, forKey: .readFrom) ?? src
        self.widgetId = try c.decode(String.self, forKey: .widgetId)
        self.group = try c.decode(String.self, forKey: .group)
        self.instance = try c.decode(String.self, forKey: .instance)
        self.nonce = try c.decode(UInt64.self, forKey: .nonce)
        self.size = try c.decodeIfPresent(String.self, forKey: .size)
        self.theme = try c.decodeIfPresent(String.self, forKey: .theme)
        self.schema = try c.decodeIfPresent(UInt32.self, forKey: .schema) ?? 1
        self.rendered = try c.decodeIfPresent([String].self, forKey: .rendered) ?? []
        self.skipped = try c.decodeIfPresent([SkippedElement].self, forKey: .skipped) ?? []
        self.ts = try c.decode(UInt64.self, forKey: .ts)
        self.trigger = try c.decodeIfPresent(String.self, forKey: .trigger)
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        // Emit only `source` — Rust Receipt has `read_from` with serde alias `source`.
        try c.encode(source, forKey: .source)
        try c.encode(widgetId, forKey: .widgetId)
        try c.encode(group, forKey: .group)
        try c.encode(instance, forKey: .instance)
        try c.encode(nonce, forKey: .nonce)
        try c.encodeIfPresent(size, forKey: .size)
        try c.encodeIfPresent(theme, forKey: .theme)
        try c.encode(schema, forKey: .schema)
        try c.encode(rendered, forKey: .rendered)
        try c.encode(skipped, forKey: .skipped)
        try c.encode(ts, forKey: .ts)
        try c.encodeIfPresent(trigger, forKey: .trigger)
    }
}

public struct TauriWidgetDataStore {

    /// Load config and plant a delivery receipt (does not bump config nonce).
    public static func loadConfig(appGroup: String, widgetId: String = "default") -> WidgetUIConfig? {
        loadConfigAcknowledging(appGroup: appGroup, widgetId: widgetId)
    }

    public static func loadConfigAcknowledging(appGroup: String, widgetId: String = "default") -> WidgetUIConfig? {
        #if os(iOS)
        guard assertAppGroupAvailable(appGroup) else { return nil }
        #endif
        let (map, source) = loadFreshestMapWithSource(appGroup: appGroup, widgetId: widgetId)
        guard let raw = map[TauriWidgetStoreKeys.configKey(widgetId)],
              let data = raw.data(using: .utf8) else {
            return nil
        }
        let nonce = UInt64(map[TauriWidgetStoreKeys.metaNonce] ?? "0") ?? 0
        let receipt = WidgetTransportReceipt(
            source: source,
            widgetId: widgetId,
            group: appGroup,
            instance: "default",
            nonce: nonce,
            size: nil,
            schema: 1,
            rendered: [],
            skipped: []
        )
        writeReceiptEverywhere(receipt, appGroup: appGroup)
        do {
            return try JSONDecoder().decode(WidgetUIConfig.self, from: data)
        } catch {
            logger.error("loadConfig decode error: \(error.localizedDescription)")
            return nil
        }
    }

    /// Load config + which transport won (for providers that set instance/size themselves).
    public static func loadConfigWithSource(
        appGroup: String,
        widgetId: String = "default"
    ) -> (WidgetUIConfig?, String, UInt64) {
        #if os(iOS)
        guard assertAppGroupAvailable(appGroup) else {
            return (nil, "unavailable", 0)
        }
        #endif
        let (map, source) = loadFreshestMapWithSource(appGroup: appGroup, widgetId: widgetId)
        let nonce = UInt64(map[TauriWidgetStoreKeys.metaNonce] ?? "0") ?? 0
        guard let raw = map[TauriWidgetStoreKeys.configKey(widgetId)],
              let data = raw.data(using: .utf8),
              let cfg = try? JSONDecoder().decode(WidgetUIConfig.self, from: data) else {
            return (nil, source, nonce)
        }
        return (cfg, source, nonce)
    }
    /// iOS: App Group must work — silent fallback looks like an empty widget.
    /// Returns false when the group container is unavailable (callers should stop the load path).
    @discardableResult
    public static func assertAppGroupAvailable(_ appGroup: String) -> Bool {
        #if os(iOS)
        if ProcessInfo.processInfo.environment["WIDGET_APP_GROUP_DATA_FILE"] != nil { return true }
        if FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: appGroup) == nil {
            logger.error(
                "App Group '\(appGroup)' unavailable: enable App Groups on BOTH the app and widget extension targets. Silent fallback disabled on iOS."
            )
            return false
        }
        #endif
        return true
    }

    /// Read a single key from the freshest available transport.
    public static func readValue(forKey key: String, appGroup: String) -> String? {
        let map = loadFreshestMap(appGroup: appGroup)
        return map[key]
    }

    /// Fan-out write of a single key into all writable transports.
    public static func writeValue(_ value: String, forKey key: String, appGroup: String) {
        #if os(iOS)
        guard assertAppGroupAvailable(appGroup) else { return }
        #endif
        var map = loadFreshestMap(appGroup: appGroup)
        map[key] = value
        touchMeta(&map)
        fanoutWrite(map, appGroup: appGroup)
    }

    /// Replace pending_actions after merge (fan-out).
    public static func writePendingActions(_ actions: [WidgetActionEnvelope], appGroup: String) {
        #if os(iOS)
        guard assertAppGroupAvailable(appGroup) else { return }
        #endif
        guard let data = try? JSONEncoder().encode(actions),
              let str = String(data: data, encoding: .utf8) else { return }
        // Retry RMW so concurrent widget taps don't clobber each other.
        for _ in 0..<5 {
            var map = loadFreshestMap(appGroup: appGroup)
            var merged = readPendingActions(appGroup: appGroup)
            // Caller already computed the full desired queue — prefer their list when longer
            // or when the disk queue is empty; otherwise append unique by (action,ts,widgetId).
            if actions.count >= merged.count {
                merged = actions
            } else {
                let existing = Set(merged.map { "\($0.action)|\($0.ts)|\($0.widgetId ?? "")" })
                for a in actions {
                    let k = "\(a.action)|\(a.ts)|\(a.widgetId ?? "")"
                    if !existing.contains(k) { merged.append(a) }
                }
            }
            guard let out = try? JSONEncoder().encode(merged),
                  let outStr = String(data: out, encoding: .utf8) else { return }
            map[TauriWidgetStoreKeys.pendingActions] = outStr
            touchMeta(&map)
            fanoutWrite(map, appGroup: appGroup)
            return
        }
        writeValue(str, forKey: TauriWidgetStoreKeys.pendingActions, appGroup: appGroup)
    }

    public static func readPendingActions(appGroup: String) -> [WidgetActionEnvelope] {
        guard let raw = readValue(forKey: TauriWidgetStoreKeys.pendingActions, appGroup: appGroup),
              let data = raw.data(using: .utf8),
              let arr = try? JSONDecoder().decode([WidgetActionEnvelope].self, from: data) else {
            return []
        }
        return arr
    }

    // MARK: - Multi-transport

    public static func loadFreshestMap(appGroup: String) -> [String: String] {
        loadFreshestMapWithSource(appGroup: appGroup, widgetId: nil).0
    }

    /// Same as `loadFreshestMap`, but also returns which transport won.
    ///
    /// File transports (container / App Group) win over UserDefaults when they
    /// carry the requested `config:{widgetId}` (or any config when widgetId is nil).
    public static func loadFreshestMapWithSource(
        appGroup: String,
        widgetId: String? = nil
    ) -> ([String: String], String) {
        var fileCandidates: [(String, [String: String])] = []
        var defaultsCandidates: [(String, [String: String])] = []

        if let m = readOwnContainerMap() {
            fileCandidates.append((TauriWidgetTransportName.container, m))
        }
        if let m = readAppGroupFileMap(appGroup: appGroup) {
            fileCandidates.append((TauriWidgetTransportName.appGroup, m))
        }
        for map in readAllUserDefaultsMaps(appGroup: appGroup) {
            defaultsCandidates.append((TauriWidgetTransportName.defaults, map))
        }

        let filesBest = pickFreshestWithSource(fileCandidates)
        if mapHasConfig(filesBest.0, widgetId: widgetId) {
            return filesBest
        }
        return pickFreshestWithSource(fileCandidates + defaultsCandidates)
    }

    private static func mapHasConfig(_ map: [String: String], widgetId: String? = nil) -> Bool {
        if let widgetId {
            return map[TauriWidgetStoreKeys.configKey(widgetId)] != nil
        }
        return map.keys.contains { $0.hasPrefix(TauriWidgetStoreKeys.configPrefix) }
    }

    private static func pickFreshestWithSource(
        _ maps: [(String, [String: String])]
    ) -> ([String: String], String) {
        var best: [String: String] = [:]
        var bestSource = TauriWidgetTransportName.container
        var bestNonce: UInt64 = 0
        var bestTs: UInt64 = 0
        for (source, map) in maps {
            let n = UInt64(map[TauriWidgetStoreKeys.metaNonce] ?? "0") ?? 0
            let t = UInt64(map[TauriWidgetStoreKeys.metaUpdatedAt] ?? "0") ?? 0
            if best.isEmpty || n > bestNonce || (n == bestNonce && t > bestTs) {
                best = map
                bestSource = source
                bestNonce = n
                bestTs = t
            }
        }
        return (best, bestSource)
    }

    private static func touchMeta(_ map: inout [String: String]) {
        let next = (UInt64(map[TauriWidgetStoreKeys.metaNonce] ?? "0") ?? 0) &+ 1
        map[TauriWidgetStoreKeys.metaNonce] = String(next)
        map[TauriWidgetStoreKeys.metaUpdatedAt] = String(UInt64(Date().timeIntervalSince1970 * 1000))
    }

    private static func fanoutWrite(_ map: [String: String], appGroup: String) {
        writeOwnContainerMap(map)
        writeUserDefaultsMap(map, appGroup: appGroup)
        writeAppGroupFileMap(map, appGroup: appGroup)
    }

    // MARK: - Receipts (separate namespace — never touch config map / nonce)

    public static func writeReceiptEverywhere(_ receipt: WidgetTransportReceipt, appGroup: String) {
        guard let data = try? JSONEncoder().encode(receipt),
              let json = String(data: data, encoding: .utf8) else { return }

        // Own container file
        let receiptURL = URL(fileURLWithPath: ownContainerReceiptPath())
        try? FileManager.default.createDirectory(
            at: receiptURL.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try? data.write(to: receiptURL, options: .atomic)

        // UserDefaults suite (sibling key, not inside widget_data)
        let plainSuite = appGroup.hasPrefix("group.") ? String(appGroup.dropFirst(6)) : appGroup
        for suite in [appGroup, plainSuite] {
            if let defaults = UserDefaults(suiteName: suite) {
                defaults.set(json, forKey: TauriWidgetStoreKeys.receiptDefaultsKey)
                defaults.synchronize()
            }
        }

        // App Group shared file
        if let dataURL = dataFileURL(appGroup: appGroup) {
            let url = dataURL.deletingLastPathComponent()
                .appendingPathComponent("widget_receipt.json")
            try? FileManager.default.createDirectory(
                at: url.deletingLastPathComponent(),
                withIntermediateDirectories: true
            )
            try? data.write(to: url, options: .atomic)
        }
    }

    /// Own-container / WidgetSandboxFile path.
    ///
    /// Overrides (tests / host parity with Rust):
    /// - `WIDGET_SANDBOX_DATA_FILE` — absolute path to `widget_data.json`
    /// - else `WIDGET_CONTAINER_ROOT` + `WIDGET_EXTENSION_BUNDLE` →
    ///   `{root}/Library/Containers/{bundle}/Data/widget_data.json`
    /// - else extension home: `NSHomeDirectory()/widget_data.json`
    public static func ownContainerDataPath() -> String {
        let env = ProcessInfo.processInfo.environment
        if let p = env["WIDGET_SANDBOX_DATA_FILE"], !p.isEmpty {
            return p
        }
        if let root = env["WIDGET_CONTAINER_ROOT"], !root.isEmpty,
           let bundle = env["WIDGET_EXTENSION_BUNDLE"], !bundle.isEmpty {
            return URL(fileURLWithPath: root)
                .appendingPathComponent("Library/Containers")
                .appendingPathComponent(bundle)
                .appendingPathComponent("Data")
                .appendingPathComponent("widget_data.json")
                .path
        }
        return NSHomeDirectory() + "/widget_data.json"
    }

    public static func ownContainerReceiptPath() -> String {
        URL(fileURLWithPath: ownContainerDataPath())
            .deletingLastPathComponent()
            .appendingPathComponent("widget_receipt.json")
            .path
    }

    // WidgetSandboxFile (macOS ad-hoc / no App Group share)
    private static func readOwnContainerMap() -> [String: String]? {
        let ownFile = ownContainerDataPath()
        guard let data = try? Data(contentsOf: URL(fileURLWithPath: ownFile)),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: String] else {
            return nil
        }
        return json
    }

    private static func writeOwnContainerMap(_ map: [String: String]) {
        let ownFile = ownContainerDataPath()
        let url = URL(fileURLWithPath: ownFile)
        try? FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        guard let out = try? JSONSerialization.data(withJSONObject: map, options: [.prettyPrinted, .sortedKeys]) else {
            return
        }
        try? out.write(to: url, options: .atomic)
    }

    // App Group UserDefaults
    private static func userDefaultsSuites(_ appGroup: String) -> [String] {
        let plain = appGroup.hasPrefix("group.") ? String(appGroup.dropFirst(6)) : appGroup
        return plain == appGroup ? [appGroup] : [appGroup, plain]
    }

    /// Every suite that has a `widget_data` map (both `group.*` and bare id).
    private static func readAllUserDefaultsMaps(appGroup: String) -> [[String: String]] {
        var out: [[String: String]] = []
        for suite in userDefaultsSuites(appGroup) {
            if let defaults = UserDefaults(suiteName: suite),
               let dict = defaults.dictionary(forKey: "widget_data") as? [String: String] {
                out.append(dict)
            }
        }
        return out
    }

    private static func readUserDefaultsMap(appGroup: String) -> [String: String]? {
        let maps = readAllUserDefaultsMaps(appGroup: appGroup)
        guard !maps.isEmpty else { return nil }
        return pickFreshestWithSource(maps.map { (TauriWidgetTransportName.defaults, $0) }).0
    }

    private static func writeUserDefaultsMap(_ map: [String: String], appGroup: String) {
        for suite in userDefaultsSuites(appGroup) {
            if let defaults = UserDefaults(suiteName: suite) {
                // remove+set so CFPreferences notices a change across processes
                defaults.removeObject(forKey: "widget_data")
                defaults.set(map, forKey: "widget_data")
                defaults.synchronize()
            }
        }
    }

    // App Group shared container file
    private static func readAppGroupFileMap(appGroup: String) -> [String: String]? {
        guard let url = dataFileURL(appGroup: appGroup),
              let data = try? Data(contentsOf: url),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: String] else {
            return nil
        }
        return json
    }

    private static func writeAppGroupFileMap(_ map: [String: String], appGroup: String) {
        guard let url = dataFileURL(appGroup: appGroup),
              let out = try? JSONSerialization.data(withJSONObject: map, options: [.prettyPrinted, .sortedKeys]) else {
            return
        }
        try? FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try? out.write(to: url, options: .atomic)
    }

    public static func dataFileURL(appGroup: String) -> URL? {
        let env = ProcessInfo.processInfo.environment
        if let p = env["WIDGET_APP_GROUP_DATA_FILE"], !p.isEmpty {
            return URL(fileURLWithPath: p)
        }
        guard let container = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: appGroup
        ) else { return nil }
        return container.appendingPathComponent("widget_data.json")
    }
}
