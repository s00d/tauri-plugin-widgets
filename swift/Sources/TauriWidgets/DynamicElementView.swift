import SwiftUI

// MARK: - Color helpers

extension Color {
    public init(hex: String) {
        var h = hex.trimmingCharacters(in: CharacterSet(charactersIn: "#"))
        if h.count == 3 || h.count == 4 {
            h = h.map { "\($0)\($0)" }.joined()
        }
        var rgb: UInt64 = 0; Scanner(string: h).scanHexInt64(&rgb)
        let r, g, b, a: Double
        switch h.count {
        case 6:
            r = Double((rgb >> 16) & 0xFF) / 255
            g = Double((rgb >> 8) & 0xFF) / 255
            b = Double(rgb & 0xFF) / 255; a = 1
        case 8:
            r = Double((rgb >> 24) & 0xFF) / 255
            g = Double((rgb >> 16) & 0xFF) / 255
            b = Double((rgb >> 8) & 0xFF) / 255
            a = Double(rgb & 0xFF) / 255
        default:
            r = 0; g = 0; b = 0; a = 1
        }
        self.init(red: r, green: g, blue: b, opacity: a)
    }

    static func adaptive(light: String, dark: String) -> Color {
        #if canImport(UIKit)
        return Color(UIColor { traits in
            traits.userInterfaceStyle == .dark
                ? UIColor(Color(hex: dark))
                : UIColor(Color(hex: light))
        })
        #elseif canImport(AppKit)
        return Color(NSColor(name: nil) { appearance in
            appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
                ? NSColor(Color(hex: dark))
                : NSColor(Color(hex: light))
        })
        #else
        return Color(hex: light)
        #endif
    }

    static func semantic(_ name: String) -> Color? {
        switch name {
        case "label": return .primary
        case "secondaryLabel": return .secondary
        case "accent": return .accentColor
        #if canImport(UIKit)
        case "systemBackground": return Color(UIColor.systemBackground)
        case "secondarySystemBackground": return Color(UIColor.secondarySystemBackground)
        case "separator": return Color(UIColor.separator)
        #elseif canImport(AppKit)
        case "systemBackground": return Color(NSColor.windowBackgroundColor)
        case "secondarySystemBackground": return Color(NSColor.controlBackgroundColor)
        case "separator": return Color(NSColor.separatorColor)
        #endif
        default: return nil
        }
    }
}

func resolveColor(_ cv: ColorValue?) -> Color? {
    guard let cv = cv else { return nil }
    switch cv {
    case .solid(let s):
        if let sem = Color.semantic(s) { return sem }
        return Color(hex: s)
    case .adaptive(let l, let d):
        return Color.adaptive(light: l, dark: d)
    }
}

// MARK: - Recursive Renderer

public struct DynamicElementView: View {
    public let element: WidgetElement

    public init(element: WidgetElement) { self.element = element }

    public var body: some View { applyStyle(to: renderElement(), element: element) }

    @ViewBuilder
    private func renderElement() -> some View {
        switch element.type {
        case "vstack":      renderVStack()
        case "hstack":      renderHStack()
        case "zstack":      renderZStack()
        case "grid":        renderGrid()
        case "container":   renderContainer()
        case "text":        renderText()
        case "image":       renderImage()
        case "progress":    renderProgress()
        case "gauge":       renderGauge()
        case "button":      renderButton()
        case "toggle":      renderToggle()
        case "divider":     renderDivider()
        case "spacer":      renderSpacer()
        case "date":        renderDate()
        case "chart":       renderChart()
        case "list":        renderList()
        case "link":        renderLink()
        case "shape":       renderShape()
        case "timer":       renderTimer()
        case "label":       renderLabel()
        case "canvas":      renderCanvas()
        default: EmptyView()
        }
    }

    // MARK: Containers

    @ViewBuilder private func renderVStack() -> some View {
        let align: HorizontalAlignment = element.alignment == "leading" ? .leading
            : element.alignment == "trailing" ? .trailing : .center
        VStack(alignment: align, spacing: element.spacing ?? 0) { renderChildren() }
    }

    @ViewBuilder private func renderHStack() -> some View {
        let align: VerticalAlignment = element.alignment == "top" ? .top
            : element.alignment == "bottom" ? .bottom : .center
        HStack(alignment: align, spacing: element.spacing ?? 0) { renderChildren() }
    }

    @ViewBuilder private func renderZStack() -> some View { ZStack { renderChildren() } }

    @ViewBuilder private func renderGrid() -> some View {
        let cols = element.columns ?? 2; let sp = element.spacing ?? 4
        LazyVGrid(
            columns: Array(repeating: GridItem(.flexible(), spacing: sp), count: Int(cols)),
            spacing: element.rowSpacing ?? sp
        ) { renderChildren() }
    }

    @ViewBuilder private func renderContainer() -> some View {
        let a = parseAlignment(element.contentAlignment)
        if let children = element.children, !children.isEmpty {
            ZStack(alignment: a) {
                ForEach(children.indices, id: \.self) { idx in
                    DynamicElementView(element: children[idx])
                }
            }
        } else {
            EmptyView()
        }
    }

    @ViewBuilder private func renderChildren() -> some View {
        if let children = element.children {
            ForEach(children.indices, id: \.self) { DynamicElementView(element: children[$0]) }
        }
    }

    // MARK: Text

    @ViewBuilder private func renderText() -> some View {
        let txt: Text = {
            let t = Text(element.content ?? "")
            if let ts = textStyleFont(element.textStyle) {
                return t.font(ts.weight(fontWeight(element.fontWeight)))
            }
            return t.font(.system(size: element.fontSize ?? 14,
                                  weight: fontWeight(element.fontWeight),
                                  design: fontDesign(element.fontDesign)))
        }()
        let colored = resolveColor(element.color).map { txt.foregroundColor($0) } ?? txt
        if let limit = element.lineLimit { colored.lineLimit(Int(limit)) } else { colored }
    }

    // MARK: Image

    @ViewBuilder private func renderImage() -> some View {
        let s = element.size ?? 24
        if let sn = element.systemName {
            let img = Image(systemName: sn).resizable()
                .aspectRatio(contentMode: element.contentMode == "fill" ? .fill : .fit)
                .frame(width: s, height: s)
            if let c = resolveColor(element.color) { img.foregroundColor(c) } else { img }
        } else if let b64 = element.data, let raw = Data(base64Encoded: b64) {
            #if canImport(UIKit)
            if let uiImage = UIImage(data: raw) {
                Image(uiImage: uiImage).resizable()
                    .aspectRatio(contentMode: element.contentMode == "fill" ? .fill : .fit)
                    .frame(width: s, height: s)
            } else { placeholderImage(s) }
            #elseif canImport(AppKit)
            if let nsImage = NSImage(data: raw) {
                Image(nsImage: nsImage).resizable()
                    .aspectRatio(contentMode: element.contentMode == "fill" ? .fill : .fit)
                    .frame(width: s, height: s)
            } else { placeholderImage(s) }
            #endif
        } else {
            placeholderImage(s)
        }
    }

    @ViewBuilder private func placeholderImage(_ s: CGFloat) -> some View {
        Image(systemName: "questionmark.square.dashed").resizable()
            .frame(width: s, height: s).foregroundColor(.gray)
    }

    // MARK: Progress

    @ViewBuilder private func renderProgress() -> some View {
        let v = element.value ?? 0; let t = element.total ?? 1
        let tc = resolveColor(element.tint) ?? Color.accentColor
        if element.barStyle == "circular" {
            ProgressView(value: v, total: t).progressViewStyle(CircularProgressViewStyle(tint: tc))
        } else {
            VStack(alignment: .leading, spacing: 2) {
                if let lbl = element.label {
                    Text(lbl).font(.caption2)
                        .foregroundColor(resolveColor(element.color) ?? .secondary)
                }
                ProgressView(value: v, total: t).tint(tc)
            }
        }
    }

    // MARK: Gauge

    @ViewBuilder private func renderGauge() -> some View {
        let v = element.value ?? 0; let lo = element.min ?? 0; let hi = element.max ?? 1
        let tc = resolveColor(element.tint) ?? Color.accentColor
        Gauge(value: v, in: lo...hi) {
            if let lbl = element.label { Text(lbl).font(.caption2) }
        } currentValueLabel: {
            if let cvl = element.currentValueLabel { Text(cvl).font(.caption) }
        }.gaugeStyle(.accessoryCircular).tint(tc)
    }

    // MARK: Button

    @ViewBuilder private func renderButton() -> some View {
        let lbl = element.label ?? element.content ?? ""
        let alignRaw = (element.textAlignment ?? element.alignment ?? "center").lowercased()
        let textAlign: TextAlignment = alignRaw == "trailing" || alignRaw == "right" || alignRaw == "end" ? .trailing : (alignRaw == "center" || alignRaw == "middle" ? .center : .leading)
        let frameAlign: Alignment = alignRaw == "trailing" || alignRaw == "right" || alignRaw == "end" ? .trailing : (alignRaw == "center" || alignRaw == "middle" ? .center : .leading)
        let hasExplicitAlign = element.textAlignment != nil || element.alignment != nil
        let baseText = Text(lbl)
            .font(.system(size: element.fontSize ?? 14, weight: .medium))
            .foregroundColor(resolveColor(element.color) ?? .white)
            .multilineTextAlignment(textAlign)
        let alignedText: AnyView = hasExplicitAlign
            ? AnyView(baseText.frame(maxWidth: .infinity, alignment: frameAlign))
            : AnyView(baseText)
        let btnContent = alignedText
            .padding(.horizontal, 12).padding(.vertical, 6)
            .background(resolveColor(element.backgroundColor) ?? Color.accentColor)
            .cornerRadius(element.cornerRadius ?? 8)
        if let act = element.action, !act.isEmpty {
            Button(intent: WidgetActionIntent(actionName: act)) { btnContent }.buttonStyle(.plain)
        } else if let u = element.url, !u.isEmpty, let url = URL(string: u) {
            Link(destination: url) { btnContent }
        } else {
            btnContent
        }
    }

    // MARK: Toggle

    @ViewBuilder private func renderToggle() -> some View {
        let isOn = element.isOn ?? false
        HStack(spacing: 6) {
            Image(systemName: isOn ? "checkmark.circle.fill" : "circle")
                .foregroundColor(isOn ? (resolveColor(element.tint) ?? .green) : .gray)
                .font(.system(size: 18))
            if let lbl = element.label {
                Text(lbl).font(.system(size: 14))
                    .foregroundColor(resolveColor(element.color) ?? .primary)
            }
        }
    }

    // MARK: Divider / Spacer

    @ViewBuilder private func renderDivider() -> some View {
        Rectangle().fill(resolveColor(element.color) ?? Color.gray.opacity(0.3))
            .frame(height: element.thickness ?? 1)
    }

    @ViewBuilder private func renderSpacer() -> some View {
        if let ml = element.minLength { Spacer(minLength: ml) } else { Spacer() }
    }

    // MARK: Date

    @ViewBuilder private func renderDate() -> some View {
        let d: Date = {
            guard let ds = element.date else { return Date() }
            let f = ISO8601DateFormatter()
            f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
            return f.date(from: ds) ?? Date()
        }()
        let tv: Text = {
            switch element.dateStyle {
            case "time": return Text(d, style: .time)
            case "date": return Text(d, style: .date)
            case "relative": return Text(d, style: .relative)
            case "offset": return Text(d, style: .offset)
            case "timer": return Text(d, style: .timer)
            default: return Text(d, style: .time)
            }
        }()
        tv.font(.system(size: element.fontSize ?? 14))
            .foregroundColor(resolveColor(element.color) ?? .primary)
    }

    // MARK: Chart

    @ViewBuilder private func renderChart() -> some View {
        let pts = element.chartData ?? []; let maxV = pts.map(\.value).max() ?? 1
        let tc = resolveColor(element.tint) ?? Color.accentColor
        switch element.chartType {
        case "line":  renderLineChart(pts: pts, maxV: maxV, tc: tc)
        case "area":  renderAreaChart(pts: pts, maxV: maxV, tc: tc)
        case "pie":   renderPieChart(pts: pts, tc: tc)
        default:      renderBarChart(pts: pts, maxV: maxV, tc: tc)
        }
    }

    @ViewBuilder private func renderList() -> some View {
        let rows = element.items ?? []
        VStack(alignment: .leading, spacing: element.spacing ?? 4) {
            ForEach(rows.indices, id: \.self) { i in
                let row = rows[i]
                let hasCheckbox = row.checked != nil
                let rowView = HStack(spacing: 6) {
                    if hasCheckbox {
                        Image(systemName: (row.checked ?? false) ? "checkmark.circle.fill" : "circle")
                            .foregroundColor((row.checked ?? false) ? .green : .gray)
                            .font(.system(size: 12))
                    }
                    Text(row.text)
                        .font(.system(size: element.fontSize ?? 13))
                        .lineLimit(1)
                        .foregroundColor(resolveColor(element.color) ?? .primary)
                }
                if let action = row.action, !action.isEmpty {
                    Button(intent: WidgetActionIntent(actionName: action)) { rowView }
                        .buttonStyle(.plain)
                } else {
                    rowView
                }
            }
        }
    }

    @ViewBuilder private func renderBarChart(pts: [ChartDataPoint], maxV: Double, tc: Color) -> some View {
        HStack(alignment: .bottom, spacing: 4) {
            ForEach(pts.indices, id: \.self) { i in
                let pt = pts[i]; let h = maxV > 0 ? CGFloat(pt.value / maxV) : 0
                VStack(spacing: 2) {
                    RoundedRectangle(cornerRadius: 2)
                        .fill(resolveColor(pt.color) ?? tc).frame(height: Swift.max(h * 60, 2))
                    Text(pt.label).font(.system(size: 8)).foregroundColor(.secondary).lineLimit(1)
                }
            }
        }
    }

    @ViewBuilder private func renderLineChart(pts: [ChartDataPoint], maxV: Double, tc: Color) -> some View {
        GeometryReader { geo in
            let w = geo.size.width; let h = geo.size.height; let cnt = CGFloat(Swift.max(pts.count - 1, 1))
            Path { p in
                for (i, pt) in pts.enumerated() {
                    let x = (CGFloat(i) / cnt) * w
                    let y = h - (maxV > 0 ? CGFloat(pt.value / maxV) * h : 0)
                    if i == 0 { p.move(to: CGPoint(x: x, y: y)) }
                    else { p.addLine(to: CGPoint(x: x, y: y)) }
                }
            }.stroke(tc, lineWidth: 2)
        }.frame(height: 60)
    }

    @ViewBuilder private func renderAreaChart(pts: [ChartDataPoint], maxV: Double, tc: Color) -> some View {
        GeometryReader { geo in
            let w = geo.size.width; let h = geo.size.height; let cnt = CGFloat(Swift.max(pts.count - 1, 1))
            ZStack {
                Path { p in
                    p.move(to: CGPoint(x: 0, y: h))
                    for (i, pt) in pts.enumerated() {
                        let x = (CGFloat(i) / cnt) * w
                        let y = h - (maxV > 0 ? CGFloat(pt.value / maxV) * h : 0)
                        p.addLine(to: CGPoint(x: x, y: y))
                    }
                    p.addLine(to: CGPoint(x: w, y: h)); p.closeSubpath()
                }.fill(tc.opacity(0.3))
                Path { p in
                    for (i, pt) in pts.enumerated() {
                        let x = (CGFloat(i) / cnt) * w
                        let y = h - (maxV > 0 ? CGFloat(pt.value / maxV) * h : 0)
                        if i == 0 { p.move(to: CGPoint(x: x, y: y)) }
                        else { p.addLine(to: CGPoint(x: x, y: y)) }
                    }
                }.stroke(tc, lineWidth: 2)
            }
        }.frame(height: 60)
    }

    @ViewBuilder private func renderPieChart(pts: [ChartDataPoint], tc: Color) -> some View {
        let total = pts.map(\.value).reduce(0, +)
        let defColors: [Color] = [.blue, .green, .orange, .red, .purple, .yellow, .pink, .teal]
        GeometryReader { geo in
            let s = Swift.min(geo.size.width, geo.size.height)
            let center = CGPoint(x: geo.size.width / 2, y: geo.size.height / 2); let radius = s / 2
            ZStack {
                ForEach(pts.indices, id: \.self) { i in
                    let sf = pts.prefix(i).map(\.value).reduce(0, +) / Swift.max(total, 1)
                    let ef = sf + pts[i].value / Swift.max(total, 1)
                    let sc = resolveColor(pts[i].color) ?? (i < defColors.count ? defColors[i] : tc)
                    Path { p in
                        p.move(to: center)
                        p.addArc(center: center, radius: radius,
                                 startAngle: .degrees(sf * 360 - 90),
                                 endAngle: .degrees(ef * 360 - 90), clockwise: false)
                        p.closeSubpath()
                    }.fill(sc)
                }
            }
        }.aspectRatio(1, contentMode: .fit).frame(height: 80)
    }

    // MARK: Link

    @ViewBuilder private func renderLink() -> some View {
        let content = VStack(spacing: 0) { renderChildren() }
        if let act = element.action, !act.isEmpty {
            Button(intent: WidgetActionIntent(actionName: act)) { content }.buttonStyle(.plain)
        } else if let u = element.url, !u.isEmpty, let url = URL(string: u) {
            Link(destination: url) { content }
        } else { content }
    }

    // MARK: Shape

    @ViewBuilder private func renderShape() -> some View {
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
                Capsule().fill(fc).frame(height: s)
                if let c = sc { Capsule().stroke(c, lineWidth: sw).frame(height: s) }
            }
        default:
            let cr = element.cornerRadius ?? 0
            ZStack {
                RoundedRectangle(cornerRadius: cr).fill(fc).frame(width: s, height: s)
                if let c = sc { RoundedRectangle(cornerRadius: cr).stroke(c, lineWidth: sw).frame(width: s, height: s) }
            }
        }
    }

    // MARK: Timer

    @ViewBuilder private func renderTimer() -> some View {
        let target: Date = {
            guard let ds = element.targetDate else { return Date() }
            let f = ISO8601DateFormatter(); f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
            return f.date(from: ds) ?? Date()
        }()
        Text(target, style: .timer)
            .font(.system(size: element.fontSize ?? 14, weight: fontWeight(element.fontWeight)))
            .foregroundColor(resolveColor(element.color) ?? .primary)
    }

    // MARK: Label

    @ViewBuilder private func renderLabel() -> some View {
        let sp = element.spacing ?? 4
        HStack(spacing: sp) {
            if let sn = element.systemName {
                let isz = (element.fontSize ?? 14) * 1.1
                Image(systemName: sn).resizable().aspectRatio(contentMode: .fit)
                    .frame(width: isz, height: isz)
                    .foregroundColor(resolveColor(element.iconColor)
                                     ?? resolveColor(element.color) ?? .primary)
            }
            Text(element.text ?? element.content ?? "")
                .font(.system(size: element.fontSize ?? 14, weight: fontWeight(element.fontWeight)))
                .foregroundColor(resolveColor(element.color) ?? .primary)
        }
    }

    // MARK: Canvas

    @ViewBuilder private func renderCanvas() -> some View {
        let cw = element.width ?? 100; let ch = element.height ?? 100
        let commands = element.elements ?? []
        let canvasView = SwiftUI.Canvas { context, size in
            let sx = size.width / cw; let sy = size.height / ch; let s = Swift.min(sx, sy)
            for cmd in commands {
                switch cmd.draw {
                case "circle":
                    let cx = (cmd.cx ?? 0) * sx; let cy = (cmd.cy ?? 0) * sy; let r = (cmd.r ?? 10) * s
                    let rect = CGRect(x: cx - r, y: cy - r, width: r * 2, height: r * 2)
                    if let f = resolveColor(cmd.fill) { context.fill(SwiftUI.Path(ellipseIn: rect), with: .color(f)) }
                    if let st = resolveColor(cmd.stroke) { context.stroke(SwiftUI.Path(ellipseIn: rect), with: .color(st), lineWidth: (cmd.strokeWidth ?? 1) * s) }
                case "line":
                    var path = SwiftUI.Path()
                    path.move(to: CGPoint(x: (cmd.x1 ?? 0) * sx, y: (cmd.y1 ?? 0) * sy))
                    path.addLine(to: CGPoint(x: (cmd.x2 ?? 0) * sx, y: (cmd.y2 ?? 0) * sy))
                    var style = StrokeStyle(lineWidth: (cmd.strokeWidth ?? 1) * s)
                    if cmd.lineCap == "round" { style.lineCap = .round }
                    context.stroke(path, with: .color(resolveColor(cmd.stroke) ?? Color(hex: "#ffffff")), style: style)
                case "rect":
                    let rect = CGRect(x: (cmd.x ?? 0) * sx, y: (cmd.y ?? 0) * sy,
                                      width: (cmd.width ?? 10) * sx, height: (cmd.height ?? 10) * sy)
                    let cr = (cmd.cornerRadius ?? 0) * s
                    let p = SwiftUI.Path(roundedRect: rect, cornerRadius: cr)
                    if let f = resolveColor(cmd.fill) { context.fill(p, with: .color(f)) }
                    if let st = resolveColor(cmd.stroke) { context.stroke(p, with: .color(st), lineWidth: (cmd.strokeWidth ?? 1) * s) }
                case "arc":
                    let cx = (cmd.cx ?? 0) * sx; let cy = (cmd.cy ?? 0) * sy; let r = (cmd.r ?? 10) * s
                    var path = SwiftUI.Path()
                    if cmd.fill != nil { path.move(to: CGPoint(x: cx, y: cy)) }
                    path.addArc(center: CGPoint(x: cx, y: cy), radius: r,
                                startAngle: .degrees(Double(cmd.startAngle ?? 0)),
                                endAngle: .degrees(Double(cmd.endAngle ?? 360)), clockwise: false)
                    if cmd.fill != nil { path.closeSubpath() }
                    if let f = resolveColor(cmd.fill) { context.fill(path, with: .color(f)) }
                    if let st = resolveColor(cmd.stroke) { context.stroke(path, with: .color(st), lineWidth: (cmd.strokeWidth ?? 1) * s) }
                case "text":
                    let fs = (cmd.fontSize ?? 12) * s
                    let txt = SwiftUI.Text(cmd.content ?? "").font(.system(size: fs))
                        .foregroundColor(resolveColor(cmd.color) ?? .primary)
                    let pt = CGPoint(x: (cmd.x ?? 0) * sx, y: (cmd.y ?? 0) * sy)
                    let anchor: UnitPoint = cmd.anchor == "end" ? .trailing : cmd.anchor == "middle" ? .center : .leading
                    context.draw(context.resolve(txt), at: pt, anchor: anchor)
                default: break
                }
            }
        }
        let shouldFill = (element.flex ?? 0) > 0
        if shouldFill {
            canvasView
                .aspectRatio(cw / ch, contentMode: .fit)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            canvasView.frame(width: cw, height: ch)
        }
    }

    // MARK: - Style Application

    @ViewBuilder
    private func applyStyle<V: View>(to view: V, element el: WidgetElement) -> some View {
        view
            .modifier(FlexMod(flex: el.flex))
            .modifier(PaddingMod(p: el.padding))
            .modifier(BgMod(bg: el.background, cr: el.cornerRadius))
            .modifier(FrameMod(f: el.frame))
            .modifier(BorderMod(b: el.border, cr: el.cornerRadius))
            .modifier(ClipShapeMod(shape: el.clipShape, cr: el.cornerRadius))
            .modifier(OpacityMod(o: el.opacity))
            .modifier(ShadowMod(s: el.shadow))
    }

    // MARK: - Helpers

    private func fontWeight(_ w: String?) -> Font.Weight {
        switch w {
        case "ultralight": return .ultraLight; case "thin": return .thin; case "light": return .light
        case "medium": return .medium; case "semibold": return .semibold; case "bold": return .bold
        case "heavy": return .heavy; case "black": return .black; default: return .regular
        }
    }

    private func fontDesign(_ d: String?) -> Font.Design {
        switch d {
        case "monospaced": return .monospaced; case "rounded": return .rounded
        case "serif": return .serif; default: return .default
        }
    }

    private func textStyleFont(_ style: String?) -> Font? {
        switch style {
        case "largeTitle": return .largeTitle
        case "title": return .title
        case "title2": return .title2
        case "title3": return .title3
        case "headline": return .headline
        case "subheadline": return .subheadline
        case "body": return .body
        case "callout": return .callout
        case "footnote": return .footnote
        case "caption": return .caption
        case "caption2": return .caption2
        default: return nil
        }
    }

    private func parseAlignment(_ a: String?) -> Alignment {
        switch a {
        case "topLeading": return .topLeading
        case "top": return .top
        case "topTrailing": return .topTrailing
        case "leading": return .leading
        case "trailing": return .trailing
        case "bottomLeading": return .bottomLeading
        case "bottom": return .bottom
        case "bottomTrailing": return .bottomTrailing
        default: return .center
        }
    }
}

// MARK: - View Modifiers

private struct PaddingMod: ViewModifier {
    let p: PaddingValue?
    func body(content: Content) -> some View {
        switch p {
        case .uniform(let v): content.padding(v)
        case .edges(let t, let b, let l, let tr):
            content.padding(EdgeInsets(top: t ?? 0, leading: l ?? 0, bottom: b ?? 0, trailing: tr ?? 0))
        case nil: content
        }
    }
}

private struct BgMod: ViewModifier {
    let bg: BackgroundValue?; let cr: CGFloat?
    func body(content: Content) -> some View {
        switch bg {
        case .solid(let hex):
            let c = Color.semantic(hex) ?? Color(hex: hex)
            content.background(RoundedRectangle(cornerRadius: cr ?? 0).fill(c))
        case .gradient(let g):
            let colors = g.colors.map { Color(hex: $0) }
            let (s, e): (UnitPoint, UnitPoint) = {
                switch g.direction {
                case "bottomToTop": return (.bottom, .top)
                case "leadingToTrailing": return (.leading, .trailing)
                case "trailingToLeading": return (.trailing, .leading)
                case "topLeadingToBottomTrailing": return (.topLeading, .bottomTrailing)
                case "topTrailingToBottomLeading": return (.topTrailing, .bottomLeading)
                default: return (.top, .bottom)
                }
            }()
            content.background(RoundedRectangle(cornerRadius: cr ?? 0)
                .fill(LinearGradient(colors: colors, startPoint: s, endPoint: e)))
        case .adaptive(let l, let d):
            content.background(RoundedRectangle(cornerRadius: cr ?? 0)
                .fill(Color.adaptive(light: l, dark: d)))
        case nil: content
        }
    }
}

private struct ShadowMod: ViewModifier {
    let s: ShadowConfig?
    func body(content: Content) -> some View {
        if let s = s {
            content.shadow(color: s.color.map { Color(hex: $0) } ?? Color.black.opacity(0.3),
                           radius: s.radius ?? 4, x: s.x ?? 0, y: s.y ?? 2)
        } else { content }
    }
}

private struct FrameMod: ViewModifier {
    let f: FrameConfig?
    func body(content: Content) -> some View {
        if let f = f {
            content.frame(width: f.width, height: f.height)
                .frame(maxWidth: f.maxWidth?.cgFloat, maxHeight: f.maxHeight?.cgFloat)
        } else { content }
    }
}

private struct BorderMod: ViewModifier {
    let b: BorderConfig?; let cr: CGFloat?
    func body(content: Content) -> some View {
        if let b = b {
            content.overlay(RoundedRectangle(cornerRadius: cr ?? 0)
                .stroke(Color(hex: b.color ?? "#888"), lineWidth: b.width ?? 1))
        } else { content }
    }
}

private struct ClipShapeMod: ViewModifier {
    let shape: String?; let cr: CGFloat?
    func body(content: Content) -> some View {
        switch shape {
        case "circle": AnyView(content.clipShape(Circle()))
        case "capsule": AnyView(content.clipShape(Capsule()))
        case "rectangle": AnyView(content.clipShape(RoundedRectangle(cornerRadius: cr ?? 0)))
        default: AnyView(content)
        }
    }
}

private struct FlexMod: ViewModifier {
    let flex: CGFloat?
    func body(content: Content) -> some View {
        if let f = flex, f > 0 {
            content.frame(maxWidth: .infinity).layoutPriority(Double(f))
        } else { content }
    }
}

private struct OpacityMod: ViewModifier {
    let o: Double?
    func body(content: Content) -> some View {
        if let o = o { content.opacity(o) } else { content }
    }
}
