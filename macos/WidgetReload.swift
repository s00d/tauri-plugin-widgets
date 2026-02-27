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
