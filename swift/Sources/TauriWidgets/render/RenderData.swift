import SwiftUI

// MARK: - Progress / Gauge / Chart / List

extension DynamicElementView {
    @ViewBuilder func renderData() -> some View {
        switch element.type {
        case "progress":    renderProgress()
        case "gauge":       renderGauge()
        case "chart":       renderChart()
        case "list":        renderList()
        default: EmptyView()
        }
    }

    // MARK: Progress

    @ViewBuilder func renderProgress() -> some View {
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

    @ViewBuilder func renderGauge() -> some View {
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

    // MARK: Chart

    @ViewBuilder func renderChart() -> some View {
        let pts = element.chartData ?? []; let maxV = pts.map(\.value).max() ?? 1
        let tc = resolveColor(element.tint) ?? Color.accentColor
        switch element.chartType {
        case "line":  renderLineChart(pts: pts, maxV: maxV, tc: tc)
        case "area":  renderAreaChart(pts: pts, maxV: maxV, tc: tc)
        case "pie":   renderPieChart(pts: pts, tc: tc)
        default:      renderBarChart(pts: pts, maxV: maxV, tc: tc)
        }
    }

    // MARK: List

    @ViewBuilder func renderList() -> some View {
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
                    Button(intent: WidgetActionIntent(actionName: action, payload: row.payload)) { rowView }
                        .buttonStyle(.plain)
                } else {
                    rowView
                }
            }
        }
    }

    // MARK: Chart sub-renderers

    @ViewBuilder func renderBarChart(pts: [ChartDataPoint], maxV: Double, tc: Color) -> some View {
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

    @ViewBuilder func renderLineChart(pts: [ChartDataPoint], maxV: Double, tc: Color) -> some View {
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

    @ViewBuilder func renderAreaChart(pts: [ChartDataPoint], maxV: Double, tc: Color) -> some View {
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

    @ViewBuilder func renderPieChart(pts: [ChartDataPoint], tc: Color) -> some View {
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
}
