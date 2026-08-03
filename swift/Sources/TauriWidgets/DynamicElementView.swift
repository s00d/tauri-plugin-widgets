import SwiftUI

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
        case "vstack", "hstack", "zstack", "grid", "container":
            renderLayout()
        case "text", "label", "date", "timer":
            renderTextGroup()
        case "image", "shape", "canvas":
            renderMedia()
        case "progress", "gauge", "chart", "list":
            renderData()
        case "button", "toggle", "link":
            renderInteractive()
        case "spacer", "divider":
            renderSpacing()
        default:
            EmptyView()
        }
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

    func fontWeight(_ w: String?) -> Font.Weight {
        switch w {
        case "ultralight": return .ultraLight; case "thin": return .thin; case "light": return .light
        case "medium": return .medium; case "semibold": return .semibold; case "bold": return .bold
        case "heavy": return .heavy; case "black": return .black; default: return .regular
        }
    }

    func fontDesign(_ d: String?) -> Font.Design {
        switch d {
        case "monospaced": return .monospaced; case "rounded": return .rounded
        case "serif": return .serif; default: return .default
        }
    }

    func textStyleFont(_ style: String?) -> Font? {
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

    func parseAlignment(_ a: String?) -> Alignment {
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
