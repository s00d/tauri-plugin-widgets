import Foundation
#if canImport(WidgetKit)
import WidgetKit
#endif

@_cdecl("macos_widget_reload_all")
public func macosWidgetReloadAll() -> Bool {
    #if canImport(WidgetKit)
    if #available(macOS 11.0, *) {
        WidgetCenter.shared.reloadAllTimelines()
        return true
    }
    #endif
    return false
}

@_cdecl("macos_widget_reload_kind")
public func macosWidgetReloadKind(_ kind: UnsafePointer<CChar>) -> Bool {
    #if canImport(WidgetKit)
    if #available(macOS 11.0, *) {
        WidgetCenter.shared.reloadTimelines(ofKind: String(cString: kind))
        return true
    }
    #endif
    return false
}

@_cdecl("macos_widget_container_path")
public func macosWidgetContainerPath(_ groupId: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>? {
    let group = String(cString: groupId)
    guard let url = FileManager.default.containerURL(
        forSecurityApplicationGroupIdentifier: group
    ) else {
        return nil
    }
    return strdup(url.path)
}

@_cdecl("macos_widget_free_string")
public func macosWidgetFreeString(_ ptr: UnsafeMutablePointer<CChar>?) {
    free(ptr)
}

private func suiteNames(_ group: String) -> [String] {
    if group.hasPrefix("group.") {
        let plain = String(group.dropFirst(6))
        return [group, plain]
    }
    return [group]
}

private func mapNonce(_ map: [String: String]) -> UInt64 {
    UInt64(map["__meta_nonce__"] ?? "0") ?? 0
}

private func mapUpdatedAt(_ map: [String: String]) -> UInt64 {
    UInt64(map["__meta_updated_at__"] ?? "0") ?? 0
}

private func freshestMap(_ maps: [[String: String]]) -> [String: String]? {
    var best: [String: String]?
    var bestNonce: UInt64 = 0
    var bestTs: UInt64 = 0
    for map in maps {
        let n = mapNonce(map)
        let t = mapUpdatedAt(map)
        if best == nil || n > bestNonce || (n == bestNonce && t > bestTs) {
            best = map
            bestNonce = n
            bestTs = t
        }
    }
    return best
}

@_cdecl("macos_widget_set_defaults")
public func macosWidgetSetDefaults(
    _ groupId: UnsafePointer<CChar>,
    _ key: UnsafePointer<CChar>,
    _ value: UnsafePointer<CChar>
) -> Bool {
    let group = String(cString: groupId)
    let k = String(cString: key)
    let v = String(cString: value)
    var ok = false
    for suite in suiteNames(group) {
        guard let defaults = UserDefaults(suiteName: suite) else { continue }
        var map = defaults.dictionary(forKey: "widget_data") as? [String: String] ?? [:]
        map[k] = v
        defaults.removeObject(forKey: "widget_data")
        defaults.set(map, forKey: "widget_data")
        defaults.synchronize()
        ok = true
    }
    return ok
}

/// Replace the entire `widget_data` map in one write (avoids O(n²) per-key fan-out).
/// Writes both `group.*` and bare suite ids (widget may read either).
@_cdecl("macos_widget_set_defaults_map")
public func macosWidgetSetDefaultsMap(
    _ groupId: UnsafePointer<CChar>,
    _ jsonMap: UnsafePointer<CChar>
) -> Bool {
    let group = String(cString: groupId)
    let raw = String(cString: jsonMap)
    guard let data = raw.data(using: .utf8),
          let obj = try? JSONSerialization.jsonObject(with: data) as? [String: String] else {
        return false
    }
    var ok = false
    for suite in suiteNames(group) {
        guard let defaults = UserDefaults(suiteName: suite) else { continue }
        defaults.removeObject(forKey: "widget_data")
        defaults.set(obj, forKey: "widget_data")
        defaults.synchronize()
        ok = true
    }
    return ok
}

/// Read the suite `widget_data` map as a JSON object string (caller frees via macos_widget_free_string).
/// Returns the freshest map across `group.*` and bare suites.
@_cdecl("macos_widget_get_defaults_map")
public func macosWidgetGetDefaultsMap(
    _ groupId: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>? {
    let group = String(cString: groupId)
    var maps: [[String: String]] = []
    for suite in suiteNames(group) {
        if let defaults = UserDefaults(suiteName: suite),
           let dict = defaults.dictionary(forKey: "widget_data") as? [String: String] {
            maps.append(dict)
        }
    }
    guard let dict = freshestMap(maps),
          let data = try? JSONSerialization.data(withJSONObject: dict),
          let json = String(data: data, encoding: .utf8) else {
        return nil
    }
    return strdup(json)
}

/// Set an arbitrary string value in the App Group UserDefaults suite (receipts, etc.).
@_cdecl("macos_widget_set_defaults_string")
public func macosWidgetSetDefaultsString(
    _ groupId: UnsafePointer<CChar>,
    _ key: UnsafePointer<CChar>,
    _ value: UnsafePointer<CChar>
) -> Bool {
    let group = String(cString: groupId)
    let k = String(cString: key)
    let v = String(cString: value)
    var ok = false
    for suite in suiteNames(group) {
        guard let defaults = UserDefaults(suiteName: suite) else { continue }
        defaults.set(v, forKey: k)
        defaults.synchronize()
        ok = true
    }
    return ok
}

/// Read an arbitrary string from the suite (caller frees via macos_widget_free_string).
@_cdecl("macos_widget_get_defaults_string")
public func macosWidgetGetDefaultsString(
    _ groupId: UnsafePointer<CChar>,
    _ key: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>? {
    let group = String(cString: groupId)
    let k = String(cString: key)
    for suite in suiteNames(group) {
        if let defaults = UserDefaults(suiteName: suite),
           let v = defaults.string(forKey: k) {
            return strdup(v)
        }
    }
    return nil
}
