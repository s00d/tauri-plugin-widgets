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
    var parentAxis: Axis? = nil
    /// When true, text must not expand to maxWidth (breaks ZStack badge centering).
    var inZStack: Bool = false
    /// WidgetKit root: `containerBackground` already paints the canvas — skip
    /// root `background`/`shadow` or a content-sized card floats inside (looks “crooked”).
    var isWidgetRoot: Bool = false

    public init(
        element: WidgetElement,
        parentAxis: Axis? = nil,
        inZStack: Bool = false,
        isWidgetRoot: Bool = false
    ) {
        self.element = element
        self.parentAxis = parentAxis
        self.inZStack = inZStack
        self.isWidgetRoot = isWidgetRoot
    }

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
        VStack(alignment: align, spacing: element.spacing ?? 0) { renderChildren(axis: .vertical) }
    }

    @ViewBuilder private func renderHStack() -> some View {
        let align: VerticalAlignment = element.alignment == "top" ? .top
            : element.alignment == "bottom" ? .bottom : .center
        HStack(alignment: align, spacing: element.spacing ?? 0) { renderChildren(axis: .horizontal) }
    }

    @ViewBuilder private func renderZStack() -> some View {
        ZStack(alignment: parseAlignment(element.alignment)) {
            if let children = element.children {
                ForEach(children.indices, id: \.self) { idx in
                    DynamicElementView(element: children[idx], inZStack: true)
                }
            }
        }
    }

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
            // Expand so contentAlignment centers inside frame/padding, not intrinsic hug.
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: a)
        } else {
            EmptyView()
        }
    }

    @ViewBuilder private func renderChildren(axis: Axis? = nil) -> some View {
        if let children = element.children {
            ForEach(children.indices, id: \.self) {
                DynamicElementView(element: children[$0], parentAxis: axis)
            }
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
        let colored = txt.foregroundColor(resolveColor(element.color) ?? .primary)
        // Circle / fixed box badges (e.g. avatar initials) center by default — leading +
        // maxWidth:.infinity pinned "AK" to the top-left of the clip.
        let hasFixedBox = element.frame?.width != nil && element.frame?.height != nil
        let defaultAlign = (element.clipShape == "circle" || hasFixedBox) ? "center" : "leading"
        let alignRaw = (element.alignment ?? defaultAlign).lowercased()
        let textAlign: TextAlignment = alignRaw == "trailing" || alignRaw == "right" || alignRaw == "end" ? .trailing
            : (alignRaw == "center" || alignRaw == "middle" ? .center : .leading)
        let frameAlign: Alignment = alignRaw == "trailing" || alignRaw == "right" || alignRaw == "end" ? .trailing
            : (alignRaw == "center" || alignRaw == "middle" ? .center : .leading)
        let base = colored.multilineTextAlignment(textAlign)
        Group {
            if let limit = element.lineLimit {
                base.lineLimit(Int(limit))
            } else {
                base
            }
        }
        .fixedSize(horizontal: inZStack || hasFixedBox, vertical: hasFixedBox)
        .frame(maxWidth: (inZStack || hasFixedBox) ? nil : .infinity, alignment: frameAlign)
    }

    // MARK: Image

    @ViewBuilder private func renderImage() -> some View {
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

    private func decodeImageData(_ raw: String?) -> Data? {
        guard var s = raw?.trimmingCharacters(in: .whitespacesAndNewlines), !s.isEmpty else { return nil }
        if s.hasPrefix("data:"), let comma = s.firstIndex(of: ",") {
            s = String(s[s.index(after: comma)...])
        }
        return Data(base64Encoded: s)
    }

    @ViewBuilder private func decodedImageView(_ raw: Data, size: CGFloat) -> some View {
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

    @ViewBuilder private func placeholderImage(_ s: CGFloat) -> some View {
        Image(systemName: "questionmark.square.dashed").resizable()
            .frame(width: s, height: s).foregroundColor(.gray)
    }

    // MARK: Progress

    @ViewBuilder private func renderProgress() -> some View {
        let v = element.value ?? 0; let t = max(element.total ?? 1, 0.0001)
        let tc = resolveColor(element.tint) ?? Color.accentColor
        let frac = min(max(v / t, 0), 1)
        if element.barStyle == "circular" {
            // Custom ring — ProgressView breaks under ImageRenderer / macOS snapshots.
            ZStack {
                Circle().stroke(tc.opacity(0.25), lineWidth: 4)
                Circle()
                    .trim(from: 0, to: frac)
                    .stroke(tc, style: StrokeStyle(lineWidth: 4, lineCap: .round))
                    .rotationEffect(.degrees(-90))
            }
            .frame(width: 40, height: 40)
        } else {
            // Avoid bare GeometryReader in unconstrained HStacks — it steals height and
            // collapses into a tall thin strip (Tasks medium sidebar).
            VStack(alignment: .leading, spacing: 2) {
                if let lbl = element.label {
                    Text(lbl).font(.caption2)
                        .lineLimit(1)
                        .minimumScaleFactor(0.75)
                        .foregroundColor(
                            resolveColor(element.color)
                                ?? resolveColor(element.tint)
                                ?? .secondary
                        )
                }
                Capsule()
                    .fill(tc.opacity(0.25))
                    .frame(height: 6)
                    .frame(maxWidth: .infinity)
                    .overlay(alignment: .leading) {
                        GeometryReader { geo in
                            Capsule()
                                .fill(tc)
                                .frame(width: max(geo.size.width * CGFloat(frac), 2), height: 6)
                        }
                    }
            }
            .frame(minWidth: 72, maxWidth: .infinity, alignment: .leading)
        }
    }

    // MARK: Gauge

    @ViewBuilder private func renderGauge() -> some View {
        let v = element.value ?? 0; let lo = element.min ?? 0; let hi = element.max ?? 1
        let tc = resolveColor(element.tint) ?? Color.accentColor
        let ink = resolveColor(element.color) ?? .primary
        let span = max(hi - lo, 0.0001)
        let frac = min(max((v - lo) / span, 0), 1)
        if element.gaugeStyle == "linear" {
            // Match desktop/HTML: label row + capsule bar (not accessoryLinear slider chrome).
            VStack(alignment: .leading, spacing: 2) {
                HStack {
                    if let lbl = element.label {
                        Text(lbl).font(.caption2).foregroundColor(ink.opacity(0.7))
                    }
                    Spacer(minLength: 0)
                    if let cvl = element.currentValueLabel {
                        Text(cvl).font(.caption).fontWeight(.semibold).foregroundColor(ink)
                    }
                }
                GeometryReader { geo in
                    ZStack(alignment: .leading) {
                        Capsule().fill(tc.opacity(0.25))
                        Capsule().fill(tc).frame(width: max(geo.size.width * frac, 2))
                    }
                }
                .frame(height: 6)
            }
        } else {
            // Custom ring — accessoryCircular is blank/misaligned under ImageRenderer.
            VStack(spacing: 4) {
                ZStack {
                    Circle().stroke(tc.opacity(0.25), lineWidth: 5)
                    Circle()
                        .trim(from: 0, to: frac)
                        .stroke(tc, style: StrokeStyle(lineWidth: 5, lineCap: .round))
                        .rotationEffect(.degrees(-90))
                    if let cvl = element.currentValueLabel {
                        Text(cvl).font(.caption).fontWeight(.semibold).foregroundColor(ink)
                    }
                }
                .frame(width: 56, height: 56)
                if let lbl = element.label {
                    Text(lbl).font(.caption2).foregroundColor(ink)
                }
            }
        }
    }

    // MARK: Button

    @ViewBuilder private func renderButton() -> some View {
        let lbl = element.label ?? element.content ?? ""
        let alignRaw = (element.textAlignment ?? element.alignment ?? "center").lowercased()
        let textAlign: TextAlignment = alignRaw == "trailing" || alignRaw == "right" || alignRaw == "end" ? .trailing : (alignRaw == "center" || alignRaw == "middle" ? .center : .leading)
        let frameAlign: Alignment = alignRaw == "trailing" || alignRaw == "right" || alignRaw == "end" ? .trailing : (alignRaw == "center" || alignRaw == "middle" ? .center : .leading)
        let hasExplicitAlign = element.textAlignment != nil || element.alignment != nil
        let fs = element.fontSize ?? 14
        // Prefer explicit `padding` as content insets; otherwise scale with fontSize so
        // keypad grids (Calculator small/medium) fit WidgetKit family height.
        let insets: EdgeInsets = {
            if let p = element.padding {
                switch p {
                case .uniform(let v):
                    return EdgeInsets(top: v, leading: v, bottom: v, trailing: v)
                case .edges(let t, let b, let l, let tr):
                    return EdgeInsets(top: t ?? 0, leading: l ?? 0, bottom: b ?? 0, trailing: tr ?? 0)
                }
            }
            let h = min(12, max(3, fs * 0.45))
            let v = min(6, max(1.5, fs * 0.22))
            return EdgeInsets(top: v, leading: h, bottom: v, trailing: h)
        }()
        let baseText = Text(lbl)
            .font(.system(size: fs, weight: .medium))
            .foregroundColor(resolveColor(element.color) ?? .primary)
            .multilineTextAlignment(textAlign)
        let alignedText: AnyView = hasExplicitAlign
            ? AnyView(baseText.frame(maxWidth: .infinity, alignment: frameAlign))
            : AnyView(baseText)
        let btnContent = alignedText
            .padding(insets)
            .frame(maxWidth: .infinity)
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
        let content = HStack(spacing: 6) {
            Image(systemName: isOn ? "checkmark.circle.fill" : "circle")
                .foregroundColor(isOn ? (resolveColor(element.tint) ?? .green) : .gray)
                .font(.system(size: 18))
            if let lbl = element.label {
                Text(lbl).font(.system(size: 14))
                    .foregroundColor(resolveColor(element.color) ?? .primary)
            }
        }
        if let act = element.action, !act.isEmpty {
            Button(intent: WidgetActionIntent(actionName: act, payload: isOn ? "false" : "true")) {
                content
            }.buttonStyle(.plain)
        } else {
            content
        }
    }

    // MARK: Divider / Spacer

    @ViewBuilder private func renderDivider() -> some View {
        let fill = Rectangle().fill(resolveColor(element.color) ?? Color.gray.opacity(0.3))
        let thickness = element.thickness ?? 1
        if parentAxis == .horizontal {
            fill.frame(width: thickness)
        } else {
            fill.frame(height: thickness)
                .padding(.vertical, 4)
        }
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
            if let parsed = f.date(from: ds) { return parsed }
            f.formatOptions = [.withInternetDateTime]
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
        let anyCheckbox = rows.contains { $0.checked != nil }
        VStack(alignment: .leading, spacing: element.spacing ?? 4) {
            ForEach(rows.indices, id: \.self) { i in
                let row = rows[i]
                let hasCheckbox = row.checked != nil
                let rowView = HStack(spacing: 6) {
                    if hasCheckbox {
                        Image(systemName: (row.checked ?? false) ? "checkmark.circle.fill" : "circle")
                            .foregroundColor((row.checked ?? false) ? .green : .gray)
                            .font(.system(size: 12))
                            .frame(width: 14)
                    } else if anyCheckbox {
                        Color.clear.frame(width: 14, height: 12)
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
                    Text(pt.label).font(.system(size: 8)).foregroundColor(.white.opacity(0.7)).lineLimit(1)
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

    // MARK: Timer

    @ViewBuilder private func renderTimer() -> some View {
        let target: Date = {
            guard let ds = element.targetDate else { return Date() }
            let f = ISO8601DateFormatter()
            f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
            if let parsed = f.date(from: ds) { return parsed }
            f.formatOptions = [.withInternetDateTime]
            return f.date(from: ds) ?? Date()
        }()
        let countingUp = (element.counting ?? "down").lowercased() == "up"
        let diffMs = countingUp
            ? Date().timeIntervalSince(target) * 1000
            : target.timeIntervalSinceNow * 1000
        let sign = diffMs < 0 ? "-" : ""
        let absSec = Int(abs(diffMs) / 1000)
        let h = absSec / 3600
        let m = (absSec % 3600) / 60
        let s = absSec % 60
        let label = String(format: "%@%d:%02d:%02d", sign, h, m, s)
        Text(label)
            .font(.system(size: element.fontSize ?? 14, weight: fontWeight(element.fontWeight)).monospacedDigit())
            .foregroundColor(resolveColor(element.color) ?? .primary)
            .lineLimit(1)
            .minimumScaleFactor(0.55)
            .allowsTightening(true)
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

    // MARK: - Style Application

    @ViewBuilder
    private func applyStyle<V: View>(to view: V, element el: WidgetElement) -> some View {
        // Buttons consume `padding` as content insets in renderButton — don't double-apply.
        let outerPad: PaddingValue? = el.type == "button" ? nil : el.padding
        let styled = view
            .modifier(FlexMod(flex: el.flex))
            .modifier(PaddingMod(p: outerPad))
            .modifier(FrameMod(f: el.frame))
        if isWidgetRoot {
            // Keep padding/frame; chrome comes from WidgetKit containerBackground.
            styled
                .modifier(BorderMod(b: el.border, cr: el.cornerRadius))
                .modifier(OpacityMod(o: el.opacity))
        } else {
            styled
                .modifier(BgMod(bg: el.background, cr: el.cornerRadius))
                .modifier(BorderMod(b: el.border, cr: el.cornerRadius))
                .modifier(ClipShapeMod(shape: el.clipShape, cr: el.cornerRadius))
                .modifier(OpacityMod(o: el.opacity))
                .modifier(ShadowMod(s: el.shadow))
        }
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
            // Explicit center so badge/avatar content isn't left/top-biased inside the box.
            content.frame(width: f.width, height: f.height, alignment: .center)
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

/// Minimal SVG path parser (M/m L/l H/h V/v Z/z). Enough for simple canvas paths.
private func parseSVGPath(_ data: String) -> Path {
    var path = Path()
    let tokens = data.replacingOccurrences(of: ",", with: " ")
        .split(whereSeparator: { $0.isWhitespace })
        .map(String.init)
    var i = 0
    var cx: CGFloat = 0
    var cy: CGFloat = 0
    func nextNumber() -> CGFloat? {
        guard i < tokens.count, let v = Double(tokens[i]) else { return nil }
        i += 1
        return CGFloat(v)
    }
    while i < tokens.count {
        let cmd = tokens[i]
        // Command letters may be glued to numbers; handle single-letter cmds
        if cmd.count == 1, let c = cmd.first, c.isLetter {
            i += 1
            switch c {
            case "M":
                if let x = nextNumber(), let y = nextNumber() { cx = x; cy = y; path.move(to: CGPoint(x: x, y: y)) }
            case "m":
                if let x = nextNumber(), let y = nextNumber() { cx += x; cy += y; path.move(to: CGPoint(x: cx, y: cy)) }
            case "L":
                if let x = nextNumber(), let y = nextNumber() { cx = x; cy = y; path.addLine(to: CGPoint(x: x, y: y)) }
            case "l":
                if let x = nextNumber(), let y = nextNumber() { cx += x; cy += y; path.addLine(to: CGPoint(x: cx, y: cy)) }
            case "H":
                if let x = nextNumber() { cx = x; path.addLine(to: CGPoint(x: cx, y: cy)) }
            case "h":
                if let x = nextNumber() { cx += x; path.addLine(to: CGPoint(x: cx, y: cy)) }
            case "V":
                if let y = nextNumber() { cy = y; path.addLine(to: CGPoint(x: cx, y: cy)) }
            case "v":
                if let y = nextNumber() { cy += y; path.addLine(to: CGPoint(x: cx, y: cy)) }
            case "Z", "z":
                path.closeSubpath()
            default:
                break
            }
        } else {
            i += 1
        }
    }
    return path
}
