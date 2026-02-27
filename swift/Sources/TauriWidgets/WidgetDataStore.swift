import Foundation
import os.log

private let logger = Logger(subsystem: "com.tauri.widgets", category: "DataStore")

public struct TauriWidgetDataStore {

    public static func loadConfig(appGroup: String) -> WidgetUIConfig? {
        guard let raw = readValue(forKey: "__widget_config__", appGroup: appGroup) else {
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

    public static func readValue(forKey key: String, appGroup: String) -> String? {
        // File in widget's own container (written by non-sandboxed main app on macOS)
        let ownFile = NSHomeDirectory() + "/widget_data.json"
        if let data = try? Data(contentsOf: URL(fileURLWithPath: ownFile)),
           let json = try? JSONSerialization.jsonObject(with: data) as? [String: String],
           let val = json[key] {
            return val
        }

        // UserDefaults (works with proper App Group provisioning)
        let plainSuite = appGroup.hasPrefix("group.") ? String(appGroup.dropFirst(6)) : appGroup
        for suite in [plainSuite, appGroup] {
            if let defaults = UserDefaults(suiteName: suite),
               let dict = defaults.dictionary(forKey: "widget_data") as? [String: String],
               let val = dict[key] {
                return val
            }
        }

        // File in App Group shared container
        if let url = dataFileURL(appGroup: appGroup),
           let data = try? Data(contentsOf: url),
           let json = try? JSONSerialization.jsonObject(with: data) as? [String: String],
           let val = json[key] {
            return val
        }

        return nil
    }

    public static func writeValue(_ value: String, forKey key: String, appGroup: String) {
        guard let url = dataFileURL(appGroup: appGroup) else { return }
        var map: [String: String] = [:]
        if let data = try? Data(contentsOf: url),
           let existing = try? JSONSerialization.jsonObject(with: data) as? [String: String] {
            map = existing
        }
        map[key] = value
        if let out = try? JSONSerialization.data(withJSONObject: map, options: [.prettyPrinted, .sortedKeys]) {
            try? out.write(to: url, options: .atomic)
        }
    }

    public static func dataFileURL(appGroup: String) -> URL? {
        guard let container = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: appGroup
        ) else { return nil }
        return container.appendingPathComponent("widget_data.json")
    }
}
