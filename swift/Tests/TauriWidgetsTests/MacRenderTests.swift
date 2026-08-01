#if os(macOS)
import XCTest
import SwiftUI
import AppKit
@testable import TauriWidgets

/// Level A — native macOS AppKit path (no simulator).
/// Covers `#elseif canImport(AppKit)` branches: Color.adaptive / semantic, NSImage decode.
final class MacRenderTests: XCTestCase {
    private static let precision: Double = 0.98
    private static let renderScale: CGFloat = 2

    func testCases() throws {
        let filter = ProcessInfo.processInfo.environment["CASE"]
        let cases = try VisualCase.loadAll(casesRoot: RepoPaths.cases, filter: filter)
        XCTAssertFalse(cases.isEmpty, "no cases under \(RepoPaths.cases.path)")

        let record =
            ProcessInfo.processInfo.environment["GOLDEN_RECORD"] == "1"
            || ProcessInfo.processInfo.environment["GOLDEN_RECORD"] == "true"
        let recordAll = ProcessInfo.processInfo.environment["GOLDEN_RECORD_ALL"] == "1"

        if record && !recordAll {
            XCTAssertNotNil(filter, "GOLDEN_RECORD requires CASE=<name> (one case at a time)")
        }

        var failures: [String] = []
        for c in cases {
            do {
                try runCase(c, record: record)
            } catch {
                failures.append("\(c.name): \(error.localizedDescription)")
            }
        }
        XCTAssertTrue(failures.isEmpty, failures.joined(separator: "\n"))
    }

    private func runCase(_ c: VisualCase, record: Bool) throws {
        let cfg = try c.config(fixturesRoot: RepoPaths.fixtures)
        guard let el = c.layout(from: cfg) else {
            throw NSError(
                domain: "TauriWidgetsTests",
                code: 21,
                userInfo: [NSLocalizedDescriptionKey: "no layout for size \(c.size)"],
            )
        }

        let image = try Self.renderAppKit(case: c, element: el)
        try PixelCompare.assertNonUniform(image: image, caseName: c.name)

        let goldenURL = RepoPaths.goldenMacos.appendingPathComponent("\(c.name).png")
        if record {
            try PixelCompare.writePNG(image, to: goldenURL)
            return
        }
        guard FileManager.default.fileExists(atPath: goldenURL.path) else {
            throw NSError(
                domain: "TauriWidgetsTests",
                code: 22,
                userInfo: [
                    NSLocalizedDescriptionKey:
                        "missing golden \(goldenURL.path) — CASE=\(c.name) GOLDEN_RECORD=1 swift test --filter MacRenderTests",
                ],
            )
        }
        let expected = try PixelCompare.loadPNG(goldenURL)
        let match = PixelCompare.similarity(expected, image)
        if match < Self.precision {
            let outDir = RepoPaths.outMacos
            try FileManager.default.createDirectory(at: outDir, withIntermediateDirectories: true)
            try PixelCompare.writePNG(image, to: outDir.appendingPathComponent("\(c.name).actual.png"))
            throw NSError(
                domain: "TauriWidgetsTests",
                code: 23,
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

    /// Render through NSHostingView at fixed 2× RGBA so AppKit paths stay deterministic.
    private static func renderAppKit(case c: VisualCase, element: WidgetElement) throws -> CGImage {
        let scheme: ColorScheme = c.theme == "light" ? .light : .dark
        let size = c.macosPoints.cgSize
        let scale = renderScale
        let pxW = Int((size.width * scale).rounded())
        let pxH = Int((size.height * scale).rounded())

        let root = DynamicElementView(element: element)
            .frame(width: size.width, height: size.height)
            .environment(\.colorScheme, scheme)
            .environment(\.locale, Locale(identifier: c.locale.replacingOccurrences(of: "_", with: "-")))
            .environment(\.sizeCategory, .large)

        let hosting = NSHostingView(rootView: root)
        hosting.frame = NSRect(origin: .zero, size: size)
        hosting.appearance = NSAppearance(named: scheme == .dark ? .darkAqua : .aqua)
        hosting.layoutSubtreeIfNeeded()

        guard let rep = NSBitmapImageRep(
            bitmapDataPlanes: nil,
            pixelsWide: pxW,
            pixelsHigh: pxH,
            bitsPerSample: 8,
            samplesPerPixel: 4,
            hasAlpha: true,
            isPlanar: false,
            colorSpaceName: .deviceRGB,
            bytesPerRow: 0,
            bitsPerPixel: 0,
        ) else {
            throw NSError(domain: "TauriWidgetsTests", code: 24)
        }
        rep.size = size
        hosting.cacheDisplay(in: hosting.bounds, to: rep)
        guard let image = rep.cgImage else {
            throw NSError(domain: "TauriWidgetsTests", code: 25)
        }
        return image
    }
}
#endif
