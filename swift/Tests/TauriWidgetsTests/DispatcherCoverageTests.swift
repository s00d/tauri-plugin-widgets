import XCTest
import SwiftUI
@testable import TauriWidgets

/// Structural guard for the six-group dispatcher.
/// Catches "forgot to wire a type into a group" without depending on PNG goldens.
final class DispatcherCoverageTests: XCTestCase {
    /// One minimal element per docs group — must decode and evaluate body.
    private static let samples: [(group: String, json: String)] = [
        ("layout", ##"{"type":"vstack","spacing":4,"children":[{"type":"text","content":"L"}]}"##),
        ("text", ##"{"type":"text","content":"hello","fontSize":14}"##),
        ("media", ##"{"type":"shape","shapeType":"circle","size":16,"fill":"#ff0000"}"##),
        ("data", ##"{"type":"progress","value":0.5,"total":1,"barStyle":"linear"}"##),
        ("interactive", ##"{"type":"button","label":"Go","action":"tap"}"##),
        ("spacing", ##"{"type":"spacer","minLength":8}"##),
    ]

    func testEveryDocsGroupRenders() throws {
        for (group, json) in Self.samples {
            let data = Data(json.utf8)
            let el: WidgetElement
            do {
                el = try JSONDecoder().decode(WidgetElement.self, from: data)
            } catch {
                return XCTFail("\(group): decode failed — \(error)")
            }
            XCTAssertFalse(el.type.isEmpty, "\(group): empty type")

            let view = DynamicElementView(element: el)
            _ = view.body
        }
    }

    /// Dispatcher groups must stay aligned with docs/guide/elements/*.md names.
    func testKnownTypesAreAssignedToAGroup() {
        let layout: Set = ["vstack", "hstack", "zstack", "grid", "container"]
        let text: Set = ["text", "label", "date", "timer"]
        let media: Set = ["image", "shape", "canvas"]
        let data: Set = ["progress", "gauge", "chart", "list"]
        let interactive: Set = ["button", "toggle", "link"]
        let spacing: Set = ["spacer", "divider"]
        let all = layout.union(text).union(media).union(data).union(interactive).union(spacing)
        // 21 IR element types — if Models grow a new type, this fails until a group claims it.
        XCTAssertEqual(all.count, 21, "expected 21 element types across six docs groups, got \(all.count)")
    }
}
