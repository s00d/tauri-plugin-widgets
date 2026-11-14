import Foundation
import XCTest

#if canImport(ActivityKit)
import ActivityKit
#endif

@testable import tauri_plugin_widgets

/// Host plugin (iOS / macOS app side): command args as the JS bridge sends them, the
/// App Group UserDefaults they land in, `setWidgetState` patches and the Live Activity
/// request → update → end payloads.
final class WidgetPluginTests: XCTestCase {
    private func decode<T: Decodable>(_ type: T.Type, _ json: String) throws -> T {
        try JSONDecoder().decode(type, from: Data(json.utf8))
    }

    private func roundTrip<T: Decodable>(_ type: T.Type, _ object: [String: Any]) throws -> T {
        try JSONDecoder().decode(type, from: try JSONSerialization.data(withJSONObject: object))
    }

    /// Given the plugin entry point, when it initialises, then it returns a `WidgetPlugin`.
    func testPluginInit() {
        XCTAssertTrue(initPlugin() is WidgetPlugin)
        XCTAssertNotNil(WidgetPlugin())
    }

    /// Given bridge JSON for setItems → getItems → register → reload, when it decodes, then valid args round-trip and missing / snake_case keys are rejected.
    func testCommandArgsChain() throws {
        // 1. setItems: plain, empty value, escapes, 10k value, via a JSON object.
        let set = try decode(SetItemsArgs.self, #"{"key": "widget_text", "value": "Hello", "group": "group.com.example"}"#)
        XCTAssertEqual([set.key, set.value, set.group], ["widget_text", "Hello", "group.com.example"])
        XCTAssertEqual(try decode(SetItemsArgs.self, #"{"key": "k", "value": "", "group": "g"}"#).value, "")
        let esc = try decode(SetItemsArgs.self, #"{"key": "k/e.y-1", "value": "line1\nline2\ttab", "group": "group.test"}"#)
        XCTAssertEqual(esc.key, "k/e.y-1")
        XCTAssertEqual(esc.value, "line1\nline2\ttab")
        XCTAssertEqual(try roundTrip(SetItemsArgs.self, ["key": "big", "value": String(repeating: "A", count: 10_000), "group": "g"])
            .value.count, 10_000)
        let rt = try roundTrip(SetItemsArgs.self, ["key": "widget_text", "value": "Test Value", "group": "group.com.example"])
        XCTAssertEqual([rt.key, rt.value, rt.group], ["widget_text", "Test Value", "group.com.example"])
        XCTAssertThrowsError(try decode(SetItemsArgs.self, #"{"key": "k", "value": "v"}"#), "group is required")

        // 2. getItems reads the same key back.
        let get = try decode(GetItemsArgs.self, #"{"key": "title", "group": "group.app"}"#)
        XCTAssertEqual([get.key, get.group], ["title", "group.app"])
        XCTAssertThrowsError(try decode(GetItemsArgs.self, #"{"group": "g"}"#), "key is required")

        // 3. setRegisterWidget: order kept, empty allowed.
        XCTAssertEqual(try decode(SetRegisterWidgetArgs.self, #"{"widgets": ["com.example.Widget1", "com.example.Widget2"]}"#).widgets,
                       ["com.example.Widget1", "com.example.Widget2"])
        XCTAssertTrue(try decode(SetRegisterWidgetArgs.self, #"{"widgets": []}"#).widgets.isEmpty)
        XCTAssertEqual(try roundTrip(SetRegisterWidgetArgs.self, ["widgets": ["com.a.W1", "com.b.W2", "com.c.W3"]]).widgets,
                       ["com.a.W1", "com.b.W2", "com.c.W3"])

        // 4. reloadTimelines: camelCase `ofKind` only.
        XCTAssertEqual(try decode(ReloadTimelinesArgs.self, #"{"ofKind": "MyWidgetKind"}"#).ofKind, "MyWidgetKind")
        XCTAssertEqual(try decode(ReloadTimelinesArgs.self, #"{"ofKind": "com.example.MyWidget"}"#).ofKind, "com.example.MyWidget")
        XCTAssertEqual(try roundTrip(ReloadTimelinesArgs.self, ["ofKind": "MyWidget"]).ofKind, "MyWidget")
        XCTAssertThrowsError(try decode(ReloadTimelinesArgs.self, #"{"of_kind": "test"}"#))
    }

    /// Given an App Group suite, when keys are set, overwritten and added, then empty / JSON / non-ASCII strings round-trip and other keys stay.
    func testUserDefaultsChain() throws {
        let suite = "test.tauri.plugin.widgets.\(UUID().uuidString)"
        defer { UserDefaults.standard.removePersistentDomain(forName: suite) }
        let d = try XCTUnwrap(UserDefaults(suiteName: suite))

        XCTAssertNil(d.string(forKey: "widget_text"))
        d.set("Hello World", forKey: "widget_text")
        XCTAssertEqual(d.string(forKey: "widget_text"), "Hello World")
        d.set("second", forKey: "widget_text")
        XCTAssertEqual(d.string(forKey: "widget_text"), "second")

        let values = ["key1": "val1", "key2": "val2", "empty_key": "",
                      "json_data": "{\"title\":\"Hello\",\"count\":42}", "unicode_key": "Привет мир 你好世界 🌍"]
        for (k, v) in values { d.set(v, forKey: k) }
        d.synchronize()
        for (k, v) in values { XCTAssertEqual(d.string(forKey: k), v, k) }
        XCTAssertEqual(d.string(forKey: "widget_text"), "second", "other keys untouched")
    }

    /// Given `setWidgetState` args, when a patch applies twice, then the operators match the extension's `set` tap and the second sees the first.
    func testWidgetStateChain() throws {
        let set = try decode(StateArgs.self, #"{"group":"group.a","widgetId":"w","instance":"i","patch":{"n":"$inc"}}"#)
        XCTAssertEqual(set.patch?.dictionary?["n"] as? String, "$inc")
        XCTAssertEqual(set.instance, "i")
        let clear = try decode(StateArgs.self, #"{"group":"group.a","widgetId":"w","patch":null}"#)
        XCTAssertNil(clear.patch)
        XCTAssertNil(clear.instance)

        let patch: [String: Any] = ["on": "$toggle", "n": "$inc", "m": "$dec", "gone": NSNull(), "s": "x"]
        let first = WidgetPlugin.applyStatePatch(patch, current: ["on": true, "n": 2], stored: ["gone": 1])
        XCTAssertEqual(first["on"] as? Bool, false)
        XCTAssertEqual((first["n"] as? NSNumber)?.intValue, 3)
        XCTAssertEqual((first["m"] as? NSNumber)?.intValue, -1)
        XCTAssertNil(first["gone"])
        XCTAssertEqual(first["s"] as? String, "x")

        // The next tap sees the stored result as current.
        let second = WidgetPlugin.applyStatePatch(patch, current: first, stored: first)
        XCTAssertEqual(second["on"] as? Bool, true)
        XCTAssertEqual((second["n"] as? NSNumber)?.intValue, 4)
        XCTAssertEqual((second["m"] as? NSNumber)?.intValue, -2)
    }

    /// Given Live Activity payloads, when start → update → end decode, then ids, relevance, alerts, timestamps and dismissal policy map.
    /// The out-of-order `timestamp` drop (`remember`) is private and needs a running
    /// ActivityKit activity, so only the decoded `timestamp` it compares is checked here.
    func testActivityLifecycleChain() throws {
        // 0. out-of-order guard: start@100 → update@200 → late update@150 dropped → no
        //    timestamp applies → end resets → an older start applies again; ids are separate.
        var guardTs = ActivityTimestampGuard()
        XCTAssertTrue(guardTs.accept(id: "a", timestamp: 100))
        XCTAssertTrue(guardTs.accept(id: "a", timestamp: 200))
        XCTAssertFalse(guardTs.accept(id: "a", timestamp: 150))
        XCTAssertTrue(guardTs.accept(id: "a", timestamp: 200), "same timestamp applies")
        XCTAssertTrue(guardTs.accept(id: "a", timestamp: nil))
        XCTAssertTrue(guardTs.accept(id: "b", timestamp: 1), "other ids are independent")
        guardTs.forget("a")
        XCTAssertTrue(guardTs.accept(id: "a", timestamp: 50))
        // 1. start: the request keeps its JSON; `id`, `staleAt`, `push` are read out.
        let start = try decode(ActivityArgs.self, #"{"group":"group.a","content":{"id":"a","title":"Go","staleAt":5000,"push":true}}"#)
        XCTAssertEqual(start.group, "group.a")
        XCTAssertEqual(start.content.id, "a")
        XCTAssertEqual(start.content.staleAt, 5000)
        XCTAssertTrue(start.content.push)
        XCTAssertNil(start.content.timestamp)
        XCTAssertTrue(start.content.json.contains("\"title\":\"Go\""))

        // 2. updates: relevance, alert, and a timestamp per update (older ones lose).
        let u1 = try decode(JSObjectBox.self, #"{"id":"a","relevanceScore":50,"timestamp":1000,"alert":{"title":"T","body":"B","sound":"ding.caf"}}"#)
        XCTAssertEqual(u1.relevanceScore, 50)
        XCTAssertEqual(u1.timestamp, 1000)
        XCTAssertEqual(u1.alert?.title, "T")
        XCTAssertEqual(u1.alert?.body, "B")
        XCTAssertEqual(u1.alert?.sound, "ding.caf")
        XCTAssertFalse(u1.push)
        let u2 = try decode(JSObjectBox.self, #"{"id":"a","timestamp":900,"alert":{"body":"no title"}}"#)
        XCTAssertLessThan(try XCTUnwrap(u2.timestamp), try XCTUnwrap(u1.timestamp))
        XCTAssertNil(u2.alert, "an alert needs a title")

        // 3. end: final content and a dismissal policy.
        let end = try decode(EndActivityArgs.self, #"{"id":"a","dismissalPolicy":"default","content":{"id":"a","title":"Done"}}"#)
        XCTAssertEqual(end.dismissalPolicy, "default")
        XCTAssertNil(end.dismissAfterMs)
        XCTAssertTrue(end.content?.json.contains("Done") == true)

        #if canImport(ActivityKit)
        if #available(iOS 16.2, *) {
            let c = WidgetPlugin.content(start.content)
            XCTAssertEqual(c.staleDate, Date(timeIntervalSince1970: 5))
            XCTAssertEqual(c.relevanceScore, 0)
            XCTAssertEqual(WidgetPlugin.content(u1).relevanceScore, 50)
            XCTAssertEqual(WidgetPlugin.dismissalPolicy(afterMs: nil, named: "default"), .default)
            XCTAssertEqual(WidgetPlugin.dismissalPolicy(afterMs: nil, named: nil), .immediate)
            XCTAssertEqual(WidgetPlugin.dismissalPolicy(afterMs: 0, named: "default"), .default)
            XCTAssertNotEqual(WidgetPlugin.dismissalPolicy(afterMs: 60_000, named: "default"), .default)
        }
        #endif
    }
}
