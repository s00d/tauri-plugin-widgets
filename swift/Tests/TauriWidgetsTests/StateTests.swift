import Foundation
import XCTest
@testable import TauriWidgets

/// Contract v3 local state on Apple, walked end to end: store keys → `set` operators →
/// a decoded button's `set` tap through `WidgetStateIntent` → stored layers, queued
/// events → merged read → timeline resolving `{{state.…}}`.
final class StateTests: XCTestCase {
    private var box: StoreSandbox!

    override func setUp() {
        super.setUp()
        box = StoreSandbox("state")
    }

    override func tearDown() {
        box.restore()
        super.tearDown()
    }

    /// Given a decoded button with `set`, when it is tapped through `WidgetStateIntent`, then the placement layer is stored, an event queued and the merged read shows it.
    func testSetTapChain() throws {
        let group = box.group

        // 1. Keys: a placement is `w#i`; lookups go placement first, then the widget.
        XCTAssertEqual(TauriWidgetStoreKeys.scopedId("w", nil), "w")
        XCTAssertEqual(TauriWidgetStoreKeys.scopedId("w", ""), "w")
        XCTAssertEqual(TauriWidgetStoreKeys.scopedId("w", "1"), "w#1")
        XCTAssertEqual(TauriWidgetStoreKeys.lookupKeys("config:", "w", "1"), ["config:w#1", "config:w"])
        XCTAssertEqual(TauriWidgetStoreKeys.lookupKeys("data:", "w", nil), ["data:w"])

        // 2. Operators against `current` (merged) while writing into `stored` (own layer):
        //    $toggle on missing → true, $inc/$dec treat bool/missing as 0, null removes.
        let ops = TauriWidgetState.apply(
            obj(#"{"on":"$toggle","off":"$toggle","n":"$inc","f":"$dec","m":"$dec","b":"$inc","keep":null,"s":"x"}"#),
            current: obj(#"{"on":true,"n":2,"f":1.5,"b":false,"keep":1}"#), stored: obj(#"{"keep":1,"other":true}"#))
        XCTAssertEqual(canon(ops), #"{"b":1,"f":0.5,"m":-1,"n":3,"off":true,"on":false,"other":true,"s":"x"}"#)

        // 3. A button's `set` decodes into the patch the tap sends; no `set`, no patch.
        let button = try JSONDecoder().decode(WidgetElement.self, from: Data(
            #"{"type":"button","label":"L","set":{"done":"$toggle","n":"$inc"},"transition":"numeric"}"#.utf8))
        let patch = try XCTUnwrap(DynamicElementView(element: button).statePatch)
        XCTAssertEqual(patch["done"] as? String, "$toggle")
        XCTAssertEqual(button.transition, "numeric")
        let plain = try JSONDecoder().decode(WidgetElement.self, from: Data(#"{"type":"button","label":"L"}"#.utf8))
        XCTAssertNil(DynamicElementView(element: plain).statePatch)

        // 4. The tap on placement `nyc`: defaults come from `config.state`, the result lands
        //    in `state:w#nyc` only.
        let cfg = #"{"state":{"n":1,"done":false},"small":{"type":"text","content":"{{state.n}} {{state.done}}"}}"#
        TauriWidgetDataStore.writeValue(cfg, forKey: TauriWidgetStoreKeys.configKey("w"), appGroup: group)
        let raw = String(decoding: try JSONSerialization.data(withJSONObject: patch, options: [.sortedKeys]), as: UTF8.self)
        let s1 = WidgetStateIntent.run(patch: raw, actionName: "tapped", payload: "p",
                                       widgetId: "w", instance: "nyc", group: group, now: 7)
        XCTAssertEqual(canon(s1), #"{"done":true,"n":2}"#)
        let map = TauriWidgetDataStore.loadFreshestMap(appGroup: group)
        XCTAssertEqual(map["state:w#nyc"], #"{"done":true,"n":2}"#)
        XCTAssertNil(map["state:w"])

        // 5. Queue: the action first, then the `$state` envelope for `widget-state-changed`.
        let queue = TauriWidgetDataStore.readPendingActions(appGroup: group)
        XCTAssertEqual(queue.map(\.action), ["tapped", WidgetStateIntent.stateEnvelopeAction])
        XCTAssertEqual(queue[0].payload, "p")
        XCTAssertEqual(queue[0].ts, 7)
        XCTAssertEqual(canon(TauriWidgetSnapshot.json(queue[1].payload)),
                       #"{"instance":"nyc","state":{"done":true,"n":2},"widgetId":"w"}"#)

        // 6. A widget-level layer sits under the placement's; `null` drops the placement's
        //    key so the layer below shows again.
        TauriWidgetDataStore.writeValue(#"{"n":10,"x":1}"#, forKey: "state:w", appGroup: group)
        let snap = TauriWidgetDataStore.loadSnapshot(appGroup: group, widgetId: "w", instance: "nyc")
        XCTAssertEqual(canon(snap.state), #"{"done":true,"n":2,"x":1}"#)
        let s2 = WidgetStateIntent.run(patch: #"{"n":null}"#, actionName: nil, payload: nil,
                                       widgetId: "w", instance: "nyc", group: group)
        XCTAssertEqual(canon(s2), #"{"done":true,"n":10,"x":1}"#)
        // No action name → only the `$state` envelope is added.
        XCTAssertEqual(TauriWidgetDataStore.readPendingActions(appGroup: group).map(\.action),
                       ["tapped", "$state", "$state"])

        // 7. The timeline and the gallery preview resolve `{{state.…}}` with the merged state.
        let snap2 = TauriWidgetDataStore.loadSnapshot(appGroup: group, widgetId: "w", instance: "nyc")
        let entry = TauriTimeline.entries(raw: snap2.rawConfig, data: nil, state: snap2.state, now: Date(),
                                          family: .systemSmall, appGroup: group, widgetId: "w", instance: "nyc")
        XCTAssertEqual(entry.first?.config?.small?.content, "10 true")
        XCTAssertEqual(TauriTimeline.preview(appGroup: group, widgetId: "w", instance: "nyc", family: .systemSmall)?
            .config?.small?.content, "10 true")
        XCTAssertNil(TauriTimeline.preview(appGroup: group, widgetId: "none", instance: nil, family: .systemSmall))
    }

    /// Given an action tap queued for the app (calculator "="), when the app answers by writing the widget's state, then the intent's wait ends at once with `true`; when nobody answers it gives up after its timeout with `false`, so the system's next timeline is never held longer.
    func testActionWaitsForTheAppsAnswer() async {
        let group = box.group
        TauriWidgetDataStore.writeValue(#"{"r":"0"}"#, forKey: "state:w", appGroup: group)
        let answer = Task.detached {
            try? await Task.sleep(nanoseconds: 100_000_000)
            TauriWidgetDataStore.writeValue(#"{"r":"2"}"#, forKey: "state:w", appGroup: group)
        }
        let answered = await WidgetActionIntent.awaitHostAnswer(appGroup: group, widgetId: "w", timeout: 5)
        XCTAssertTrue(answered)
        await answer.value

        let start = Date()
        let silent = await WidgetActionIntent.awaitHostAnswer(appGroup: group, widgetId: "w", timeout: 0.2)
        XCTAssertFalse(silent)
        XCTAssertLessThan(Date().timeIntervalSince(start), 2, "gives up at its timeout")
    }

    /// Given merged state, when templates use `state.*` in text, `bind`, `when` and `repeat`, then each resolves and a loop variable named `state` shadows the root.
    func testResolveWithState() {
        let cfg = obj(#"{"state":{"n":1,"done":false},"small":{"type":"vstack","children":[{"type":"text","content":"{{state.n}}/{{state.missing}}"},{"type":"toggle","bind":{"isOn":"state.done"}},{"type":"text","content":"hidden","when":{"path":"state.done"}},{"type":"repeat","each":"state.list","template":{"type":"text","content":"{{item}}"}}]}}"#)

        // Defaults alone: `done` false hides the `when` child, the empty list repeats nothing.
        let before = TauriResolve.resolve(cfg, .init(nowMs: 0, locale: "en", state: cfg["state"]))
        let k0 = (before["small"] as! [String: Any])["children"] as! [[String: Any]]
        XCTAssertEqual(k0.compactMap { $0["content"] as? String }, ["1/"])
        XCTAssertEqual(k0[1]["isOn"] as? Bool, false)

        // Stored layer over the defaults: the toggle binds, `when` shows, `repeat` expands.
        let state = TauriResolve.mergeData([cfg["state"], obj(#"{"done":true,"list":["a","b"]}"#)])
        let out = TauriResolve.resolve(cfg, .init(nowMs: 0, locale: "en", state: state))
        let kids = (out["small"] as! [String: Any])["children"] as! [[String: Any]]
        XCTAssertEqual(kids[0]["content"] as? String, "1/")
        XCTAssertEqual(kids[1]["isOn"] as? Bool, true)
        XCTAssertEqual(kids.compactMap { $0["content"] as? String }, ["1/", "hidden", "a", "b"])

        let shadow = obj(#"{"small":{"type":"repeat","each":"data.l","as":"state","template":{"type":"text","content":"{{state}}"}}}"#)
        let r = TauriResolve.resolve(shadow, .init(nowMs: 0, locale: "", data: ["l": ["x"]], state: ["k": 1]))
        XCTAssertEqual(canon(r["small"]), #"{"children":[{"content":"x","type":"text"}],"type":"vstack"}"#)
        XCTAssertEqual(TauriResolve.contractVersion, 3)
        XCTAssertEqual(TauriResolve.interpolate([:], .init(nowMs: 0, locale: ""), "{{stateful}}"), "{{stateful}}")
    }
}
