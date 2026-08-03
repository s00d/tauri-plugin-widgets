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
            g.asView()
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

    /// Fixed-size widget frame with full-bleed chrome + continuous corner mask.
    /// Used by visual tests / ImageRenderer (no live WidgetKit containerBackground).
    @ViewBuilder
    public static func framed(
        element: WidgetElement,
        width: CGFloat,
        height: CGFloat,
        cornerRadius: CGFloat = 22
    ) -> some View {
        ZStack(alignment: .topLeading) {
            background(for: element)
            DynamicElementView(element: element, isWidgetRoot: true)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
        .frame(width: width, height: height)
        .clipShape(RoundedRectangle(cornerRadius: cornerRadius, style: .continuous))
    }
}
