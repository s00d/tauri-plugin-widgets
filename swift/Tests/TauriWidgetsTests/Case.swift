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
