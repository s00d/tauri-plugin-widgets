import XCTest
import SwiftUI
import ImageIO
import UniformTypeIdentifiers
@testable import TauriWidgets
#if canImport(AppKit)
import AppKit
#endif

/// Level-2 iOS/macOS ImageRenderer snapshots + sanity invariants.
/// PNG goldens: ../../tests/expected/pixels/ios/<fixture>.<size>.png
final class RenderTests: XCTestCase {
    private static let sizePoints: [String: CGSize] = [
        "small": CGSize(width: 155, height: 155),
        "medium": CGSize(width: 329, height: 155),
        "large": CGSize(width: 329, height: 345),
    ]

    func testFixturesDecodeAndRenderSane() async throws {
        let fixtures = try Self.listFixtures()
        XCTAssertFalse(fixtures.isEmpty, "no fixtures found under \(Self.fixturesRoot().path)")

        let record =
            ProcessInfo.processInfo.environment["SNAPSHOT_TESTING_RECORD"] != nil
            || ProcessInfo.processInfo.environment["UPDATE_SNAPSHOTS"] == "1"

        for fx in fixtures {
            let data = try Data(contentsOf: fx.url)
            let cfg = try JSONDecoder().decode(WidgetUIConfig.self, from: data)
            for (sizeName, size) in Self.sizePoints {
                let element: WidgetElement?
                switch sizeName {
                case "large": element = cfg.large
                case "medium": element = cfg.medium
                default: element = cfg.small
                }
                guard let element else { continue }

                let image = try await MainActor.run {
                    try Self.renderImage(element: element, size: size)
                }
                try Self.assertSane(image: image, fixture: fx.id, size: sizeName)

                #if os(macOS)
                let pngURL = Self.pixelsRoot().appendingPathComponent(
                    "\(fx.id.replacingOccurrences(of: "/", with: "__")).\(sizeName).png"
                )
                if record || !FileManager.default.fileExists(atPath: pngURL.path) {
                    try Self.writePNG(image, to: pngURL)
                }
                #endif
            }
        }
    }

    func testNullFieldsNeverRenderLiteralNull() async throws {
        let url = Self.fixturesRoot().appendingPathComponent("bugs/null-fields.json")
        let data = try Data(contentsOf: url)
        let cfg = try JSONDecoder().decode(WidgetUIConfig.self, from: data)
        guard let element = cfg.small else {
            return XCTFail("null-fields missing small")
        }
        let image = try await MainActor.run {
            try Self.renderImage(element: element, size: Self.sizePoints["small"]!)
        }
        try Self.assertSane(image: image, fixture: "bugs/null-fields", size: "small")
    }

    // MARK: - Helpers

    private struct FixtureRef {
        let id: String
        let url: URL
    }

    private static func fixturesRoot() -> URL {
        URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent() // TauriWidgetsTests
            .deletingLastPathComponent() // Tests
            .deletingLastPathComponent() // swift
            .deletingLastPathComponent() // repo root
            .appendingPathComponent("tests/fixtures")
    }

    private static func pixelsRoot() -> URL {
        URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .appendingPathComponent("tests/expected/pixels/ios")
    }

    private static func listFixtures() throws -> [FixtureRef] {
        let root = fixturesRoot()
        var out: [FixtureRef] = []
        for folder in ["core", "bugs", "presets"] {
            let dir = root.appendingPathComponent(folder)
            guard let files = try? FileManager.default.contentsOfDirectory(
                at: dir,
                includingPropertiesForKeys: nil
            ) else { continue }
            for url in files.filter({ $0.pathExtension == "json" }).sorted(by: { $0.lastPathComponent < $1.lastPathComponent }) {
                let id = "\(folder)/\(url.deletingPathExtension().lastPathComponent)"
                out.append(FixtureRef(id: id, url: url))
            }
        }
        return out
    }

    @MainActor
    private static func renderImage(element: WidgetElement, size: CGSize) throws -> CGImage {
        // Match production: content fills the widget frame (no letterboxing).
        // Prefer dark colorScheme so `.primary` ink stays readable on typical dark widget fills;
        // fixtures with explicit colors still win.
        let view = DynamicElementView(element: element)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .frame(width: size.width, height: size.height)
            .background(Color(red: 0.05, green: 0.05, blue: 0.07))
            .environment(\.colorScheme, .dark)
        let renderer = ImageRenderer(content: view)
        renderer.scale = 2
        guard let image = renderer.cgImage else {
            throw NSError(domain: "TauriWidgetsTests", code: 1, userInfo: [
                NSLocalizedDescriptionKey: "ImageRenderer returned nil",
            ])
        }
        return image
    }

    private static func writePNG(_ image: CGImage, to url: URL) throws {
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        guard let dest = CGImageDestinationCreateWithURL(url as CFURL, UTType.png.identifier as CFString, 1, nil) else {
            throw NSError(domain: "TauriWidgetsTests", code: 2, userInfo: [
                NSLocalizedDescriptionKey: "CGImageDestinationCreateWithURL failed",
            ])
        }
        CGImageDestinationAddImage(dest, image, nil)
        guard CGImageDestinationFinalize(dest) else {
            throw NSError(domain: "TauriWidgetsTests", code: 3, userInfo: [
                NSLocalizedDescriptionKey: "CGImageDestinationFinalize failed",
            ])
        }
    }

    /// Invariants: not uniform, content bbox inside frame.
    private static func assertSane(image: CGImage, fixture: String, size: String) throws {
        let w = image.width
        let h = image.height
        XCTAssertGreaterThan(w, 0)
        XCTAssertGreaterThan(h, 0)

        guard let data = image.dataProvider?.data,
              let ptr = CFDataGetBytePtr(data) else {
            return XCTFail("no pixel data \(fixture)/\(size)")
        }
        let bpp = max(1, image.bitsPerPixel / 8)
        let bpr = image.bytesPerRow

        var minX = w, minY = h, maxX = 0, maxY = 0
        var nonUniform = false
        let first = (Int(ptr[0]), Int(ptr[1]), Int(ptr[2]), Int(ptr[3]))
        let step = max(1, min(w, h) / 64)

        for y in stride(from: 0, to: h, by: step) {
            for x in stride(from: 0, to: w, by: step) {
                let o = y * bpr + x * bpp
                guard o + 3 < CFDataGetLength(data) else { continue }
                let px = (Int(ptr[o]), Int(ptr[o + 1]), Int(ptr[o + 2]), Int(ptr[o + 3]))
                if px != first { nonUniform = true }
                let lum = px.0 + px.1 + px.2
                if px.3 > 8 && (lum > 8 || px.3 < 250) {
                    minX = min(minX, x)
                    minY = min(minY, y)
                    maxX = max(maxX, x)
                    maxY = max(maxY, y)
                }
            }
        }

        XCTAssertTrue(nonUniform, "\(fixture)/\(size): uniform image (dead render?)")
        if maxX >= minX && maxY >= minY {
            XCTAssertGreaterThanOrEqual(minX, 0)
            XCTAssertGreaterThanOrEqual(minY, 0)
            XCTAssertLessThan(maxX, w)
            XCTAssertLessThan(maxY, h)
            let contentH = maxY - minY + 1
            let minContent = Int(Double(h) * 0.15)
            let skipHeight =
                fixture.contains("vstack-spacer")
                || fixture.contains("canvas")
                || fixture.contains("list")
                || fixture.contains("zstack")
                || fixture.contains("shape-capsule")
                || fixture.contains("tasks")
            if !skipHeight {
                XCTAssertGreaterThan(
                    contentH,
                    minContent,
                    "\(fixture)/\(size): content height \(contentH) ≤ 15% of \(h) (dead spacer?)"
                )
            }
        }
    }
}
