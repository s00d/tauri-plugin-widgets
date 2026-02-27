import XCTest

@testable import tauri_plugin_widgets

// MARK: - Decodable Args Tests

final class DecodableArgsTests: XCTestCase {

    // MARK: SetItemsArgs

    func testSetItemsArgsDecoding() throws {
        let json = """
        {"key": "widget_text", "value": "Hello", "group": "group.com.example"}
        """.data(using: .utf8)!

        let args = try JSONDecoder().decode(SetItemsArgs.self, from: json)
        XCTAssertEqual(args.key, "widget_text")
        XCTAssertEqual(args.value, "Hello")
        XCTAssertEqual(args.group, "group.com.example")
    }

    func testSetItemsArgsDecodingWithEmptyValue() throws {
        let json = """
        {"key": "k", "value": "", "group": "g"}
        """.data(using: .utf8)!

        let args = try JSONDecoder().decode(SetItemsArgs.self, from: json)
        XCTAssertEqual(args.value, "")
    }

    func testSetItemsArgsMissingFieldThrows() {
        let json = """
        {"key": "k", "value": "v"}
        """.data(using: .utf8)!

        XCTAssertThrowsError(try JSONDecoder().decode(SetItemsArgs.self, from: json))
    }

    // MARK: GetItemsArgs

    func testGetItemsArgsDecoding() throws {
        let json = """
        {"key": "title", "group": "group.app"}
        """.data(using: .utf8)!

        let args = try JSONDecoder().decode(GetItemsArgs.self, from: json)
        XCTAssertEqual(args.key, "title")
        XCTAssertEqual(args.group, "group.app")
    }

    func testGetItemsArgsMissingKeyThrows() {
        let json = """
        {"group": "g"}
        """.data(using: .utf8)!

        XCTAssertThrowsError(try JSONDecoder().decode(GetItemsArgs.self, from: json))
    }

    // MARK: SetRegisterWidgetArgs

    func testSetRegisterWidgetArgsDecoding() throws {
        let json = """
        {"widgets": ["com.example.Widget1", "com.example.Widget2"]}
        """.data(using: .utf8)!

        let args = try JSONDecoder().decode(SetRegisterWidgetArgs.self, from: json)
        XCTAssertEqual(args.widgets.count, 2)
        XCTAssertEqual(args.widgets[0], "com.example.Widget1")
        XCTAssertEqual(args.widgets[1], "com.example.Widget2")
    }

    func testSetRegisterWidgetArgsEmptyArray() throws {
        let json = """
        {"widgets": []}
        """.data(using: .utf8)!

        let args = try JSONDecoder().decode(SetRegisterWidgetArgs.self, from: json)
        XCTAssertTrue(args.widgets.isEmpty)
    }

    // MARK: ReloadTimelinesArgs

    func testReloadTimelinesArgsDecoding() throws {
        let json = """
        {"ofKind": "MyWidgetKind"}
        """.data(using: .utf8)!

        let args = try JSONDecoder().decode(ReloadTimelinesArgs.self, from: json)
        XCTAssertEqual(args.ofKind, "MyWidgetKind")
    }

    func testReloadTimelinesArgsCamelCase() throws {
        let json = """
        {"ofKind": "com.example.MyWidget"}
        """.data(using: .utf8)!

        let args = try JSONDecoder().decode(ReloadTimelinesArgs.self, from: json)
        XCTAssertEqual(args.ofKind, "com.example.MyWidget")
    }

    func testReloadTimelinesArgsSnakeCaseFails() {
        let json = """
        {"of_kind": "test"}
        """.data(using: .utf8)!

        XCTAssertThrowsError(try JSONDecoder().decode(ReloadTimelinesArgs.self, from: json))
    }
}

// MARK: - UserDefaults Integration Tests

final class UserDefaultsIntegrationTests: XCTestCase {

    private let testSuite = "test.tauri.plugin.widgets.\(UUID().uuidString)"

    override func tearDown() {
        super.tearDown()
        UserDefaults.standard.removePersistentDomain(forName: testSuite)
    }

    func testSetAndGetString() {
        guard let defaults = UserDefaults(suiteName: testSuite) else {
            XCTFail("Could not create UserDefaults for suite: \(testSuite)")
            return
        }

        defaults.set("Hello World", forKey: "widget_text")
        defaults.synchronize()

        let value = defaults.string(forKey: "widget_text")
        XCTAssertEqual(value, "Hello World")
    }

    func testGetMissingKeyReturnsNil() {
        guard let defaults = UserDefaults(suiteName: testSuite) else {
            XCTFail("Could not create UserDefaults for suite: \(testSuite)")
            return
        }

        let value = defaults.string(forKey: "nonexistent_key")
        XCTAssertNil(value)
    }

    func testOverwriteValue() {
        guard let defaults = UserDefaults(suiteName: testSuite) else {
            XCTFail("Could not create UserDefaults for suite: \(testSuite)")
            return
        }

        defaults.set("first", forKey: "key")
        defaults.synchronize()
        XCTAssertEqual(defaults.string(forKey: "key"), "first")

        defaults.set("second", forKey: "key")
        defaults.synchronize()
        XCTAssertEqual(defaults.string(forKey: "key"), "second")
    }

    func testMultipleKeys() {
        guard let defaults = UserDefaults(suiteName: testSuite) else {
            XCTFail("Could not create UserDefaults for suite: \(testSuite)")
            return
        }

        defaults.set("val1", forKey: "key1")
        defaults.set("val2", forKey: "key2")
        defaults.set("val3", forKey: "key3")
        defaults.synchronize()

        XCTAssertEqual(defaults.string(forKey: "key1"), "val1")
        XCTAssertEqual(defaults.string(forKey: "key2"), "val2")
        XCTAssertEqual(defaults.string(forKey: "key3"), "val3")
    }

    func testEmptyStringValue() {
        guard let defaults = UserDefaults(suiteName: testSuite) else {
            XCTFail("Could not create UserDefaults for suite: \(testSuite)")
            return
        }

        defaults.set("", forKey: "empty_key")
        defaults.synchronize()

        let value = defaults.string(forKey: "empty_key")
        XCTAssertEqual(value, "")
    }

    func testJsonStringValue() {
        guard let defaults = UserDefaults(suiteName: testSuite) else {
            XCTFail("Could not create UserDefaults for suite: \(testSuite)")
            return
        }

        let jsonString = "{\"title\":\"Hello\",\"count\":42}"
        defaults.set(jsonString, forKey: "json_data")
        defaults.synchronize()

        let value = defaults.string(forKey: "json_data")
        XCTAssertEqual(value, jsonString)
    }

    func testUnicodeValue() {
        guard let defaults = UserDefaults(suiteName: testSuite) else {
            XCTFail("Could not create UserDefaults for suite: \(testSuite)")
            return
        }

        let unicodeStr = "Привет мир 你好世界 🌍"
        defaults.set(unicodeStr, forKey: "unicode_key")
        defaults.synchronize()

        let value = defaults.string(forKey: "unicode_key")
        XCTAssertEqual(value, unicodeStr)
    }
}

// MARK: - Plugin Init Tests

final class PluginInitTests: XCTestCase {

    func testInitPluginReturnsWidgetPlugin() {
        let plugin = initPlugin()
        XCTAssertTrue(plugin is WidgetPlugin)
    }

    func testWidgetPluginCreation() {
        let plugin = WidgetPlugin()
        XCTAssertNotNil(plugin)
    }
}

// MARK: - Serialization Round-Trip Tests

final class SerializationRoundTripTests: XCTestCase {

    func testSetItemsRoundTrip() throws {
        let original: [String: Any] = [
            "key": "widget_text",
            "value": "Test Value",
            "group": "group.com.example"
        ]
        let data = try JSONSerialization.data(withJSONObject: original)
        let decoded = try JSONDecoder().decode(SetItemsArgs.self, from: data)

        XCTAssertEqual(decoded.key, "widget_text")
        XCTAssertEqual(decoded.value, "Test Value")
        XCTAssertEqual(decoded.group, "group.com.example")
    }

    func testReloadTimelinesRoundTripCamelCase() throws {
        let original: [String: Any] = ["ofKind": "MyWidget"]
        let data = try JSONSerialization.data(withJSONObject: original)
        let decoded = try JSONDecoder().decode(ReloadTimelinesArgs.self, from: data)

        XCTAssertEqual(decoded.ofKind, "MyWidget")
    }

    func testSetRegisterWidgetRoundTrip() throws {
        let widgets = ["com.a.W1", "com.b.W2", "com.c.W3"]
        let original: [String: Any] = ["widgets": widgets]
        let data = try JSONSerialization.data(withJSONObject: original)
        let decoded = try JSONDecoder().decode(SetRegisterWidgetArgs.self, from: data)

        XCTAssertEqual(decoded.widgets, widgets)
    }

    func testSpecialCharactersInValues() throws {
        let json = """
        {"key": "k/e.y-1", "value": "line1\\nline2\\ttab", "group": "group.test"}
        """.data(using: .utf8)!

        let args = try JSONDecoder().decode(SetItemsArgs.self, from: json)
        XCTAssertEqual(args.key, "k/e.y-1")
        XCTAssertEqual(args.value, "line1\nline2\ttab")
    }

    func testLongValues() throws {
        let longValue = String(repeating: "A", count: 10_000)
        let original: [String: Any] = [
            "key": "big",
            "value": longValue,
            "group": "g"
        ]
        let data = try JSONSerialization.data(withJSONObject: original)
        let decoded = try JSONDecoder().decode(SetItemsArgs.self, from: data)

        XCTAssertEqual(decoded.value.count, 10_000)
    }
}
