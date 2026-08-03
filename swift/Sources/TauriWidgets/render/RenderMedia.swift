import SwiftUI

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
            let img = Image(systemName: sn).resizable()
                .aspectRatio(contentMode: element.contentMode == "fill" ? .fill : .fit)
                .frame(width: s, height: s)
            if let c = resolveColor(element.color) { img.foregroundColor(c) } else { img }
        } else if let decoded = decodeImageData(element.data) {
            decodedImageView(decoded, size: s)
        } else if let url = element.url, let decoded = decodeImageData(url) {
            decodedImageView(decoded, size: s)
        } else {
            placeholderImage(s)
        }
    }

    func decodeImageData(_ raw: String?) -> Data? {
        guard var s = raw?.trimmingCharacters(in: .whitespacesAndNewlines), !s.isEmpty else { return nil }
        if s.hasPrefix("data:"), let comma = s.firstIndex(of: ",") {
            s = String(s[s.index(after: comma)...])
        }
        return Data(base64Encoded: s)
    }

    @ViewBuilder func decodedImageView(_ raw: Data, size: CGFloat) -> some View {
        #if canImport(UIKit)
        if let uiImage = UIImage(data: raw) {
            Image(uiImage: uiImage).resizable()
                .aspectRatio(contentMode: element.contentMode == "fill" ? .fill : .fit)
                .frame(width: size, height: size)
        } else { placeholderImage(size) }
        #elseif canImport(AppKit)
        if let nsImage = NSImage(data: raw) {
            Image(nsImage: nsImage).resizable()
                .aspectRatio(contentMode: element.contentMode == "fill" ? .fill : .fit)
                .frame(width: size, height: size)
        } else { placeholderImage(size) }
        #endif
    }

    @ViewBuilder func placeholderImage(_ s: CGFloat) -> some View {
        Image(systemName: "questionmark.square.dashed").resizable()
            .frame(width: s, height: s).foregroundColor(.gray)
    }

    // MARK: Shape

    @ViewBuilder func renderShape() -> some View {
        let s = element.size ?? 24
        let fc = resolveColor(element.fill) ?? Color.accentColor
        let sc = resolveColor(element.stroke); let sw = element.strokeWidth ?? 1
        switch element.shapeType {
        case "circle":
            ZStack {
                Circle().fill(fc).frame(width: s, height: s)
                if let c = sc { Circle().stroke(c, lineWidth: sw).frame(width: s, height: s) }
            }
        case "capsule":
            ZStack {
                Capsule().fill(fc).frame(width: s * 2, height: s)
                if let c = sc { Capsule().stroke(c, lineWidth: sw).frame(width: s * 2, height: s) }
            }
        default:
            let cr = element.cornerRadius ?? 0
            ZStack {
                RoundedRectangle(cornerRadius: cr).fill(fc).frame(width: s, height: s)
                if let c = sc { RoundedRectangle(cornerRadius: cr).stroke(c, lineWidth: sw).frame(width: s, height: s) }
            }
        }
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
