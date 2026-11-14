import Foundation
@testable import TauriWidgets

struct MacosPoints: Equatable {
    var width: CGFloat
    var height: CGFloat

    var cgSize: CGSize { CGSize(width: width, height: height) }

    /// Default Mac WidgetKit canvas (Notification Center / desktop), not iPhone points.
    static func defaults(for size: String) -> MacosPoints {
        switch size {
        case "medium": return MacosPoints(width: 338, height: 158)
        case "large": return MacosPoints(width: 338, height: 354)
        default: return MacosPoints(width: 158, height: 158)
        }
    }
}

struct VisualCase: Equatable {
    let name: String
    let fixture: String
    let size: String
    let theme: String
    let locale: String
    /// iOS / ImageRenderer canvas (iPhone-ish).
    let points: CGSize
    /// Native macOS WidgetKit canvas — separate from iOS points.
    let macosPoints: MacosPoints
    /// Minimum pixel similarity; lower only for cases whose content depends on the clock.
    let precision: Double

    func config(fixturesRoot: URL) throws -> WidgetUIConfig {
        let url = fixturesRoot.appendingPathComponent("\(fixture).json")
        let data = try Data(contentsOf: url)
        return try JSONDecoder().decode(WidgetUIConfig.self, from: data)
    }

    func layout(from cfg: WidgetUIConfig) -> WidgetElement? {
        switch size {
        case "large": return cfg.large
        case "medium": return cfg.medium
        default: return cfg.small
        }
    }

    static func loadAll(casesRoot: URL, filter: String? = nil) throws -> [VisualCase] {
        let files = try FileManager.default.contentsOfDirectory(
            at: casesRoot,
            includingPropertiesForKeys: nil
        )
        .filter { $0.pathExtension == "json" }
        .sorted { $0.lastPathComponent < $1.lastPathComponent }

        var out: [VisualCase] = []
        for url in files {
            let name = url.deletingPathExtension().lastPathComponent
            if let filter, filter != name { continue }
            let obj = try JSONSerialization.jsonObject(with: Data(contentsOf: url)) as? [String: Any]
            guard let obj,
                  let fixture = obj["fixture"] as? String,
                  let size = obj["size"] as? String
            else {
                throw NSError(
                    domain: "TauriWidgetsTests",
                    code: 10,
                    userInfo: [NSLocalizedDescriptionKey: "bad case \(url.lastPathComponent)"],
                )
            }

            let iosPoints: CGSize = {
                switch size {
                case "medium": return CGSize(width: 329, height: 155)
                case "large": return CGSize(width: 329, height: 345)
                default: return CGSize(width: 155, height: 155)
                }
            }()

            let macos: MacosPoints = {
                if let m = obj["macos"] as? [String: Any],
                   let w = m["width"] as? Double,
                   let h = m["height"] as? Double {
                    return MacosPoints(width: CGFloat(w), height: CGFloat(h))
                }
                if let m = obj["macos"] as? [String: Any],
                   let w = m["width"] as? Int,
                   let h = m["height"] as? Int {
                    return MacosPoints(width: CGFloat(w), height: CGFloat(h))
                }
                return MacosPoints.defaults(for: size)
            }()

            out.append(
                VisualCase(
                    name: name,
                    fixture: fixture,
                    size: size,
                    theme: (obj["theme"] as? String) ?? "dark",
                    locale: (obj["locale"] as? String) ?? "en_US",
                    points: iosPoints,
                    macosPoints: macos,
                    precision: (obj["precision"] as? Double) ?? 0.98,
                )
            )
        }
        return out
    }
}

enum RepoPaths {
    static func root() -> URL {
        URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent() // TauriWidgetsTests
            .deletingLastPathComponent() // Tests
            .deletingLastPathComponent() // swift
            .deletingLastPathComponent() // repo
    }

    static var cases: URL { root().appendingPathComponent("tests/cases") }
    static var fixtures: URL { root().appendingPathComponent("tests/fixtures") }
    static var goldenIos: URL { root().appendingPathComponent("tests/golden/ios") }
    static var goldenMacos: URL { root().appendingPathComponent("tests/golden/macos") }
    static var outIos: URL { root().appendingPathComponent("out/ios") }
    static var outMacos: URL { root().appendingPathComponent("out/macos") }
}

// MARK: - Shared helpers for the store / resolve chains

func obj(_ json: String) -> [String: Any] {
    (try! JSONSerialization.jsonObject(with: Data(json.utf8))) as! [String: Any]
}

/// Canonical JSON (NSNumber / Bool / key order differences do not matter); nil and NSNull → "null".
func canon(_ v: Any?) -> String {
    guard let v, !(v is NSNull) else { return "null" }
    let d = try! JSONSerialization.data(withJSONObject: v, options: [.sortedKeys, .fragmentsAllowed])
    return String(decoding: d, as: UTF8.self)
}

/// Private App Group for one test: a unique group id, temp store files via the env
/// overrides, all restored / removed by `restore()`.
final class StoreSandbox {
    let root: URL
    let group: String
    private var saved: [String: String?] = [:]

    /// `rustLayout`: own-container file at `{root}/Library/Containers/{bundle}/Data`
    /// (the layout `tests/transports.rs` writes), else a plain `own/` file.
    init(_ name: String, rustLayout: Bool = false) {
        root = FileManager.default.temporaryDirectory.appendingPathComponent("tauri-widgets-\(name)-\(UUID().uuidString)")
        group = "group.test.\(name).\(UUID().uuidString)"
        try? FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
        set("WIDGET_APP_GROUP_DATA_FILE", root.appendingPathComponent("appgroup/widget_data.json").path)
        if rustLayout {
            set("WIDGET_SANDBOX_DATA_FILE", nil)
            set("WIDGET_CONTAINER_ROOT", root.path)
            set("WIDGET_EXTENSION_BUNDLE", "test.app.widgetkit")
        } else {
            set("WIDGET_SANDBOX_DATA_FILE", root.appendingPathComponent("own/widget_data.json").path)
        }
    }

    var appGroupFile: URL { URL(fileURLWithPath: ProcessInfo.processInfo.environment["WIDGET_APP_GROUP_DATA_FILE"]!) }
    var ownFile: URL { URL(fileURLWithPath: TauriWidgetDataStore.ownContainerDataPath()) }

    func writeMap(_ map: [String: String], to url: URL) throws {
        try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        try JSONSerialization.data(withJSONObject: map, options: [.prettyPrinted]).write(to: url, options: .atomic)
    }

    func restore() {
        for (k, v) in saved {
            if let v { setenv(k, v, 1) } else { unsetenv(k) }
        }
        saved.removeAll()
        for suite in [group, String(group.dropFirst(6))] {
            UserDefaults.standard.removePersistentDomain(forName: suite)
        }
        try? FileManager.default.removeItem(at: root)
    }

    private func set(_ key: String, _ value: String?) {
        if saved[key] == nil { saved[key] = .some(getenv(key).map { String(cString: $0) }) }
        if let value { setenv(key, value, 1) } else { unsetenv(key) }
    }
}
