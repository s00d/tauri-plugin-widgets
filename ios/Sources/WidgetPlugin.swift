import Tauri
import UIKit
import WebKit

#if canImport(WidgetKit)
import WidgetKit
#endif

struct SetItemsArgs: Decodable {
    let key: String
    let value: String
    let group: String
}

struct GetItemsArgs: Decodable {
    let key: String
    let group: String
}

struct SetRegisterWidgetArgs: Decodable {
    let widgets: [String]
}

struct ReloadTimelinesArgs: Decodable {
    let ofKind: String
}

class WidgetPlugin: Plugin {
    private var registeredWidgets: [String] = []
    private let safeRegex = try! NSRegularExpression(pattern: "[^A-Za-z0-9._-]", options: [])

    private func sanitize(_ value: String, fallback: String = "_") -> String {
        let range = NSRange(location: 0, length: value.utf16.count)
        let cleaned = safeRegex.stringByReplacingMatches(in: value, options: [], range: range, withTemplate: "_")
        let trimmed = cleaned.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? fallback : trimmed
    }

    /// Derives the App Group ID from the passed group parameter.
    /// If the group already starts with "group.", use it as-is.
    /// Otherwise, derive from the main app's bundle identifier.
    private func resolveAppGroup(_ group: String) -> String {
        let clean = sanitize(group, fallback: "")
        if clean.hasPrefix("group.") {
            return clean
        }
        if let bundleId = Bundle.main.bundleIdentifier {
            return "group.\(sanitize(bundleId, fallback: "app"))"
        }
        return "group.app"
    }

    /// Returns the URL of the shared data file for an App Group.
    private func dataFileURL(group: String) -> URL? {
        let appGroup = resolveAppGroup(group)
        return FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: appGroup
        )?.appendingPathComponent("widget_data.json")
    }

    /// Reads the shared data file as a [String: String] dictionary.
    private func readDataMap(group: String) -> [String: String] {
        guard let url = dataFileURL(group: group),
              let data = try? Data(contentsOf: url),
              let map = try? JSONSerialization.jsonObject(with: data) as? [String: String] else {
            return [:]
        }
        return map
    }

    /// Writes the [String: String] dictionary to the shared data file atomically.
    private func writeDataMap(_ map: [String: String], group: String) -> Bool {
        guard let url = dataFileURL(group: group) else { return false }
        do {
            let data = try JSONSerialization.data(withJSONObject: map, options: [.prettyPrinted, .sortedKeys])
            try data.write(to: url, options: .atomic)
            return true
        } catch {
            return false
        }
    }

    @objc public override func load(webview: WKWebView) {}

    @objc func setItems(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(SetItemsArgs.self)
        let safeKey = sanitize(args.key)
        var map = readDataMap(group: args.group)
        map[safeKey] = args.value
        let ok = writeDataMap(map, group: args.group)
        if ok {
            invoke.resolve(["results": true])
        } else {
            let appGroup = resolveAppGroup(args.group)
            invoke.reject("Failed to write to shared container for group: \(appGroup). Ensure App Groups capability is enabled.")
        }
    }

    @objc func getItems(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(GetItemsArgs.self)
        let safeKey = sanitize(args.key)
        let map = readDataMap(group: args.group)
        if let value = map[safeKey] {
            invoke.resolve(["results": value])
        } else {
            invoke.resolve(["results": NSNull()])
        }
    }

    @objc func setRegisterWidget(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(SetRegisterWidgetArgs.self)
        registeredWidgets = args.widgets
        invoke.resolve(["results": true])
    }

    @objc func reloadAllTimelines(_ invoke: Invoke) throws {
        #if canImport(WidgetKit)
        if #available(iOS 14.0, macOS 11.0, *) {
            WidgetCenter.shared.reloadAllTimelines()
            invoke.resolve(["results": true])
        } else {
            invoke.reject("WidgetKit requires iOS 14.0+ / macOS 11.0+")
        }
        #else
        invoke.reject("WidgetKit is not available on this platform")
        #endif
    }

    @objc func reloadTimelines(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(ReloadTimelinesArgs.self)

        #if canImport(WidgetKit)
        if #available(iOS 14.0, macOS 11.0, *) {
            WidgetCenter.shared.reloadTimelines(ofKind: args.ofKind)
            invoke.resolve(["results": true])
        } else {
            invoke.reject("WidgetKit requires iOS 14.0+ / macOS 11.0+")
        }
        #else
        invoke.reject("WidgetKit is not available on this platform")
        #endif
    }

    @objc func requestWidget(_ invoke: Invoke) throws {
        invoke.reject("requestWidget is not supported on iOS/macOS. Users add widgets manually via the widget gallery.")
    }
}

@_cdecl("init_plugin_widgets")
func initPlugin() -> Plugin {
    return WidgetPlugin()
}
