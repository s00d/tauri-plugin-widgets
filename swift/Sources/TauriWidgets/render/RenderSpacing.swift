import SwiftUI

// MARK: - Divider / Spacer

extension DynamicElementView {
    @ViewBuilder func renderSpacing() -> some View {
        switch element.type {
        case "spacer":  renderSpacer()
        case "divider": renderDivider()
        default: EmptyView()
        }
    }

    @ViewBuilder func renderDivider() -> some View {
        let fill = Rectangle().fill(resolveColor(element.color) ?? Color.gray.opacity(0.3))
        let thickness = element.thickness ?? 1
        if parentAxis == .horizontal {
            fill.frame(width: thickness)
        } else {
            fill.frame(height: thickness)
                .padding(.vertical, 4)
        }
    }

    @ViewBuilder func renderSpacer() -> some View {
        if let ml = element.minLength { Spacer(minLength: ml) } else { Spacer() }
    }
}
