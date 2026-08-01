import Foundation
import XCTest
@testable import TauriWidgets

/// Symmetric with Rust `tests/macos_transports.rs`: same temp Containers layout,
/// Swift `loadFreshestMap` must see what a Rust-shaped write left on disk.
final class TransportTests: XCTestCase {
    private var tmpRoot: URL!
    private var previousEnv: [String: String?] = [:]

    override func setUp() {
        super.setUp()
        tmpRoot = FileManager.default.temporaryDirectory
            .appendingPathComponent("tauri-widgets-transport-\(UUID().uuidString)")
        try? FileManager.default.createDirectory(at: tmpRoot, withIntermediateDirectories: true)

        setEnv("WIDGET_CONTAINER_ROOT", tmpRoot.path)
        setEnv("WIDGET_EXTENSION_BUNDLE", "test.app.widgetkit")
        setEnv(
            "WIDGET_APP_GROUP_DATA_FILE",
            tmpRoot.appendingPathComponent("appgroup/widget_data.json").path
        )
    }

    override func tearDown() {
        for (k, v) in previousEnv {
            if let v {
                setenv(k, v, 1)
            } else {
                unsetenv(k)
            }
        }
        previousEnv.removeAll()
        try? FileManager.default.removeItem(at: tmpRoot)
        super.tearDown()
    }

    func testLoadFreshestSeesRustSandboxWrite() throws {
        let group = "group.test.app"
        let widgetId = "probe"
        let sandbox = URL(fileURLWithPath: TauriWidgetDataStore.ownContainerDataPath())

        let map: [String: String] = [
            TauriWidgetStoreKeys.configKey(widgetId): #"{"version":1,"small":{"type":"text","content":"from-rust"}}"#,
            TauriWidgetStoreKeys.metaNonce: "42",
            TauriWidgetStoreKeys.metaUpdatedAt: "1700000000000",
        ]
        try writeMap(map, to: sandbox)

        let (loaded, source) = TauriWidgetDataStore.loadFreshestMapWithSource(appGroup: group)
        XCTAssertEqual(loaded[TauriWidgetStoreKeys.metaNonce], "42")
        XCTAssertEqual(source, TauriWidgetTransportName.container)

        let cfg = TauriWidgetDataStore.loadConfigAcknowledging(appGroup: group, widgetId: widgetId)
        XCTAssertEqual(cfg?.small?.content, "from-rust")

        // Receipt is a sibling file — config map nonce unchanged.
        let receiptURL = URL(fileURLWithPath: TauriWidgetDataStore.ownContainerReceiptPath())
        let receiptData = try Data(contentsOf: receiptURL)
        let receipt = try JSONDecoder().decode(WidgetTransportReceipt.self, from: receiptData)
        XCTAssertEqual(receipt.source, TauriWidgetTransportName.container)
        XCTAssertEqual(receipt.readFrom, TauriWidgetTransportName.container)
        XCTAssertEqual(receipt.nonce, 42)

        let mapAfter = try JSONSerialization.jsonObject(
            with: Data(contentsOf: sandbox)
        ) as? [String: String]
        XCTAssertEqual(mapAfter?[TauriWidgetStoreKeys.metaNonce], "42", "receipt must not bump config nonce")
    }

    func testPickFreshestAcrossFileTransports() throws {
        let group = "group.test.app"
        let widgetId = "probe"

        let older: [String: String] = [
            TauriWidgetStoreKeys.configKey(widgetId): #"{"version":1,"small":{"type":"text","content":"old"}}"#,
            TauriWidgetStoreKeys.metaNonce: "1",
            TauriWidgetStoreKeys.metaUpdatedAt: "1",
        ]
        let newer: [String: String] = [
            TauriWidgetStoreKeys.configKey(widgetId): #"{"version":1,"small":{"type":"text","content":"new"}}"#,
            TauriWidgetStoreKeys.metaNonce: "99",
            TauriWidgetStoreKeys.metaUpdatedAt: "2",
        ]

        try writeMap(
            older,
            to: URL(fileURLWithPath: ProcessInfo.processInfo.environment["WIDGET_APP_GROUP_DATA_FILE"]!)
        )
        try writeMap(newer, to: URL(fileURLWithPath: TauriWidgetDataStore.ownContainerDataPath()))

        let cfg = TauriWidgetDataStore.loadConfig(appGroup: group, widgetId: widgetId)
        XCTAssertEqual(cfg?.small?.content, "new")
    }

    private func setEnv(_ key: String, _ value: String) {
        if previousEnv[key] == nil {
            previousEnv[key] = getenv(key).map { String(cString: $0) }
        }
        setenv(key, value, 1)
    }

    private func writeMap(_ map: [String: String], to url: URL) throws {
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        let data = try JSONSerialization.data(withJSONObject: map, options: [.prettyPrinted])
        try data.write(to: url, options: .atomic)
    }
}
