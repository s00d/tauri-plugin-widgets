import SwiftUI

// MARK: - Recursive Renderer

public struct DynamicElementView: View {
    public let element: WidgetElement
    var parentAxis: Axis? = nil
    /// Parent VStack `alignment` (nil = stretch) — places full-width rows' content.
    var parentAlign: String? = nil
    /// When true, text must not expand to maxWidth (breaks ZStack badge centering).
    var inZStack: Bool = false
    /// WidgetKit root: `containerBackground` already paints the canvas — skip
    /// root `background`/`shadow` or a content-sized card floats inside (looks “crooked”).
    var isWidgetRoot: Bool = false
    /// Set by `TauriWidgetView` from the timeline entry (see `TauriWidgetEntry`).
    @Environment(\.tauriWidgetIdentity) var identity

    /// Intent tagged with this widget's own appGroup / widgetId.
    func actionIntent(_ name: String, payload: String? = nil) -> WidgetActionIntent {
        WidgetActionIntent(
            actionName: name, payload: payload,
            widgetId: identity?.widgetId, appGroup: identity?.appGroup
        )
    }

    /// `set` + optional `action` as one in-extension state write (contract v3).
    func stateIntent(_ patch: [String: Any], action: String? = nil, payload: String? = nil) -> WidgetStateIntent {
        WidgetStateIntent(
            patch: patch, actionName: action, payload: payload,
            widgetId: identity?.widgetId, instance: identity?.instance, appGroup: identity?.appGroup
        )
    }

    /// The element's `set` object, when non-empty.
    var statePatch: [String: Any]? {
        guard let m = element.set?.values, !m.isEmpty else { return nil }
        return m
    }

    public init(
        element: WidgetElement,
        parentAxis: Axis? = nil,
        parentAlign: String? = nil,
        inZStack: Bool = false,
        isWidgetRoot: Bool = false
    ) {
        self.element = element
        self.parentAxis = parentAxis
        self.parentAlign = parentAlign
        self.inZStack = inZStack
        self.isWidgetRoot = isWidgetRoot
    }

    public var body: some View {
        applyStyle(to: renderElement(), element: element)
            .modifier(A11yMod(label: element.accessibilityLabel, privacy: element.privacySensitive == true))
            .modifier(TransitionMod(kind: element.transition))
    }

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
        let isStack = el.type == "vstack" || el.type == "hstack"
        // ZStack stacks: padding THEN fill — so spacers expand inside the inset,
        // and corner badges aren't flush under the chrome clip.
        let base = view
            .modifier(FlexMod(flex: el.flex))
            .modifier(PaddingMod(p: outerPad))
            .ifLet(el.aspectRatio) { $0.aspectRatio(CGFloat($1), contentMode: .fit) }
            .modifier(FrameMod(f: el.frame))
        let styled: AnyView = {
            if inZStack && isStack {
                return AnyView(
                    base.frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
                )
            }
            return AnyView(base)
        }()
        if isWidgetRoot {
            // Keep padding/frame; chrome comes from WidgetKit containerBackground.
            // A root border outlines the whole canvas (desktop/Android), not the content box.
            styled
                .ifLet(el.border) { v, _ in v.frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading) }
                .modifier(BorderMod(b: el.border, cr: el.cornerRadius))
                .modifier(OpacityMod(o: el.opacity))
                .modifier(PlacementMod(el: el))
        } else {
            styled
                .modifier(BgMod(bg: el.background, cr: el.cornerRadius))
                .modifier(BorderMod(b: el.border, cr: el.cornerRadius))
                .modifier(ClipShapeMod(shape: el.clipShape, cr: el.cornerRadius))
                .modifier(OpacityMod(o: el.opacity))
                .modifier(ShadowMod(s: el.shadow))
                .modifier(PlacementMod(el: el))
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

/// Type-erased `Encodable` (an existential cannot be passed to `JSONEncoder.encode`).
struct AnyEncodable: Encodable {
    let value: Encodable
    init(_ value: Encodable) { self.value = value }
    func encode(to encoder: Encoder) throws { try value.encode(to: encoder) }
}
