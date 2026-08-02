import XCTest
import SwiftUI
@testable import TauriWidgets
#if canImport(AppKit)
import AppKit
#endif

/// Level-2 visual stand: declarative cases → SwiftUI chrome → PNG golden assert.
/// Not SpringBoard — WidgetKit chrome (background + corner mask) is simulated.
final class RenderTests: XCTestCase {
    private static let precision: Double = 0.98

    func testCases() async throws {
        let filter = ProcessInfo.processInfo.environment["CASE"]
        let cases = try VisualCase.loadAll(casesRoot: RepoPaths.cases, filter: filter)
        XCTAssertFalse(cases.isEmpty, "no cases under \(RepoPaths.cases.path)")

        let record =
            ProcessInfo.processInfo.environment["GOLDEN_RECORD"] == "1"
            || ProcessInfo.processInfo.environment["GOLDEN_RECORD"] == "true"
            || ProcessInfo.processInfo.environment["UPDATE_SNAPSHOTS"] == "1"
        let recordAll = ProcessInfo.processInfo.environment["GOLDEN_RECORD_ALL"] == "1"

        if record && !recordAll {
            XCTAssertNotNil(filter, "GOLDEN_RECORD requires CASE=<name> (one case at a time)")
        }

        var failures: [String] = []
        for c in cases {
            do {
                try await runCase(c, record: record)
            } catch {
                failures.append("\(c.name): \(error.localizedDescription)")
            }
        }
        XCTAssertTrue(failures.isEmpty, failures.joined(separator: "\n"))
    }

    func testNullFieldsNeverRenderLiteralNull() async throws {
        let cases = try VisualCase.loadAll(casesRoot: RepoPaths.cases, filter: "null-fields.small")
        guard let c = cases.first else {
            return XCTFail("null-fields.small case missing")
        }
        let cfg = try c.config(fixturesRoot: RepoPaths.fixtures)
        guard let el = c.layout(from: cfg) else {
            return XCTFail("null-fields missing layout")
        }
        let image = try await MainActor.run {
            try Self.renderImage(case: c, element: el)
        }
        try PixelCompare.assertNonUniform(image: image, caseName: c.name)
    }

    private func runCase(_ c: VisualCase, record: Bool) async throws {
        let cfg = try c.config(fixturesRoot: RepoPaths.fixtures)
        guard let el = c.layout(from: cfg) else {
            throw NSError(
                domain: "TauriWidgetsTests",
                code: 11,
                userInfo: [NSLocalizedDescriptionKey: "no layout for size \(c.size)"],
            )
        }
        let image = try await MainActor.run {
            try Self.renderImage(case: c, element: el)
        }
        try PixelCompare.assertNonUniform(image: image, caseName: c.name)

        let goldenURL = RepoPaths.goldenIos.appendingPathComponent("\(c.name).png")
        if record {
            try PixelCompare.writePNG(image, to: goldenURL)
            return
        }
        guard FileManager.default.fileExists(atPath: goldenURL.path) else {
            throw NSError(
                domain: "TauriWidgetsTests",
                code: 12,
                userInfo: [
                    NSLocalizedDescriptionKey:
                        "missing golden \(goldenURL.path) — GOLDEN_RECORD=1 CASE=\(c.name) swift test",
                ],
            )
        }
        let expected = try PixelCompare.loadPNG(goldenURL)
        let match = PixelCompare.similarity(expected, image)
        if match < Self.precision {
            let outDir = RepoPaths.outIos
            try FileManager.default.createDirectory(at: outDir, withIntermediateDirectories: true)
            try PixelCompare.writePNG(image, to: outDir.appendingPathComponent("\(c.name).actual.png"))
            throw NSError(
                domain: "TauriWidgetsTests",
                code: 13,
                userInfo: [
                    NSLocalizedDescriptionKey: String(
                        format: "golden mismatch \(c.name): similarity %.4f < %.2f (actual → \(outDir.path))",
                        match,
                        Self.precision,
                    ),
                ],
            )
        }
    }

    @MainActor
    private static func renderImage(case c: VisualCase, element: WidgetElement) throws -> CGImage {
        let scheme: ColorScheme = c.theme == "light" ? .light : .dark
        let view = WidgetChrome.framed(
            element: element,
            width: c.points.width,
            height: c.points.height,
        )
        .environment(\.colorScheme, scheme)
        .environment(\.locale, Locale(identifier: c.locale.replacingOccurrences(of: "_", with: "-")))
        .environment(\.sizeCategory, .large)

        let renderer = ImageRenderer(content: view)
        renderer.scale = 2
        guard let image = renderer.cgImage else {
            throw NSError(
                domain: "TauriWidgetsTests",
                code: 1,
                userInfo: [NSLocalizedDescriptionKey: "ImageRenderer returned nil"],
            )
        }
        return image
    }
}
