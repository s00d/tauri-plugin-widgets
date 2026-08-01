import Foundation
import os.log

private let logger = Logger(subsystem: "com.tauri.widgets", category: "DataStore")

/// Storage contract (0.4+):
/// - `config:{widgetId}` — widget UI JSON
/// - `pending_actions` — JSON array of action envelopes
/// - `__meta_nonce__` / `__meta_updated_at__` — freshness for multi-transport pick
public enum TauriWidgetStoreKeys {
    public static let configPrefix = "config:"
    public static let pendingActions = "pending_actions"
    public static let metaNonce = "__meta_nonce__"
    public static let metaUpdatedAt = "__meta_updated_at__"

    public static func configKey(_ widgetId: String) -> String {
        configPrefix + widgetId
    }
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

public struct TauriWidgetDataStore {

    public static func loadConfig(appGroup: String, widgetId: String = "default") -> WidgetUIConfig? {
        guard let raw = readValue(forKey: TauriWidgetStoreKeys.configKey(widgetId), appGroup: appGroup) else {
            return nil
        }
        guard let data = raw.data(using: .utf8) else { return nil }
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
        var candidates: [[String: String]] = []

        if let m = readOwnContainerMap() { candidates.append(m) }
        if let m = readUserDefaultsMap(appGroup: appGroup) { candidates.append(m) }
        if let m = readAppGroupFileMap(appGroup: appGroup) { candidates.append(m) }

        return pickFreshest(candidates)
    }

    private static func pickFreshest(_ maps: [[String: String]]) -> [String: String] {
        var best: [String: String] = [:]
        var bestNonce: UInt64 = 0
        var bestTs: UInt64 = 0
        for map in maps {
            let n = UInt64(map[TauriWidgetStoreKeys.metaNonce] ?? "0") ?? 0
            let t = UInt64(map[TauriWidgetStoreKeys.metaUpdatedAt] ?? "0") ?? 0
            if best.isEmpty || n > bestNonce || (n == bestNonce && t > bestTs) {
                best = map
                bestNonce = n
                bestTs = t
            }
        }
        return best
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

    // WidgetSandboxFile (macOS ad-hoc / no App Group share)
    private static func readOwnContainerMap() -> [String: String]? {
        let ownFile = NSHomeDirectory() + "/widget_data.json"
        guard let data = try? Data(contentsOf: URL(fileURLWithPath: ownFile)),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: String] else {
            return nil
        }
        return json
    }

    private static func writeOwnContainerMap(_ map: [String: String]) {
        let ownFile = NSHomeDirectory() + "/widget_data.json"
        guard let out = try? JSONSerialization.data(withJSONObject: map, options: [.prettyPrinted, .sortedKeys]) else {
            return
        }
        try? out.write(to: URL(fileURLWithPath: ownFile), options: .atomic)
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
        try? out.write(to: url, options: .atomic)
    }

    public static func dataFileURL(appGroup: String) -> URL? {
        guard let container = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: appGroup
        ) else { return nil }
        return container.appendingPathComponent("widget_data.json")
    }
}
