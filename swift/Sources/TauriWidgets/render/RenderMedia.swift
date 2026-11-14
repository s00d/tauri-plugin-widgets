import SwiftUI
import ImageIO

// MARK: - Image / Shape / Canvas

extension DynamicElementView {
    @ViewBuilder func renderMedia() -> some View {
        switch element.type {
        case "image":   renderImage()
        case "shape":   renderShape()
        case "canvas":  renderCanvas()
        default: EmptyView()
        }
    }

    // MARK: Image

    @ViewBuilder func renderImage() -> some View {
        let s = element.size ?? 24
        if let sn = element.systemName {
            symbolImage(sn, size: s)
        } else if let decoded = decodeImageData(element.data) {
            decodedImageView(decoded, size: s)
        } else if let url = element.url, let decoded = decodeImageData(url) {
            decodedImageView(decoded, size: s)
        } else if let fb = element.fallbackSystemName {
            symbolImage(fb, size: s)
        } else {
            placeholderImage(s)
        }
    }

    /// `contentMode`: fit (default) / fill keep the ratio, stretch ignores it, center draws
    /// at natural size, unscaled and clipped to the box.
    @ViewBuilder func sized(_ img: Image, size s: CGFloat) -> some View {
        switch element.contentMode {
        case "center": img.frame(width: s, height: s).clipped()
        case "stretch": img.resizable().frame(width: s, height: s)
        case "fill": img.resizable().aspectRatio(contentMode: .fill).frame(width: s, height: s)
        default: img.resizable().aspectRatio(contentMode: .fit).frame(width: s, height: s)
        }
    }

    /// SF Symbol with `color`, `symbolRendering` (+ `paletteColors`) and `symbolWeight`.
    @ViewBuilder func symbolImage(_ name: String, size s: CGFloat) -> some View {
        let base = sized(Image(systemName: name), size: s)
            .ifLet(element.symbolWeight) { $0.fontWeight(fontWeight($1)) }
        let palette = (element.paletteColors ?? []).compactMap { resolveColor($0) }
        switch element.symbolRendering {
        case "palette" where !palette.isEmpty:
            let p2 = palette.count > 1 ? palette[1] : palette[0]
            let p3 = palette.count > 2 ? palette[2] : p2
            base.symbolRenderingMode(.palette).foregroundStyle(palette[0], p2, p3)
        case "multicolor":
            base.symbolRenderingMode(.multicolor)
        case "hierarchical":
            base.symbolRenderingMode(.hierarchical)
                .foregroundStyle(resolveColor(element.color) ?? .primary)
        case "monochrome":
            base.symbolRenderingMode(.monochrome)
                .ifLet(resolveColor(element.color)) { $0.foregroundColor($1) }
        default:
            if let c = resolveColor(element.color) { base.foregroundColor(c) } else { base }
        }
    }

    func decodeImageData(_ raw: String?) -> Data? {
        guard var s = raw?.trimmingCharacters(in: .whitespacesAndNewlines), !s.isEmpty else { return nil }
        if s.hasPrefix("data:"), let comma = s.firstIndex(of: ",") {
            s = String(s[s.index(after: comma)...])
        }
        return Data(base64Encoded: s)
    }

    /// Decode at most ~2× the widget point size: a full-resolution photo (12 MP ≈ 48 MB)
    /// makes WidgetKit reject the archive or kills the extension.
    func downsampledCGImage(_ raw: Data, size: CGFloat) -> CGImage? {
        let options = [kCGImageSourceShouldCache: false] as CFDictionary
        guard let source = CGImageSourceCreateWithData(raw as CFData, options) else { return nil }
        let thumb = [
            kCGImageSourceCreateThumbnailFromImageAlways: true,
            kCGImageSourceCreateThumbnailWithTransform: true,
            kCGImageSourceShouldCacheImmediately: true,
            kCGImageSourceThumbnailMaxPixelSize: max(64, min(1024, Int(size * 3))),
        ] as CFDictionary
        return CGImageSourceCreateThumbnailAtIndex(source, 0, thumb)
    }

    @ViewBuilder func decodedImageView(_ raw: Data, size: CGFloat) -> some View {
        #if canImport(UIKit)
        if let cg = downsampledCGImage(raw, size: size) {
            sized(Image(uiImage: UIImage(cgImage: cg)), size: size)
        } else if let uiImage = UIImage(data: raw) {
            sized(Image(uiImage: uiImage), size: size)
        } else if let fb = element.fallbackSystemName {
            symbolImage(fb, size: size)
        } else { placeholderImage(size) }
        #elseif canImport(AppKit)
        if let cg = downsampledCGImage(raw, size: size) {
            sized(Image(nsImage: NSImage(cgImage: cg, size: .zero)), size: size)
        } else if let nsImage = NSImage(data: raw) {
            sized(Image(nsImage: nsImage), size: size)
        } else if let fb = element.fallbackSystemName {
            symbolImage(fb, size: size)
        } else { placeholderImage(size) }
        #endif
    }

    @ViewBuilder func placeholderImage(_ s: CGFloat) -> some View {
        Image(systemName: "questionmark.square.dashed").resizable()
            .frame(width: s, height: s).foregroundColor(.gray)
    }

    // MARK: Shape

    @ViewBuilder func renderShape() -> some View {
        let fc = resolveColor(element.fill) ?? Color.accentColor
        let sc = resolveColor(element.stroke); let sw = element.strokeWidth ?? 1
        // Shapes have no intrinsic size: `minWidth`/`minHeight` seed a missing side and
        // `aspectRatio` derives the other (same rule as Android `frameSizeDp`).
        let (frameW, frameH) = Self.shapeFrame(element)
        let explicitSize = element.size
        switch element.shapeType {
        case "circle":
            let s = explicitSize ?? frameW ?? frameH ?? 24
            ZStack {
                Circle().fill(fc).frame(width: s, height: s)
                if let c = sc { Circle().stroke(c, lineWidth: sw).frame(width: s, height: s) }
            }
        case "ellipse":
            // size → circle-sized; frame w/h → oval; nothing → fill the slot.
            if explicitSize == nil && frameW == nil && frameH == nil {
                ZStack {
                    Ellipse().fill(fc)
                    if let c = sc { Ellipse().stroke(c, lineWidth: sw) }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                let w = frameW ?? explicitSize ?? frameH ?? 24
                let h = frameH ?? explicitSize ?? frameW ?? 24
                ZStack {
                    Ellipse().fill(fc).frame(width: w, height: h)
                    if let c = sc { Ellipse().stroke(c, lineWidth: sw).frame(width: w, height: h) }
                }
            }
        case "capsule":
            // Prefer explicit frame (accent bar 6×36). size-only keeps width=2*size contract.
            let capsuleSize: (CGFloat, CGFloat) = {
                if let fw = frameW, let fh = frameH { return (fw, fh) }
                if let fw = frameW { return (fw, explicitSize ?? fw) }
                if let fh = frameH { return (explicitSize ?? fh, fh) }
                let s = explicitSize ?? 24
                return (s * 2, s)
            }()
            ZStack {
                Capsule().fill(fc).frame(width: capsuleSize.0, height: capsuleSize.1)
                if let c = sc {
                    Capsule().stroke(c, lineWidth: sw).frame(width: capsuleSize.0, height: capsuleSize.1)
                }
            }
        default:
            // rectangle / roundedRectangle: both honor `cornerRadius`.
            let cr = element.cornerRadius ?? 0
            // No size/frame → fill proposed size (zstack underlay). Otherwise hug.
            if explicitSize == nil && frameW == nil && frameH == nil {
                ZStack {
                    RoundedRectangle(cornerRadius: cr).fill(fc)
                    if let c = sc {
                        RoundedRectangle(cornerRadius: cr).stroke(c, lineWidth: sw)
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                let s = explicitSize ?? min(frameW ?? 24, frameH ?? 24)
                let w = frameW ?? s
                let h = frameH ?? s
                ZStack {
                    RoundedRectangle(cornerRadius: cr).fill(fc).frame(width: w, height: h)
                    if let c = sc {
                        RoundedRectangle(cornerRadius: cr).stroke(c, lineWidth: sw).frame(width: w, height: h)
                    }
                }
            }
        }
    }

    static func shapeFrame(_ el: WidgetElement) -> (CGFloat?, CGFloat?) {
        var w = el.frame?.width ?? el.frame?.minWidth
        var h = el.frame?.height ?? el.frame?.minHeight
        if let r = el.aspectRatio, r > 0 {
            if let fw = w, h == nil { h = fw / r } else if let fh = h, w == nil { w = fh * r }
        }
        return (w, h)
    }

    // MARK: Canvas

    @ViewBuilder func renderCanvas() -> some View {
        let cw = max(element.width ?? 100, 1)
        let ch = max(element.height ?? 100, 1)
        let commands = element.elements ?? []
        // Uniform scale + letterbox — independent sx/sy stretches circles into ovals
        // and splits clock faces when the view is flex-resized (Analog Clock large).
        let canvasView = SwiftUI.Canvas { context, size in
            let s = Swift.min(size.width / cw, size.height / ch)
            let ox = (size.width - cw * s) / 2
            let oy = (size.height - ch * s) / 2
            func X(_ v: CGFloat) -> CGFloat { ox + v * s }
            func Y(_ v: CGFloat) -> CGFloat { oy + v * s }
            for cmd in commands {
                switch cmd.draw {
                case "circle":
                    let cx = X(cmd.cx ?? 0); let cy = Y(cmd.cy ?? 0); let r = (cmd.r ?? 10) * s
                    let rect = CGRect(x: cx - r, y: cy - r, width: r * 2, height: r * 2)
                    if let f = resolveColor(cmd.fill) { context.fill(SwiftUI.Path(ellipseIn: rect), with: .color(f)) }
                    if let st = resolveColor(cmd.stroke) {
                        context.stroke(SwiftUI.Path(ellipseIn: rect), with: .color(st), lineWidth: (cmd.strokeWidth ?? 1) * s)
                    }
                case "line":
                    var path = SwiftUI.Path()
                    path.move(to: CGPoint(x: X(cmd.x1 ?? 0), y: Y(cmd.y1 ?? 0)))
                    path.addLine(to: CGPoint(x: X(cmd.x2 ?? 0), y: Y(cmd.y2 ?? 0)))
                    var style = StrokeStyle(lineWidth: (cmd.strokeWidth ?? 1) * s)
                    if cmd.lineCap == "round" { style.lineCap = .round }
                    context.stroke(path, with: .color(resolveColor(cmd.stroke) ?? Color(hex: "#ffffff")), style: style)
                case "rect":
                    let rect = CGRect(
                        x: X(cmd.x ?? 0), y: Y(cmd.y ?? 0),
                        width: (cmd.width ?? 10) * s, height: (cmd.height ?? 10) * s
                    )
                    let cr = (cmd.cornerRadius ?? 0) * s
                    let p = SwiftUI.Path(roundedRect: rect, cornerRadius: cr)
                    if let f = resolveColor(cmd.fill) { context.fill(p, with: .color(f)) }
                    if let st = resolveColor(cmd.stroke) {
                        context.stroke(p, with: .color(st), lineWidth: (cmd.strokeWidth ?? 1) * s)
                    }
                case "arc":
                    let cx = X(cmd.cx ?? 0); let cy = Y(cmd.cy ?? 0); let r = (cmd.r ?? 10) * s
                    var path = SwiftUI.Path()
                    if cmd.fill != nil { path.move(to: CGPoint(x: cx, y: cy)) }
                    path.addArc(
                        center: CGPoint(x: cx, y: cy), radius: r,
                        startAngle: .degrees(Double(cmd.startAngle ?? 0)),
                        endAngle: .degrees(Double(cmd.endAngle ?? 360)), clockwise: false
                    )
                    if cmd.fill != nil { path.closeSubpath() }
                    if let f = resolveColor(cmd.fill) { context.fill(path, with: .color(f)) }
                    if let st = resolveColor(cmd.stroke) {
                        context.stroke(path, with: .color(st), lineWidth: (cmd.strokeWidth ?? 1) * s)
                    }
                case "text":
                    let fs = (cmd.fontSize ?? 12) * s
                    let txt = SwiftUI.Text(cmd.content ?? "").font(.system(size: fs))
                        .foregroundColor(resolveColor(cmd.color) ?? .white)
                    let pt = CGPoint(x: X(cmd.x ?? 0), y: Y(cmd.y ?? 0))
                    let anchor: UnitPoint = cmd.anchor == "end" ? .trailing : cmd.anchor == "middle" ? .center : .leading
                    context.draw(context.resolve(txt), at: pt, anchor: anchor)
                case "path":
                    if let d = cmd.d {
                        var path = parseSVGPath(d)
                        path = path.applying(
                            CGAffineTransform(a: s, b: 0, c: 0, d: s, tx: ox, ty: oy)
                        )
                        if let f = resolveColor(cmd.fill) { context.fill(path, with: .color(f)) }
                        if let st = resolveColor(cmd.stroke) {
                            context.stroke(path, with: .color(st), lineWidth: (cmd.strokeWidth ?? 1) * s)
                        }
                    }
                default: break
                }
            }
        }
        // Fit into offered space; never force a fixed pt size larger than the family.
        canvasView
            .aspectRatio(cw / ch, contentMode: .fit)
            .frame(maxWidth: cw, maxHeight: ch)
            .frame(maxWidth: (element.flex ?? 0) > 0 ? .infinity : nil,
                   maxHeight: (element.flex ?? 0) > 0 ? .infinity : nil)
    }
}
