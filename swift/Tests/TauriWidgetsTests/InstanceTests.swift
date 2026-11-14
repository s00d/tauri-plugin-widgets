import Foundation
import XCTest
@testable import TauriWidgets

/// Per-placement widgets: provider registration → Edit Widget choices (`instances:*`) →
/// the chosen placement's own config / data → bounded fetch cache.
final class InstanceTests: XCTestCase {
    private var box: StoreSandbox!
    private var savedConfig = ("", "")

    override func setUp() {
        super.setUp()
        box = StoreSandbox("instance")
        savedConfig = (TauriWidgetsConfig.appGroup, TauriWidgetsConfig.widgetId)
    }

    override func tearDown() {
        (TauriWidgetsConfig.appGroup, TauriWidgetsConfig.widgetId) = savedConfig
        box.restore()
        super.tearDown()
    }

    /// Given providers and published `instances:*` lists, when a placement picks an instance, then choices merge, the pick's own config / data win and the fetch cache stays bounded.
    func testInstanceChain() async throws {
        let group = box.group
        let query = TauriInstanceQuery()

        // 1. Providers record kind → widgetId for the host's getActiveWidgets and become the
        //    App Group the entity query reads.
        _ = TauriWidgetProvider(appGroup: group, widgetId: "default", kind: "MyTauriWidget")
        _ = TauriInstanceProvider(appGroup: group, widgetId: "weather", kind: "MyTauriWidget.instance")
        let kinds = UserDefaults(suiteName: group)?.dictionary(forKey: TauriWidgetsConfig.kindsKey) as? [String: String]
        XCTAssertEqual(kinds, ["MyTauriWidget": "default", "MyTauriWidget.instance": "weather"])
        XCTAssertEqual(TauriWidgetsConfig.resolvedAppGroup, group)

        // 2. Nothing published yet: no choices, no default.
        let empty = try await query.suggestedEntities()
        XCTAssertTrue(empty.isEmpty)
        let none = await query.defaultResult()
        XCTAssertNil(none)

        // 3. Two widgets publish lists: merged in key order (`bank` before `weather`), a
        //    duplicate id keeps its first entry, a missing title shows the id, items
        //    without an id are skipped.
        TauriWidgetDataStore.writeValue(#"[{"id":"nyc","title":"New York"},{"id":"ber"},{"title":"no id"}]"#,
                                        forKey: TauriWidgetStoreKeys.instancesPrefix + "weather", appGroup: group)
        TauriWidgetDataStore.writeValue(#"[{"id":"acc","title":"Account"},{"id":"nyc","title":"dup"}]"#,
                                        forKey: TauriWidgetStoreKeys.instancesPrefix + "bank", appGroup: group)
        let suggested = try await query.suggestedEntities()
        XCTAssertEqual(suggested.map(\.id), ["acc", "nyc", "ber"])
        XCTAssertEqual(suggested.map(\.title), ["Account", "dup", "ber"])

        // 4. A new placement starts on the first item; a choice the app dropped stays
        //    selected under its id.
        let first = await query.defaultResult()
        XCTAssertEqual(first?.id, "acc")
        let picked = try await query.entities(for: ["ber", "gone"])
        XCTAssertEqual(picked.map(\.title), ["ber", "gone"])
        XCTAssertEqual(TauriInstanceIntent(instance: picked[0]).instance?.id, "ber")

        // 5. The picked placement: own config beats the widget's, own data beats the
        //    widget's, `{{instance}}` fills in; another placement falls back to the widget.
        TauriWidgetDataStore.writeValue(#"{"small":{"type":"text","content":"{{instance}} {{data.t}}"}}"#,
                                        forKey: TauriWidgetStoreKeys.configKey("weather"), appGroup: group)
        TauriWidgetDataStore.writeValue(#"{"t":"20"}"#, forKey: TauriWidgetStoreKeys.dataPrefix + "weather", appGroup: group)
        TauriWidgetDataStore.writeValue(#"{"t":"25"}"#, forKey: TauriWidgetStoreKeys.dataPrefix + "weather#nyc", appGroup: group)
        func shown(_ instance: String?) -> String? {
            let snap = TauriWidgetDataStore.loadSnapshot(appGroup: group, widgetId: "weather", instance: instance)
            return TauriTimeline.entries(raw: snap.rawConfig, data: snap.data(fetched: nil), now: Date(),
                                         family: .systemSmall, appGroup: group, widgetId: "weather",
                                         instance: instance).first?.config?.small?.content
        }
        XCTAssertEqual(shown("nyc"), "nyc 25")
        XCTAssertEqual(shown("ber"), "ber 20")
        TauriWidgetDataStore.writeValue(#"{"small":{"type":"text","content":"own"}}"#,
                                        forKey: TauriWidgetStoreKeys.configKey("weather#nyc"), appGroup: group)
        XCTAssertEqual(shown("nyc"), "own")
        XCTAssertEqual(shown("ber"), "ber 20")

        // 6. Fetched `source` responses: one per placement, only the most recent kept.
        for i in 0..<(TauriFetchedCache.maxEntries + 5) {
            TauriFetchedCache.write(appGroup: group, scopedId: "w\(i)", raw: "{}", ts: UInt64(i))
        }
        XCTAssertNil(TauriFetchedCache.read(appGroup: group, scopedId: "w0"))
        XCTAssertEqual(TauriFetchedCache.read(appGroup: group, scopedId: "w\(TauriFetchedCache.maxEntries + 4)")?.ts,
                       UInt64(TauriFetchedCache.maxEntries + 4))
        XCTAssertEqual(UserDefaults(suiteName: group)?.dictionary(forKey: TauriFetchedCache.key)?.count,
                       TauriFetchedCache.maxEntries)
    }
}
