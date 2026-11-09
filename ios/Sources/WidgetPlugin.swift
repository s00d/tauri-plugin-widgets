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

struct SetWidgetConfigArgs: Decodable {
    let config: String
    let group: String
    let widgetId: String
}

struct GetWidgetConfigArgs: Decodable {
    let group: String
    let widgetId: String
}

struct GroupArgs: Decodable {
    let group: String
}

class WidgetPlugin: Plugin {
    private var registeredWidgets: [String] = []
    private let safeRegex = try! NSRegularExpression(pattern: "[^A-Za-z0-9._:-]", options: [])
    private let groupSafeRegex = try! NSRegularExpression(pattern: "[^A-Za-z0-9._-]", options: [])

    private let configPrefix = "config:"
    private let pendingActionsKey = "pending_actions"
    private let metaNonceKey = "__meta_nonce__"
    private let metaUpdatedAtKey = "__meta_updated_at__"

    private func sanitize(_ value: String, fallback: String = "_") -> String {
        let range = NSRange(location: 0, length: value.utf16.count)
        let cleaned = safeRegex.stringByReplacingMatches(in: value, options: [], range: range, withTemplate: "_")
        let trimmed = cleaned.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? fallback : trimmed
    }

    private func sanitizeGroup(_ value: String, fallback: String = "_") -> String {
        let range = NSRange(location: 0, length: value.utf16.count)
        let cleaned = groupSafeRegex.stringByReplacingMatches(in: value, options: [], range: range, withTemplate: "_")
        let trimmed = cleaned.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? fallback : trimmed
    }

    /// App Group must be explicit `group.*` — no silent substitution.
    private func resolveAppGroup(_ group: String) throws -> String {
        let clean = sanitizeGroup(group, fallback: "")
        guard clean.hasPrefix("group.") else {
            throw NSError(
                domain: "tauri-plugin-widgets",
                code: 1,
                userInfo: [NSLocalizedDescriptionKey: "group must be an App Group id starting with 'group.'"]
            )
        }
        return clean
    }

    private func dataFileURL(group: String) throws -> URL {
        let appGroup = try resolveAppGroup(group)
        guard let url = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: appGroup
        )?.appendingPathComponent("widget_data.json") else {
            throw NSError(
                domain: "tauri-plugin-widgets",
                code: 2,
                userInfo: [NSLocalizedDescriptionKey: "App Group container unavailable for \(appGroup). Enable App Groups."]
            )
        }
        return url
    }

    /// Exclusive `flock` on `widget_data.json.lock`, shared with the widget extension
    /// (`TauriWidgetDataStore.withStoreLock`): every read-modify-write of the map runs
    /// under it, so a widget tap is never wiped by a host write or drained twice.
    private func withStoreLock<T>(group: String, _ body: () throws -> T) throws -> T {
        let url = try dataFileURL(group: group)
        let fd = open(url.path + ".lock", O_RDWR | O_CREAT, 0o644)
        guard fd >= 0 else { return try body() }
        defer { close(fd) }
        flock(fd, LOCK_EX)
        defer { flock(fd, LOCK_UN) }
        return try body()
    }

    private func readDataMap(group: String) throws -> [String: String] {
        let url = try dataFileURL(group: group)
        guard let data = try? Data(contentsOf: url),
              let map = try? JSONSerialization.jsonObject(with: data) as? [String: String] else {
            return [:]
        }
        return map
    }

    private func touchMeta(_ map: inout [String: String]) {
        let next = (UInt64(map[metaNonceKey] ?? "0") ?? 0) &+ 1
        map[metaNonceKey] = String(next)
        map[metaUpdatedAtKey] = String(UInt64(Date().timeIntervalSince1970 * 1000))
    }

    private func writeDataMap(_ map: [String: String], group: String) throws {
        let url = try dataFileURL(group: group)
        let data = try JSONSerialization.data(withJSONObject: map, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: url, options: .atomic)

        // Also mirror into App Group UserDefaults when suite is available.
        let appGroup = try resolveAppGroup(group)
        if let defaults = UserDefaults(suiteName: appGroup) {
            defaults.set(map, forKey: "widget_data")
            defaults.synchronize()
        }
    }

    private func configKey(_ widgetId: String) -> String {
        configPrefix + widgetId
    }

    @objc public override func load(webview: WKWebView) {}

    @objc func setItems(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(SetItemsArgs.self)
        let safeKey = sanitize(args.key)
        try withStoreLock(group: args.group) {
            var map = try readDataMap(group: args.group)
            map[safeKey] = args.value
            touchMeta(&map)
            try writeDataMap(map, group: args.group)
        }
        invoke.resolve(["results": true])
    }

    @objc func getItems(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(GetItemsArgs.self)
        let safeKey = sanitize(args.key)
        let map = try readDataMap(group: args.group)
        if let value = map[safeKey] {
            invoke.resolve(["results": value])
        } else {
            invoke.resolve(["results": NSNull()])
        }
    }

    @objc func setRegisterWidget(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(SetRegisterWidgetArgs.self)
        guard !args.widgets.isEmpty else {
            invoke.reject("widgets must be a non-empty array")
            return
        }
        registeredWidgets = args.widgets
        invoke.resolve(["results": true])
    }

    @objc func setWidgetConfig(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(SetWidgetConfigArgs.self)
        guard !args.widgetId.isEmpty else {
            invoke.reject("widgetId must not be empty")
            return
        }
        try withStoreLock(group: args.group) {
            var map = try readDataMap(group: args.group)
            map[configKey(args.widgetId)] = args.config
            touchMeta(&map)
            try writeDataMap(map, group: args.group)
        }
        invoke.resolve(["results": true])
    }

    @objc func getWidgetConfig(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(GetWidgetConfigArgs.self)
        guard !args.widgetId.isEmpty else {
            invoke.reject("widgetId must not be empty")
            return
        }
        let map = try readDataMap(group: args.group)
        if let value = map[configKey(args.widgetId)] {
            invoke.resolve(["results": value])
        } else {
            invoke.resolve(["results": NSNull()])
        }
    }

    @objc func pollPendingActions(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(GroupArgs.self)
        // Take the whole queue under the store lock: the extension appends under it too.
        let claimed: [[String: Any]] = try withStoreLock(group: args.group) {
            var map = try readDataMap(group: args.group)
            let raw = map[pendingActionsKey] ?? "[]"
            guard let data = raw.data(using: .utf8),
                  let arr = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]],
                  !arr.isEmpty else { return [] }
            map[pendingActionsKey] = "[]"
            touchMeta(&map)
            try writeDataMap(map, group: args.group)
            return arr
        }
        invoke.resolve(["results": claimed])
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
