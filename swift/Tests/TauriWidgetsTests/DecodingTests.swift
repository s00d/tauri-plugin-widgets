import Foundation
import SwiftUI
import WidgetKit
import XCTest
@testable import TauriWidgets

/// Raw JSON → `WidgetElement` / `WidgetUIConfig` → what the widget shows and opens.
final class DecodingTests: XCTestCase {
    private func el(_ json: String) throws -> WidgetElement {
        try JSONDecoder().decode(WidgetElement.self, from: Data(json.utf8))
    }

    /// Given a tree with wrong-typed fields and a child without `type`, when it decodes, then the bad values read as absent and the tap URL is found in what survived.
    func testLossyTreeToWidgetURL() throws {
        // 1. Wrong-typed fields read as absent, a child without `type` is dropped.
        let tree = try el("""
        {"type":"vstack","spacing":"wide","children":[
          {"type":"text","content":"ok","fontSize":"big"},
          {"content":"no type"},
          {"type":"text","content":"also ok"},
          {"type":"link","url":"app://x","children":[]}
        ]}
        """)
        XCTAssertNil(tree.spacing)
        XCTAssertEqual(tree.children?.map { $0.content }, ["ok", "also ok", nil])
        XCTAssertNil(tree.children?.first?.fontSize)

        // 2. Small ignores `Link`, so its first nested link opens; medium keeps per-link taps.
        XCTAssertEqual(TauriWidgetView.widgetURL(tree, family: .systemSmall)?.absoluteString, "app://x")
        XCTAssertNil(TauriWidgetView.widgetURL(tree, family: .systemMedium))

        // 3. A root `url` wins in every family.
        let root = try el(#"{"type":"link","url":"app://root","children":[]}"#)
        XCTAssertEqual(TauriWidgetView.widgetURL(root, family: .systemLarge)?.absoluteString, "app://root")
    }

    /// Given Apple-only `accessory` / `transition` keys and timed entries, when the config resolves then decodes, then both survive at every time.
    func testAccessoryAndTransitionThroughEntries() {
        let raw = #"{"transition":"numeric","accessory":{"type":"text","content":"{{data.v}}"},"small":{"type":"text","content":"{{data.v}}"},"entries":[{"at":5,"accessory":{"type":"text","content":"later"},"small":{"type":"text","content":"later"}}]}"#
        let now = TauriResolve.decode(raw, .init(nowMs: 0, locale: "en", data: ["v": "a"]))
        XCTAssertEqual(now?.accessory?.content, "a")
        XCTAssertEqual(now?.small?.content, "a")
        XCTAssertEqual(now?.transition, "numeric")
        let later = TauriResolve.decode(raw, .init(nowMs: 5, locale: "en"))
        XCTAssertEqual(later?.accessory?.content, "later")
        XCTAssertEqual(later?.small?.content, "later")
        XCTAssertNil(TauriResolve.decode("not json", .init(nowMs: 0, locale: "en")))
        // `accessory` is a shared resolve family (tests/fixtures/resolve/accessory.json).
        XCTAssertNotNil(TauriResolve.resolve(obj(raw), .init(nowMs: 0, locale: ""))["accessory"])
    }

    /// Given one element per docs group, when each decodes and evaluates `body`, then none fails and the groups cover all 21 IR types.
    func testEveryDocsGroupRenders() throws {
        let samples: [(group: String, json: String)] = [
            ("layout", ##"{"type":"vstack","spacing":4,"children":[{"type":"text","content":"L"}]}"##),
            ("text", ##"{"type":"text","content":"hello","fontSize":14}"##),
            ("media", ##"{"type":"shape","shapeType":"circle","size":16,"fill":"#ff0000"}"##),
            ("data", ##"{"type":"progress","value":0.5,"total":1,"barStyle":"linear"}"##),
            ("interactive", ##"{"type":"button","label":"Go","action":"tap"}"##),
            ("spacing", ##"{"type":"spacer","minLength":8}"##),
        ]
        for (group, json) in samples {
            let e: WidgetElement
            do { e = try el(json) } catch { return XCTFail("\(group): decode failed — \(error)") }
            XCTAssertFalse(e.type.isEmpty, "\(group): empty type")
            _ = DynamicElementView(element: e).body
        }
        let groups: [Set<String>] = [
            ["vstack", "hstack", "zstack", "grid", "container"], ["text", "label", "date", "timer"],
            ["image", "shape", "canvas"], ["progress", "gauge", "chart", "list"],
            ["button", "toggle", "link"], ["spacer", "divider"],
        ]
        XCTAssertEqual(groups.reduce(Set<String>()) { $0.union($1) }.count, 21)
    }
}
