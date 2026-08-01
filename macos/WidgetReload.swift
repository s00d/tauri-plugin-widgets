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

@_cdecl("macos_widget_set_defaults")
public func macosWidgetSetDefaults(
    _ groupId: UnsafePointer<CChar>,
    _ key: UnsafePointer<CChar>,
    _ value: UnsafePointer<CChar>
) -> Bool {
    let group = String(cString: groupId)
    let k = String(cString: key)
    let v = String(cString: value)
    guard let defaults = UserDefaults(suiteName: group) else { return false }

    var map = defaults.dictionary(forKey: "widget_data") as? [String: String] ?? [:]
    map[k] = v
    defaults.set(map, forKey: "widget_data")
    defaults.synchronize()
    return true
}

/// Replace the entire `widget_data` map in one write (avoids O(n²) per-key fan-out).
@_cdecl("macos_widget_set_defaults_map")
public func macosWidgetSetDefaultsMap(
    _ groupId: UnsafePointer<CChar>,
    _ jsonMap: UnsafePointer<CChar>
) -> Bool {
    let group = String(cString: groupId)
    let raw = String(cString: jsonMap)
    guard let defaults = UserDefaults(suiteName: group) else { return false }
    guard let data = raw.data(using: .utf8),
          let obj = try? JSONSerialization.jsonObject(with: data) as? [String: String] else {
        return false
    }
    defaults.set(obj, forKey: "widget_data")
    defaults.synchronize()
    return true
}

/// Read the suite `widget_data` map as a JSON object string (caller frees via macos_widget_free_string).
@_cdecl("macos_widget_get_defaults_map")
public func macosWidgetGetDefaultsMap(
    _ groupId: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>? {
    let group = String(cString: groupId)
    guard let defaults = UserDefaults(suiteName: group),
          let dict = defaults.dictionary(forKey: "widget_data") as? [String: String],
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
    guard let defaults = UserDefaults(suiteName: group) else { return false }
    defaults.set(v, forKey: k)
    defaults.synchronize()
    return true
}

/// Read an arbitrary string from the suite (caller frees via macos_widget_free_string).
@_cdecl("macos_widget_get_defaults_string")
public func macosWidgetGetDefaultsString(
    _ groupId: UnsafePointer<CChar>,
    _ key: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>? {
    let group = String(cString: groupId)
    let k = String(cString: key)
    guard let defaults = UserDefaults(suiteName: group),
          let v = defaults.string(forKey: k) else {
        return nil
    }
    return strdup(v)
}
