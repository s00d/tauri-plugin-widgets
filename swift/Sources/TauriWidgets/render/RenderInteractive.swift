import SwiftUI

// MARK: - Button / Toggle / Link

extension DynamicElementView {
    @ViewBuilder func renderInteractive() -> some View {
        switch element.type {
        case "button":  renderButton()
        case "toggle":  renderToggle()
        case "link":    renderLink()
        default: EmptyView()
        }
    }

    // MARK: Button

    @ViewBuilder func renderButton() -> some View {
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
            .foregroundColor(resolveColor(element.color) ?? .white)
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

    @ViewBuilder func renderToggle() -> some View {
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

    // MARK: Link

    @ViewBuilder func renderLink() -> some View {
        let content = VStack(spacing: 0) { renderChildren() }
        if let act = element.action, !act.isEmpty {
            Button(intent: WidgetActionIntent(actionName: act)) { content }.buttonStyle(.plain)
        } else if let u = element.url, !u.isEmpty, let url = URL(string: u) {
            Link(destination: url) { content }
        } else { content }
    }
}
