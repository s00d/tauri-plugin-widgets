import SwiftUI
import Charts

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

    /// `startAt`/`endAt` (Unix ms) as a date range, when both are set and ordered.
    var timeRange: ClosedRange<Date>? {
        guard let a = element.startAt, let b = element.endAt, b > a else { return nil }
        return Date(timeIntervalSince1970: Double(a) / 1000)...Date(timeIntervalSince1970: Double(b) / 1000)
    }

    /// Fraction of `timeRange` elapsed now (clamped), or nil when not time-driven.
    var timeFraction: Double? {
        guard let r = timeRange else { return nil }
        let span = r.upperBound.timeIntervalSince(r.lowerBound)
        return min(max(Date.now.timeIntervalSince(r.lowerBound) / span, 0), 1)
    }

    /// Longest `startAt`…`endAt` span (seconds) drawn as a live `ProgressView(timerInterval:)`.
    static let liveProgressMaxSpan: TimeInterval = 600

    @ViewBuilder func renderProgress() -> some View {
        let v = element.value ?? 0; let t = max(element.total ?? 1, 0.0001)
        let tc = resolveColor(element.tint) ?? Color.accentColor
        let track = resolveColor(element.trackColor) ?? tc.opacity(0.25)
        let frac = timeFraction ?? min(max(v / t, 0), 1)
        // Inside a short range the system ProgressView fills by itself (no timeline reloads).
        // It redraws all the time while visible: measured in Notification Center at ~34% CPU
        // for a 10 min range and ~100% for months, so a longer range is a static bar computed
        // at render time (each timeline entry / reload), which over hours moves too little to
        // need animation. Before / after the range a static bar is exact too.
        let live: ClosedRange<Date>? = timeRange.flatMap { r in
            r.contains(Date.now) && r.upperBound.timeIntervalSince(r.lowerBound) <= Self.liveProgressMaxSpan ? r : nil
        }
        if let live {
            let pv = ProgressView(timerInterval: live, countsDown: false) {
                if let lbl = element.label { Text(lbl).font(.caption2) }
            } currentValueLabel: { EmptyView() }
            if element.barStyle == "circular" {
                pv.progressViewStyle(.circular).tint(tc)
            } else {
                pv.progressViewStyle(.linear).tint(tc)
            }
        } else if element.barStyle == "circular" {
            let lw = element.thickness ?? 4
            // Custom ring — ProgressView breaks under ImageRenderer / macOS snapshots.
            ZStack {
                Circle().stroke(track, lineWidth: lw)
                Circle()
                    .trim(from: 0, to: frac)
                    .stroke(tc, style: StrokeStyle(lineWidth: lw, lineCap: .round))
                    .rotationEffect(.degrees(-90))
            }
            .frame(width: 40, height: 40)
        } else {
            let h = element.thickness ?? 6
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
                    .fill(track)
                    .frame(height: h)
                    .frame(maxWidth: .infinity)
                    .overlay(alignment: .leading) {
                        GeometryReader { geo in
                            Capsule()
                                .fill(tc)
                                .frame(width: max(geo.size.width * CGFloat(frac), 2), height: h)
                        }
                    }
            }
            .frame(minWidth: 72, maxWidth: .infinity, alignment: .leading)
        }
    }

    // MARK: Gauge

    @ViewBuilder func renderGauge() -> some View {
        let lo = element.min ?? 0; let hi = element.max ?? 1
        let tc = resolveColor(element.tint) ?? Color.accentColor
        let ink = resolveColor(element.color) ?? .primary
        let span = max(hi - lo, 0.0001)
        // Time-driven: no native live Gauge API — value is computed at render time.
        let frac = timeFraction ?? min(max(((element.value ?? 0) - lo) / span, 0), 1)
        let ends = element.minLabel != nil || element.maxLabel != nil
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
                if ends { gaugeEndLabels(ink) }
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
                if ends { gaugeEndLabels(ink).frame(width: 56) }
                if let lbl = element.label {
                    Text(lbl).font(.caption2).foregroundColor(ink)
                }
            }
            // Desktop: centered block (margin auto) that stretches in an unaligned column.
            .frame(maxWidth: parentAxis == .vertical && parentAlign == nil ? .infinity : nil)
        }
    }

    @ViewBuilder func gaugeEndLabels(_ ink: Color) -> some View {
        HStack {
            Text(element.minLabel ?? "")
            Spacer(minLength: 0)
            Text(element.maxLabel ?? "")
        }
        .font(.system(size: 9)).foregroundColor(ink.opacity(0.6)).lineLimit(1)
    }

    // MARK: Chart

    @ViewBuilder func renderChart() -> some View {
        let pts = element.chartData ?? []; let maxV = pts.map(\.value).max() ?? 1
        let tc = resolveColor(element.tint) ?? Color.accentColor
        if usesSwiftCharts {
            renderSwiftChart(pts: pts, tc: tc)
        } else {
            switch element.chartType {
            case "line":  renderLineChart(pts: pts, maxV: maxV, tc: tc)
        case "area":  renderAreaChart(pts: pts, maxV: maxV, tc: tc)
        case "pie":   renderPieChart(pts: pts, tc: tc)
        default:      renderBarChart(pts: pts, maxV: maxV, tc: tc)
            }
        }
    }

    // MARK: List

    @ViewBuilder func renderList() -> some View {
        // Custom rows (`children`) or typed `items` — stress-nest uses rich child rows.
        if let children = element.children, !children.isEmpty {
            VStack(alignment: .leading, spacing: element.spacing ?? 4) {
                ForEach(children.indices, id: \.self) { idx in
                    DynamicElementView(element: children[idx], parentAxis: .vertical)
                }
            }
        } else {
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
                        Button(intent: actionIntent(action, payload: row.payload)) { rowView }
                            .buttonStyle(.plain)
                    } else {
                        rowView
                    }
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

    // MARK: Swift Charts (v2 options)

    /// Legacy hand-drawn charts stay for plain configs (goldens); any v2 chart option
    /// switches to Swift Charts.
    var usesSwiftCharts: Bool {
        element.chartType == "point" || element.showXAxis != nil || element.showYAxis != nil
            || element.showLegend != nil || element.yMin != nil || element.yMax != nil
            || element.interpolation != nil || element.innerRadius != nil
            || (element.chartData ?? []).contains { $0.series != nil }
    }

    @ViewBuilder func renderSwiftChart(pts: [ChartDataPoint], tc: Color) -> some View {
        let defColors: [Color] = [.blue, .green, .orange, .red, .purple, .yellow, .pink, .teal]
        let isPie = element.chartType == "pie"
        // Color domain: pie slices by label, otherwise series (first explicit color wins).
        let keys: [String] = {
            var seen: [String] = []
            for p in pts {
                let k = isPie ? p.label : (p.series ?? "")
                if !seen.contains(k) { seen.append(k) }
            }
            return seen
        }()
        let colors: [Color] = keys.enumerated().map { i, k in
            let explicit = pts.first { (isPie ? $0.label : ($0.series ?? "")) == k && $0.color != nil }
            return resolveColor(explicit?.color) ?? (keys.count == 1 && !isPie ? tc : defColors[i % defColors.count])
        }
        let interp: InterpolationMethod = element.interpolation == "monotone" ? .monotone
            : element.interpolation == "step" ? .stepCenter : .linear
        let ink = resolveColor(element.color) ?? .secondary
        let chart = Chart(pts.indices, id: \.self) { i in
            let p = pts[i]
            let key = isPie ? p.label : (p.series ?? "")
            switch element.chartType {
            case "pie":
                SectorMark(angle: .value("Value", p.value),
                           innerRadius: .ratio(element.innerRadius ?? 0), angularInset: 1)
                    .foregroundStyle(by: .value("Series", key))
            case "line":
                LineMark(x: .value("Label", p.label), y: .value("Value", p.value))
                    .interpolationMethod(interp)
                    .foregroundStyle(by: .value("Series", key))
            case "area":
                AreaMark(x: .value("Label", p.label), y: .value("Value", p.value))
                    .interpolationMethod(interp)
                    .foregroundStyle(by: .value("Series", key))
            case "point":
                PointMark(x: .value("Label", p.label), y: .value("Value", p.value))
                    .foregroundStyle(by: .value("Series", key))
            default:
                BarMark(x: .value("Label", p.label), y: .value("Value", p.value))
                    .foregroundStyle(by: .value("Series", key))
                    .position(by: .value("Series", key))
            }
        }
        .chartForegroundStyleScale(domain: keys, range: colors)
        .chartLegend((element.showLegend ?? false) && (isPie || keys.count > 1) ? .visible : .hidden)
        .chartXAxis((element.showXAxis ?? true) && !isPie ? .automatic : .hidden)
        .chartYAxis((element.showYAxis ?? false) && !isPie ? .automatic : .hidden)
        .foregroundStyle(ink)
        let values = pts.map(\.value)
        if !isPie && (element.yMin != nil || element.yMax != nil) {
            let lo = element.yMin ?? Swift.min(0, values.min() ?? 0)
            let hi = Swift.max(element.yMax ?? values.max() ?? 1, lo + 0.0001)
            chart.chartYScale(domain: lo...hi).frame(minHeight: 50)
        } else if isPie {
            chart.aspectRatio(1, contentMode: .fit)
        } else {
            chart.frame(minHeight: 50)
        }
    }
}
