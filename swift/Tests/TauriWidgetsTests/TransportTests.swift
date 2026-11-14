import Foundation
import XCTest
@testable import TauriWidgets

/// Symmetric with Rust `tests/transports.rs`: same temp Containers layout. One
/// store walked through the transport races: Rust-shaped container write → receipt →
/// App Group file vs container by nonce → stale UserDefaults with a higher nonce.
final class TransportTests: XCTestCase {
    private var box: StoreSandbox!

    override func setUp() {
        super.setUp()
        box = StoreSandbox("transport", rustLayout: true)
    }

    override func tearDown() {
        box.restore()
        super.tearDown()
    }

    private func map(_ content: String, nonce: String, at: String) -> [String: String] {
        [
            TauriWidgetStoreKeys.configKey("probe"): #"{"version":1,"small":{"type":"text","content":"\#(content)"}}"#,
            TauriWidgetStoreKeys.metaNonce: nonce,
            TauriWidgetStoreKeys.metaUpdatedAt: at,
        ]
    }

    /// Given Rust-shaped writes in the container, App Group file and stale UserDefaults, when Swift loads, then the freshest live file wins and the receipt never bumps the nonce.
    func testTransportRace() throws {
        let group = box.group

        // 1. Rust wrote the sandbox container file: Swift reads it from the container.
        try box.writeMap(map("from-rust", nonce: "42", at: "1700000000000"), to: box.ownFile)
        let (loaded, source) = TauriWidgetDataStore.loadFreshestMapWithSource(appGroup: group)
        XCTAssertEqual(loaded[TauriWidgetStoreKeys.metaNonce], "42")
        XCTAssertEqual(source, TauriWidgetTransportName.container)
        XCTAssertEqual(TauriWidgetDataStore.loadConfigAcknowledging(appGroup: group, widgetId: "probe")?.small?.content,
                       "from-rust")

        // 2. The read plants a receipt in a sibling file; the config map's nonce is untouched.
        let receipt = try JSONDecoder().decode(WidgetTransportReceipt.self, from: Data(
            contentsOf: URL(fileURLWithPath: TauriWidgetDataStore.ownContainerReceiptPath())))
        XCTAssertEqual(receipt.source, TauriWidgetTransportName.container)
        XCTAssertEqual(receipt.readFrom, TauriWidgetTransportName.container)
        XCTAssertEqual(receipt.nonce, 42)
        let after = try JSONSerialization.jsonObject(with: Data(contentsOf: box.ownFile)) as? [String: String]
        XCTAssertEqual(after?[TauriWidgetStoreKeys.metaNonce], "42", "receipt must not bump config nonce")

        // 3. Both file transports carry a config: the higher nonce wins.
        try box.writeMap(map("old", nonce: "1", at: "1"), to: box.appGroupFile)
        try box.writeMap(map("new", nonce: "99", at: "2"), to: box.ownFile)
        XCTAssertEqual(TauriWidgetDataStore.loadConfig(appGroup: group, widgetId: "probe")?.small?.content, "new")
        try box.writeMap(map("ag", nonce: "100", at: "3"), to: box.appGroupFile)
        XCTAssertEqual(TauriWidgetDataStore.loadFreshestMapWithSource(appGroup: group).1, TauriWidgetTransportName.appGroup)
        XCTAssertEqual(TauriWidgetDataStore.loadConfig(appGroup: group, widgetId: "probe")?.small?.content, "ag")

        // 4. Stale UserDefaults with a leftover high nonce never beat live file writes.
        try box.writeMap(map("old", nonce: "1", at: "1"), to: box.appGroupFile)
        try box.writeMap(map("from-file", nonce: "3", at: "300"), to: box.ownFile)
        let ud = UserDefaults(suiteName: group)!
        ud.set(map("stale-ud", nonce: "65", at: "1"), forKey: "widget_data")
        ud.synchronize()
        let (m, src) = TauriWidgetDataStore.loadFreshestMapWithSource(appGroup: group)
        XCTAssertEqual(src, TauriWidgetTransportName.container)
        XCTAssertEqual(m[TauriWidgetStoreKeys.metaNonce], "3")
        XCTAssertEqual(TauriWidgetDataStore.loadConfig(appGroup: group, widgetId: "probe")?.small?.content, "from-file")
    }
}
