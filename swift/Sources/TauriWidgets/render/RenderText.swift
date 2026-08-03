import SwiftUI

// MARK: - Text / Date / Timer / Label

extension DynamicElementView {
    @ViewBuilder func renderTextGroup() -> some View {
        switch element.type {
        case "text":    renderText()
        case "label":   renderLabel()
        case "date":    renderDate()
        case "timer":   renderTimer()
        default: EmptyView()
        }
    }

    // MARK: Text

    @ViewBuilder func renderText() -> some View {
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

    // MARK: Date

    @ViewBuilder func renderDate() -> some View {
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

    // MARK: Timer

    @ViewBuilder func renderTimer() -> some View {
        let target: Date = {
            guard let ds = element.targetDate else { return Date() }
            let f = ISO8601DateFormatter()
            f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
            if let parsed = f.date(from: ds) { return parsed }
            f.formatOptions = [.withInternetDateTime]
            return f.date(from: ds) ?? Date()
        }()
        let countingUp = (element.counting ?? "down").lowercased() == "up"
        // WidgetKit keeps Text(timerInterval:) live without a process-resident TimelineView.
        let style = Font.system(size: element.fontSize ?? 14, weight: fontWeight(element.fontWeight)).monospacedDigit()
        let color = resolveColor(element.color) ?? Color.primary
        if countingUp {
            Text(timerInterval: target...Date.distantFuture, countsDown: false)
                .font(style)
                .foregroundColor(color)
                .lineLimit(1)
                .minimumScaleFactor(0.55)
                .allowsTightening(true)
        } else {
            Text(timerInterval: Date.now...target, countsDown: true)
                .font(style)
                .foregroundColor(color)
                .lineLimit(1)
                .minimumScaleFactor(0.55)
                .allowsTightening(true)
        }
    }

    // MARK: Label

    @ViewBuilder func renderLabel() -> some View {
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
}
