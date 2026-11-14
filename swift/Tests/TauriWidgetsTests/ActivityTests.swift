import Foundation
import XCTest
@testable import TauriWidgets

/// Live Activity content on the extension side. `TauriActivityContent` itself is iOS-only,
/// so the chain drives the same steps it takes: read `activity-template:{name}` from the
/// App Group, fill it with the request's `data`, then the request's own keys on top.
final class ActivityTests: XCTestCase {
    private var box: StoreSandbox!

    override func setUp() {
        super.setUp()
        box = StoreSandbox("activity")
    }

    override func tearDown() {
        box.restore()
        super.tearDown()
    }

    private func content(_ json: String) -> [String: Any] {
        let c = obj(json)
        guard let name = c["template"] as? String, !name.isEmpty,
              let tpl = TauriWidgetDataStore.readValue(forKey: TauriWidgetStoreKeys.activityTemplatePrefix + name,
                                                       appGroup: box.group).flatMap(TauriWidgetSnapshot.object)
        else { return c }
        return TauriResolve.applyActivityTemplate(c, template: tpl)
    }

    /// Given a stored `activity-template:*`, when a request starts, updates and names an unknown template, then the template fills from `data` and the request's own keys win.
    func testActivityTemplateLifecycle() {
        // 1. Start before the app stored the template: the request is used as is.
        let early = content(#"{"id":"r","title":"Plain","template":"ride","data":{"id":7}}"#)
        XCTAssertEqual(early["title"] as? String, "Plain")
        XCTAssertNotNil(early["template"])

        // 2. The app registers `ride`.
        let tpl = ##"{"title":"Ride {{data.id}}","lockScreen":{"type":"vstack","children":[{"type":"text","content":"ETA {{data.eta}}"},{"type":"text","content":"late","when":{"path":"data.late"}}]},"minimal":{"type":"text","content":"x","when":{"path":"data.none"}},"tint":"#f00"}"##
        TauriWidgetDataStore.writeValue(tpl, forKey: TauriWidgetStoreKeys.activityTemplatePrefix + "ride", appGroup: box.group)

        // 3. Start: an empty title takes the template's, the request's `tint` wins, the
        //    routing keys are dropped, a root `when` that fails drops the whole slot.
        let start = content(##"{"id":"r","title":"","template":"ride","data":{"id":7,"eta":"5 min"},"tint":"#0f0"}"##)
        XCTAssertEqual(start["title"] as? String, "Ride 7")
        XCTAssertEqual(start["tint"] as? String, "#0f0")
        XCTAssertEqual(start["id"] as? String, "r")
        XCTAssertNil(start["template"]); XCTAssertNil(start["data"]); XCTAssertNil(start["minimal"])
        XCTAssertEqual(canon(start["lockScreen"]), #"{"children":[{"content":"ETA 5 min","type":"text"}],"type":"vstack"}"#)

        // 4. Update with new data: the same template re-fills; a set title is kept.
        let update = content(#"{"id":"r","title":"Mine","template":"ride","data":{"id":7,"eta":"9 min","late":true}}"#)
        XCTAssertEqual(update["title"] as? String, "Mine")
        XCTAssertEqual(update["tint"] as? String, "#f00")
        XCTAssertEqual(canon(update["lockScreen"]),
                       #"{"children":[{"content":"ETA 9 min","type":"text"},{"content":"late","type":"text"}],"type":"vstack"}"#)

        // 5. An unknown template name falls back to the raw request.
        XCTAssertEqual(content(#"{"id":"r","title":"T","template":"nope"}"#)["template"] as? String, "nope")
    }

    /// Given the host and extension `ActivityAttributes`, when their Codable shapes are compared, then they match.
    /// ActivityKit pairs the app's `Activity.request` with the extension's
    /// `ActivityConfiguration` by type name and Codable shape: the host plugin's copy
    /// (`ios/Sources/WidgetPlugin.swift`) must declare the same stored properties.
    func testActivityAttributesMatchHost() throws {
        func shape(_ file: String) throws -> String {
            let src = try String(contentsOf: RepoPaths.root().appendingPathComponent(file), encoding: .utf8)
            let start = try XCTUnwrap(src.range(of: "struct TauriActivityAttributes: ActivityAttributes {"), file)
            var depth = 0
            var body = ""
            for ch in src[start.lowerBound...] {
                body.append(ch)
                if ch == "{" { depth += 1 }
                if ch == "}" { depth -= 1; if depth == 0 { break } }
            }
            return body.split(separator: "\n")
                .map { $0.trimmingCharacters(in: .whitespaces) }
                .filter { !$0.hasPrefix("//") && !$0.hasPrefix("@") && !$0.contains("init(") && !$0.isEmpty }
                .map { $0.replacingOccurrences(of: "public ", with: "") }
                .joined(separator: "\n")
        }
        let pkg = try shape("swift/Sources/TauriWidgets/LiveActivity.swift")
        XCTAssertEqual(pkg, try shape("ios/Sources/WidgetPlugin.swift"))
        XCTAssertTrue(pkg.contains("var json: String") && pkg.contains("var id: String") && pkg.contains("var group: String"), pkg)
    }
}
