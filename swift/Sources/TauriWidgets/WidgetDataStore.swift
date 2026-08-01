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

/// Async delivery receipt from the widget extension (outside config nonce space).
public struct WidgetTransportReceipt: Codable {
    public let readFrom: String
    public let nonce: UInt64
    public let ts: UInt64

    public init(readFrom: String, nonce: UInt64, ts: UInt64) {
        self.readFrom = readFrom
        self.nonce = nonce
        self.ts = ts
    }
}

public struct TauriWidgetDataStore {

    /// Load config and plant a delivery receipt (does not bump config nonce).
    public static func loadConfig(appGroup: String, widgetId: String = "default") -> WidgetUIConfig? {
        loadConfigAcknowledging(appGroup: appGroup, widgetId: widgetId)
    }

    public static func loadConfigAcknowledging(appGroup: String, widgetId: String = "default") -> WidgetUIConfig? {
        let (map, source) = loadFreshestMapWithSource(appGroup: appGroup)
        defer {
            let nonce = UInt64(map[TauriWidgetStoreKeys.metaNonce] ?? "0") ?? 0
            let receipt = WidgetTransportReceipt(
                readFrom: source,
                nonce: nonce,
                ts: UInt64(Date().timeIntervalSince1970 * 1000)
            )
            writeReceiptEverywhere(receipt, appGroup: appGroup)
        }
        guard let raw = map[TauriWidgetStoreKeys.configKey(widgetId)],
              let data = raw.data(using: .utf8) else {
            return nil
        }
        do {
            return try JSONDecoder().decode(WidgetUIConfig.self, from: data)
        } catch {
            logger.error("loadConfig decode error: \(error.localizedDescription)")
            return nil
        }
    }

    /// Read a single key from the freshest available transport.
    public static func readValue(forKey key: String, appGroup: String) -> String? {
        let map = loadFreshestMap(appGroup: appGroup)
        return map[key]
    }

    /// Fan-out write of a single key into all writable transports.
    public static func writeValue(_ value: String, forKey key: String, appGroup: String) {
        var map = loadFreshestMap(appGroup: appGroup)
        map[key] = value
        touchMeta(&map)
        fanoutWrite(map, appGroup: appGroup)
    }

    /// Replace pending_actions after merge (fan-out).
    public static func writePendingActions(_ actions: [WidgetActionEnvelope], appGroup: String) {
        guard let data = try? JSONEncoder().encode(actions),
              let str = String(data: data, encoding: .utf8) else { return }
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
        loadFreshestMapWithSource(appGroup: appGroup).0
    }

    /// Same as `loadFreshestMap`, but also returns which transport won.
    public static func loadFreshestMapWithSource(appGroup: String) -> ([String: String], String) {
        var candidates: [(String, [String: String])] = []

        if let m = readOwnContainerMap() {
            candidates.append((TauriWidgetTransportName.container, m))
        }
        if let m = readUserDefaultsMap(appGroup: appGroup) {
            candidates.append((TauriWidgetTransportName.defaults, m))
        }
        if let m = readAppGroupFileMap(appGroup: appGroup) {
            candidates.append((TauriWidgetTransportName.appGroup, m))
        }

        return pickFreshestWithSource(candidates)
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
    private static func readUserDefaultsMap(appGroup: String) -> [String: String]? {
        let plainSuite = appGroup.hasPrefix("group.") ? String(appGroup.dropFirst(6)) : appGroup
        for suite in [appGroup, plainSuite] {
            if let defaults = UserDefaults(suiteName: suite),
               let dict = defaults.dictionary(forKey: "widget_data") as? [String: String] {
                return dict
            }
        }
        return nil
    }

    private static func writeUserDefaultsMap(_ map: [String: String], appGroup: String) {
        let plainSuite = appGroup.hasPrefix("group.") ? String(appGroup.dropFirst(6)) : appGroup
        for suite in [appGroup, plainSuite] {
            if let defaults = UserDefaults(suiteName: suite) {
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
