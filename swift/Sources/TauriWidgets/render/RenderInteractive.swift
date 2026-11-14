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
        // nil variant keeps the legacy filled look (== prominent) so existing layouts don't move.
        let variant = element.variant ?? "prominent"
        let fill = resolveColor(element.backgroundColor) ?? Color.accentColor
        let ink: Color = resolveColor(element.color)
            ?? (variant == "prominent" ? .white : (variant == "bordered" ? fill : Color.accentColor))
        let label: Text = {
            let t = Text(lbl).font(.system(size: fs, weight: .medium))
            guard let sn = element.systemName else { return t }
            return Text(Image(systemName: sn)).font(.system(size: fs, weight: .medium))
                + Text(lbl.isEmpty ? "" : " ") + t
        }()
        let baseText = label
            .foregroundColor(ink)
            .multilineTextAlignment(textAlign)
        // Explicit variants (v2) shrink a long label instead of breaking it mid-word;
        // legacy (no variant) wraps as before.
        let fitted: AnyView = element.variant == nil
            ? AnyView(baseText)
            : AnyView(baseText.lineLimit(1).minimumScaleFactor(0.7))
        let alignedText: AnyView = hasExplicitAlign
            ? AnyView(fitted.frame(maxWidth: .infinity, alignment: frameAlign))
            : fitted
        let cr = element.cornerRadius ?? 8
        let padded = alignedText
            .padding(insets)
            .frame(maxWidth: .infinity)
        let btnContent: AnyView = {
            switch variant {
            case "bordered":
                return AnyView(padded.background(fill.opacity(0.2)).cornerRadius(cr))
            case "plain":
                return AnyView(padded)
            case "outline":
                return AnyView(padded.overlay(RoundedRectangle(cornerRadius: cr).stroke(ink, lineWidth: 1)))
            default:
                return AnyView(padded.background(fill).cornerRadius(cr))
            }
        }()
        if element.disabled == true {
            btnContent.opacity(0.4)
        } else if let patch = statePatch {
            // No `.invalidatableContent()` here: it made the button non-interactive on macOS.
            Button(intent: stateIntent(patch, action: element.action)) { btnContent }.buttonStyle(.plain)
        } else if let act = element.action, !act.isEmpty {
            Button(intent: actionIntent(act)) { btnContent }.buttonStyle(.plain)
        } else if let u = element.url, !u.isEmpty, let url = URL(string: u) {
            Link(destination: url) { btnContent }
        } else {
            btnContent
        }
    }

    // MARK: Toggle

    @ViewBuilder func renderToggle() -> some View {
        let isOn = element.isOn ?? false
        if let patch = statePatch {
            // The system flips the toggle at once and runs the intent (state write + redraw).
            Toggle(isOn: isOn, intent: stateIntent(patch, action: element.action, payload: isOn ? "false" : "true")) {
                EmptyView()
            }
            // No `.invalidatableContent()` on control content: on macOS it made the system
            // stop treating the control as interactive (a tap just opened the app).
            .toggleStyle(TauriToggleStyle { toggleContent($0) })
        } else if let act = element.action, !act.isEmpty {
            Button(intent: actionIntent(act, payload: isOn ? "false" : "true")) {
                toggleContent(isOn)
            }.buttonStyle(.plain)
        } else {
            toggleContent(isOn)
        }
    }

    func toggleContent(_ isOn: Bool) -> some View {
        let onColor = resolveColor(element.tint) ?? .green
        let offColor = resolveColor(element.uncheckedTint) ?? .gray
        return HStack(spacing: 6) {
            switch element.variant {
            case "switch":
                Capsule()
                    .fill(isOn ? onColor : offColor.opacity(0.5))
                    .frame(width: 32, height: 18)
                    .overlay(alignment: isOn ? .trailing : .leading) {
                        Circle().fill(.white).padding(2)
                    }
            case "checkbox":
                Image(systemName: isOn ? "checkmark.square.fill" : "square")
                    .foregroundColor(isOn ? onColor : offColor)
                    .font(.system(size: 18))
            default:
                // Legacy round check (no variant) — kept for existing layouts.
                Image(systemName: isOn ? "checkmark.circle.fill" : "circle")
                    .foregroundColor(isOn ? onColor : offColor)
                    .font(.system(size: 18))
            }
            if let lbl = element.label {
                Text(lbl).font(.system(size: 14))
                    .foregroundColor(resolveColor(element.color) ?? .primary)
            }
        }
    }

    // MARK: Link

    @ViewBuilder func renderLink() -> some View {
        // Horizontal axis so nested text hugs (not maxWidth infinity → empty bg square).
        let content = HStack(spacing: 0) { renderChildren(axis: .horizontal) }
            .fixedSize(horizontal: true, vertical: true)
        if let patch = statePatch {
            Button(intent: stateIntent(patch, action: element.action)) { content }.buttonStyle(.plain)
        } else if let act = element.action, !act.isEmpty {
            Button(intent: actionIntent(act)) { content }.buttonStyle(.plain)
        } else if let u = element.url, !u.isEmpty, let url = URL(string: u) {
            Link(destination: url) { content }
        } else { content }
    }
}

/// Draws the existing toggle visuals for `Toggle(isOn:intent:)`.
struct TauriToggleStyle<V: View>: ToggleStyle {
    let draw: (Bool) -> V
    func makeBody(configuration: Configuration) -> some View { draw(configuration.isOn) }
}
