import Foundation
import SwiftUI
import WidgetKit

/// Shared WidgetKit-like chrome for production widget + visual tests.
public enum WidgetChrome {
    @ViewBuilder
    public static func background(for el: WidgetElement) -> some View {
        switch el.background {
        case .solid(let hex):
            if let sem = Color.semantic(hex) { sem } else { Color(hex: hex) }
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
            LinearGradient(colors: colors, startPoint: s, endPoint: e)
        case .adaptive(let light, let dark):
            Color.adaptive(light: light, dark: dark)
        case nil:
            #if os(macOS)
            Color(.windowBackgroundColor)
            #else
            Color(.systemBackground)
            #endif
        }
    }

    /// Fixed-size widget frame with containerBackground + continuous corner mask.
    @ViewBuilder
    public static func framed(
        element: WidgetElement,
        width: CGFloat,
        height: CGFloat,
        cornerRadius: CGFloat = 22
    ) -> some View {
        DynamicElementView(element: element)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .frame(width: width, height: height)
            .containerBackground(for: .widget) { background(for: element) }
            .clipShape(RoundedRectangle(cornerRadius: cornerRadius, style: .continuous))
    }
}
