import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

enum PixelCompare {
    static func writePNG(_ image: CGImage, to url: URL) throws {
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true,
        )
        guard let dest = CGImageDestinationCreateWithURL(
            url as CFURL,
            UTType.png.identifier as CFString,
            1,
            nil,
        ) else {
            throw NSError(domain: "TauriWidgetsTests", code: 2)
        }
        CGImageDestinationAddImage(dest, image, nil)
        guard CGImageDestinationFinalize(dest) else {
            throw NSError(domain: "TauriWidgetsTests", code: 3)
        }
    }

    static func loadPNG(_ url: URL) throws -> CGImage {
        guard let src = CGImageSourceCreateWithURL(url as CFURL, nil),
              let img = CGImageSourceCreateImageAtIndex(src, 0, nil)
        else {
            throw NSError(
                domain: "TauriWidgetsTests",
                code: 4,
                userInfo: [NSLocalizedDescriptionKey: "cannot load \(url.path)"],
            )
        }
        return img
    }

    /// Draw into a fixed RGBA8888 buffer so PNG round-trips / AppKit bitmaps compare fairly.
    static func rgba8(_ image: CGImage) -> (data: Data, width: Int, height: Int)? {
        let w = image.width
        let h = image.height
        guard w > 0, h > 0 else { return nil }
        var data = Data(count: w * h * 4)
        let ok = data.withUnsafeMutableBytes { raw -> Bool in
            guard let base = raw.bindMemory(to: UInt8.self).baseAddress else { return false }
            guard let ctx = CGContext(
                data: base,
                width: w,
                height: h,
                bitsPerComponent: 8,
                bytesPerRow: w * 4,
                space: CGColorSpaceCreateDeviceRGB(),
                bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue,
            ) else { return false }
            ctx.draw(image, in: CGRect(x: 0, y: 0, width: w, height: h))
            return true
        }
        return ok ? (data, w, h) : nil
    }

    /// Fraction of matching pixels (max channel delta ≤ threshold) after RGBA normalize.
    static func similarity(_ a: CGImage, _ b: CGImage, threshold: Int = 12) -> Double {
        guard let ra = rgba8(a), let rb = rgba8(b),
              ra.width == rb.width, ra.height == rb.height
        else { return 0 }
        let n = ra.width * ra.height
        if n == 0 { return 0 }
        var good = 0
        ra.data.withUnsafeBytes { pa in
            rb.data.withUnsafeBytes { pb in
                let aPtr = pa.bindMemory(to: UInt8.self)
                let bPtr = pb.bindMemory(to: UInt8.self)
                for i in 0..<n {
                    let o = i * 4
                    let dr = abs(Int(aPtr[o]) - Int(bPtr[o]))
                    let dg = abs(Int(aPtr[o + 1]) - Int(bPtr[o + 1]))
                    let db = abs(Int(aPtr[o + 2]) - Int(bPtr[o + 2]))
                    if max(dr, dg, db) <= threshold { good += 1 }
                }
            }
        }
        return Double(good) / Double(n)
    }

    static func assertNonUniform(image: CGImage, caseName: String) throws {
        guard let rgba = rgba8(image) else {
            throw NSError(domain: "TauriWidgetsTests", code: 5)
        }
        let bytes = [UInt8](rgba.data)
        guard bytes.count >= 4 else {
            throw NSError(domain: "TauriWidgetsTests", code: 5)
        }
        let first = (bytes[0], bytes[1], bytes[2], bytes[3])
        let step = max(1, min(rgba.width, rgba.height) / 64)
        for y in stride(from: 0, to: rgba.height, by: step) {
            for x in stride(from: 0, to: rgba.width, by: step) {
                let o = (y * rgba.width + x) * 4
                let px = (bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3])
                if px != first { return }
            }
        }
        throw NSError(
            domain: "TauriWidgetsTests",
            code: 6,
            userInfo: [NSLocalizedDescriptionKey: "\(caseName): uniform image"],
        )
    }
}
