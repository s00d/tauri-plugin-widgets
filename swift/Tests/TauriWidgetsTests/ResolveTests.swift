import Foundation
import WidgetKit
import XCTest
@testable import TauriWidgets

/// Same fixtures as `src/resolve.rs`, `tests/unit/resolve.test.ts` and the Kotlin port.
final class ResolveTests: XCTestCase {
    /// Given the shared resolve fixtures, when each resolves, then the output matches the expected JSON of every port.
    func testSharedFixtures() throws {
        let dir = RepoPaths.fixtures.appendingPathComponent("resolve")
        let files = try FileManager.default.contentsOfDirectory(atPath: dir.path).filter { $0.hasSuffix(".json") }.sorted()
        XCTAssertGreaterThanOrEqual(files.count, 5)
        var executed: [String] = []
        for f in files {
            let c = obj(try String(contentsOf: dir.appendingPathComponent(f), encoding: .utf8))
            // Fixture format v2: `platforms` allow-list, `skip.swift` with a reason.
            if let only = c["platforms"] as? [String], !only.contains("swift") { continue }
            if let reason = (c["skip"] as? [String: Any])?["swift"] as? String {
                XCTAssertFalse(reason.isEmpty, "\(f): skip needs a reason")
                continue
            }
            executed.append(f)
            let config = c["config"] as! [String: Any]
            let x = c["ctx"] as! [String: Any]
            let ctx = TauriResolve.Context(
                nowMs: (x["nowMs"] as! NSNumber).uint64Value,
                locale: x["locale"] as? String ?? "",
                instance: x["instance"] as? String,
                data: TauriResolve.mergeData([config["data"], x["data"]]),
                // Merged state sits beside `data` (in `ctx`, or at the case root).
                state: TauriResolve.mergeData([config["state"], c["state"], x["state"]])
            )
            XCTAssertEqual(canon(TauriResolve.resolve(config, ctx)), canon(c["expected"]), f)
            let next = TauriResolve.nextChangeMs(config, nowMs: ctx.nowMs)
            XCTAssertEqual(next.map { NSNumber(value: $0) }.map(canon) ?? "null", canon(c["nextChangeMs"]), f)
            if c["source"] is NSNull {
                XCTAssertNil(TauriResolve.sourceRequest(config, ctx), f)
            }
            if let want = c["source"] as? [String: Any] {
                let got = try XCTUnwrap(TauriResolve.sourceRequest(config, ctx), f)
                XCTAssertEqual(got.url, want["url"] as? String, f)
                XCTAssertEqual(got.method, want["method"] as? String, f)
                XCTAssertEqual(got.headers, want["headers"] as? [String: String], f)
                XCTAssertEqual(got.body, want["body"] as? String, f)
                XCTAssertEqual(got.refreshMinutes, (want["refreshMinutes"] as? NSNumber)?.intValue, f)
            }
        }
        print("swift resolve fixtures executed: \(executed.joined(separator: ", "))")
        XCTAssertFalse(executed.isEmpty)
    }

    /// Given timed `entries`, when time passes their boundaries, then the layout switches and the timeline lists one entry per change.
    func testEntriesTimelineChain() {
        // 1. Active layouts: unsorted `at`, inclusive boundary, per-family merge; negative,
        //    non-numeric and missing `at` never activate; a null layout keeps the base.
        let cfg = obj(#"{"small":{"c":"a"},"entries":[{"at":300,"small":{"c":"c"}},{"at":100,"small":{"c":"b"}},{"at":100,"medium":{"c":"m"}},{"at":-1,"small":{"c":"neg"}},{"at":true,"small":{"c":"bool"}},{"small":{"c":"noat"}}]}"#)
        XCTAssertEqual(canon(TauriResolve.activeLayouts(cfg, nowMs: 99)), #"{"small":{"c":"a"}}"#)
        XCTAssertEqual(canon(TauriResolve.activeLayouts(cfg, nowMs: 100)), #"{"medium":{"c":"m"},"small":{"c":"a"}}"#)
        XCTAssertEqual(canon(TauriResolve.activeLayouts(cfg, nowMs: 300)["small"]), #"{"c":"c"}"#)
        XCTAssertEqual(canon(TauriResolve.activeLayouts(obj(#"{"small":1,"entries":[{"at":0,"small":null}]}"#), nowMs: 0)), #"{"small":1}"#)

        // 2. The schedule the timeline is built from.
        XCTAssertEqual(TauriResolve.nextChangeMs(cfg, nowMs: 99), 100)
        XCTAssertEqual(TauriResolve.nextChangeMs(cfg, nowMs: 300), nil)
        XCTAssertEqual(TauriResolve.upcomingChanges(cfg, nowMs: 0, limit: 10), [100, 300])
        XCTAssertEqual(TauriResolve.upcomingChanges(cfg, nowMs: 0, limit: 1), [100])

        // 3. Resolve happens before decoding, at the entry's own time.
        let raw = #"{"small":{"type":"text","content":"{{data.v}}"},"entries":[{"at":5,"small":{"type":"text","content":"later"}}]}"#
        XCTAssertEqual(TauriResolve.decode(raw, .init(nowMs: 0, locale: "en", data: ["v": "hi"]))?.small?.content, "hi")
        XCTAssertEqual(TauriResolve.decode(raw, .init(nowMs: 5, locale: "en"))?.small?.content, "later")

        // 4. WidgetKit timeline: one entry now plus one per future change.
        let future = #"{"small":{"type":"text","content":"a"},"entries":[{"at":4102444800000,"small":{"type":"text","content":"b"}}]}"#
        let list = TauriTimeline.entries(raw: future, data: nil, now: Date(), family: .systemSmall,
                                         appGroup: "group.t", widgetId: "w", instance: "i")
        XCTAssertEqual(list.count, 2)
        XCTAssertEqual(list[0].config?.small?.content, "a")
        XCTAssertEqual(list[1].config?.small?.content, "b")
        XCTAssertEqual(list[1].date, Date(timeIntervalSince1970: 4_102_444_800))
        XCTAssertEqual(list[1].instance, "i")
        XCTAssertEqual(TauriTimeline.entries(raw: nil, data: nil, now: Date(), family: .systemSmall,
                                             appGroup: "g", widgetId: "w", instance: nil).count, 1)

        // 5. The next timeline request comes at the last entry when it is sooner than the
        //    regular refresh, so a short schedule does not freeze on its last entry.
        let now = Date(timeIntervalSince1970: 1_000)
        let soon = #"{"small":{"type":"text","content":"a"},"entries":[{"at":1060000,"small":{"type":"text","content":"b"}}]}"#
        let short = TauriTimeline.entries(raw: soon, data: nil, now: now, family: .systemSmall,
                                          appGroup: "g", widgetId: "w", instance: nil)
        XCTAssertEqual(TauriTimeline.refreshDate(now: now, minutes: 15, entries: short),
                       Date(timeIntervalSince1970: 1_060))
        XCTAssertEqual(TauriTimeline.refreshDate(now: now, minutes: 15, entries: list),
                       Date(timeIntervalSince1970: 1_000 + 15 * 60), "far schedule: regular refresh")
        XCTAssertEqual(TauriTimeline.refreshDate(now: now, minutes: 15, entries: [short[0]]),
                       Date(timeIntervalSince1970: 1_000 + 15 * 60), "no schedule: regular refresh")
        XCTAssertEqual(TauriTimeline.size(.systemSmall), "small")
        XCTAssertEqual(TauriTimeline.size(.systemMedium), "medium")
        XCTAssertEqual(TauriTimeline.size(.systemLarge), "large")
    }

    /// Given data layers and string tables, when placeholders resolve, then layers merge, the locale table is picked and values format like the other ports.
    func testInterpolationChain() {
        // 1. Data layers: later objects override keys, a scalar replaces, nulls are skipped.
        XCTAssertEqual(canon(TauriResolve.mergeData([obj(#"{"x":1,"y":1}"#), nil, NSNull(), obj(#"{"y":2}"#)])), #"{"x":1,"y":2}"#)
        XCTAssertEqual(canon(TauriResolve.mergeData([obj(#"{"x":1}"#), 5])), "5")
        XCTAssertEqual(canon(TauriResolve.mergeData([5, obj(#"{"y":2}"#)])), #"{"y":2}"#)
        XCTAssertNil(TauriResolve.mergeData([]))

        // 2. Values: integers without `.0`, bools, null/missing/out-of-range empty, objects as JSON.
        let data = obj(#"{"t":72.0,"f":1.5,"b":true,"z":false,"n":null,"list":["x","y"],"o":{"k":1}}"#)
        let s = TauriResolve.interpolate([:], .init(nowMs: 0, locale: "en", data: data),
            "{{data.t}}|{{data.f}}|{{data.b}}|{{data.z}}|{{data.n}}|{{data.list.1}}|{{data.o}}|{{data.missing}}|{{data.list.9}}|{{ data.t }}|{{data.list.x}}|{{data.t.deep}}")
        XCTAssertEqual(s, #"72|1.5|true|false||y|{"k":1}|||72||"#)
        XCTAssertEqual(TauriResolve.interpolate([:], .init(nowMs: 0, locale: "ru"), "{{oops}} {{locale}} {{data.x"), "{{oops}} ru {{data.x")
        XCTAssertEqual(TauriResolve.interpolate([:], .init(nowMs: 0, locale: "", instance: "nyc"), "[{{instance}}]"), "[nyc]")
        XCTAssertEqual(TauriResolve.interpolate([:], .init(nowMs: 0, locale: ""), "[{{instance}}]{{data.a}}"), "[]")

        // 3. Locale table: exact, region-insensitive, language, then default / en / first.
        let tables = obj(#"{"defaultLocale":"de","strings":{"en":{},"de":{},"pt-BR":{},"ru":{}}}"#)
        XCTAssertEqual(TauriResolve.pickLocaleTable(tables, locale: "pt_BR"), "pt-BR")
        XCTAssertEqual(TauriResolve.pickLocaleTable(tables, locale: "PT-pt"), "pt-BR")
        XCTAssertEqual(TauriResolve.pickLocaleTable(tables, locale: "ru-RU"), "ru")
        XCTAssertEqual(TauriResolve.pickLocaleTable(tables, locale: "fr"), "de")
        XCTAssertEqual(TauriResolve.pickLocaleTable(tables, locale: ""), "de")
        XCTAssertEqual(TauriResolve.pickLocaleTable(obj(#"{"strings":{"fr":{},"en":{}}}"#), locale: "ja"), "en")
        XCTAssertEqual(TauriResolve.pickLocaleTable(obj(#"{"strings":{"fr":{},"de":{}}}"#), locale: "ja"), "de")
        XCTAssertNil(TauriResolve.pickLocaleTable(obj("{}"), locale: "en"))
        XCTAssertNil(TauriResolve.pickLocaleTable(obj(#"{"strings":{}}"#), locale: "en"))

        // 4. `t.*` from the picked table, falling back to the default table, then the key.
        let cfg = obj(#"{"defaultLocale":"en","strings":{"en":{"a":"A","b":"B"},"ru":{"a":"А"}}}"#)
        XCTAssertEqual(TauriResolve.interpolate(cfg, .init(nowMs: 0, locale: "ru"), "{{t.a}}{{t.b}}{{t.c}}"), "АBc")

        // 5. A config without layouts keeps its version (default 1).
        XCTAssertEqual(canon(TauriResolve.resolve(obj(#"{"version":2}"#), .init(nowMs: 0, locale: ""))), #"{"version":2}"#)
        XCTAssertEqual(canon(TauriResolve.resolve([:], .init(nowMs: 0, locale: ""))), #"{"version":1}"#)
    }

    /// Given `source` configs, when the request is built, then only allowed URLs pass and defaults, templating and clamping apply.
    func testSourceRequestChain() {
        let ctx = TauriResolve.Context(nowMs: 0, locale: "ru")
        func get(_ src: String) -> TauriResolve.SourceRequest? {
            TauriResolve.sourceRequest(obj(#"{"source":"# + src + "}"), ctx)
        }
        // 1. Only https, or http to exactly localhost / 127.0.0.1.
        XCTAssertNil(get(#"{"url":"http://example.com"}"#))
        for bad in ["http://localhost.evil.com/a", "http://localhost@evil.com/a", "http://127.0.0.1.nip.io/a", "http://localhost:80x/a"] {
            XCTAssertFalse(TauriResolve.sourceURLAllowed(bad), bad)
        }
        XCTAssertTrue(TauriResolve.sourceURLAllowed("http://LOCALHOST"))
        XCTAssertTrue(TauriResolve.sourceURLAllowed("http://127.0.0.1:8080?x=1"))
        XCTAssertNil(get(#"{}"#))
        XCTAssertNil(TauriResolve.sourceRequest([:], ctx))

        // 2. Defaults: GET, no headers, no body, 30 minutes.
        XCTAssertEqual(get(#"{"url":"http://localhost:1/a"}"#),
                       .init(url: "http://localhost:1/a", method: "GET", headers: [:], body: nil, refreshMinutes: 30))
        XCTAssertNotNil(get(#"{"url":"http://127.0.0.1/a"}"#))

        // 3. Templated url/body/headers, method upper-cased, non-string headers dropped,
        //    refresh clamped to 15...1440, body only with a non-GET method.
        XCTAssertEqual(get(#"{"url":"https://a/{{locale}}","method":"post","body":"{{locale}}","refreshMinutes":1,"headers":{"A":"{{locale}}","B":1}}"#),
                       .init(url: "https://a/ru", method: "POST", headers: ["A": "ru"], body: "ru", refreshMinutes: 15))
        XCTAssertEqual(get(#"{"url":"https://a","body":"x","refreshMinutes":99999}"#)?.refreshMinutes, 1440)
        XCTAssertNil(get(#"{"url":"https://a","body":"x"}"#)?.body)
        XCTAssertEqual(get(#"{"url":"https://a","refreshMinutes":-5}"#)?.refreshMinutes, 30)
        XCTAssertEqual(get(#"{"url":"https://a","refreshMinutes":20}"#)?.refreshMinutes, 20)
    }
}
