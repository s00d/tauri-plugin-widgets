import SwiftUI
import WidgetKit

// MARK: - View Modifiers

struct PaddingMod: ViewModifier {
    let p: PaddingValue?
    func body(content: Content) -> some View {
        switch p {
        case .uniform(let v): content.padding(v)
        case .edges(let t, let b, let l, let tr):
            content.padding(EdgeInsets(top: t ?? 0, leading: l ?? 0, bottom: b ?? 0, trailing: tr ?? 0))
        case nil: content
        }
    }
}

struct BgMod: ViewModifier {
    let bg: BackgroundValue?; let cr: CGFloat?
    func body(content: Content) -> some View {
        switch bg {
        case .solid(let hex):
            let c = Color.semantic(hex) ?? Color(hex: hex)
            // Fill the framed bounds (not the text glyph box) so 40×24 badges stay 40×24.
            content.background {
                RoundedRectangle(cornerRadius: cr ?? 0).fill(c)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        case .gradient(let g):
            content.background {
                g.filledShape(cornerRadius: cr ?? 0)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        case .adaptive(let l, let d):
            content.background {
                RoundedRectangle(cornerRadius: cr ?? 0)
                    .fill(Color.adaptive(light: l, dark: d))
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        case nil: content
        }
    }
}

struct ShadowMod: ViewModifier {
    let s: ShadowConfig?
    func body(content: Content) -> some View {
        if let s = s {
            content.shadow(color: s.color.map { Color(hex: $0) } ?? Color.black.opacity(0.3),
                           radius: s.radius ?? 4, x: s.x ?? 0, y: s.y ?? 2)
        } else { content }
    }
}

struct FrameMod: ViewModifier {
    let f: FrameConfig?
    func body(content: Content) -> some View {
        if let f = f {
            // Explicit center so badge/avatar content isn't left/top-biased inside the box.
            // fixedSize keeps accent bars (6×36) from being crushed by HStack flex siblings.
            content
                .frame(width: f.width, height: f.height, alignment: .center)
                .fixedSize(
                    horizontal: f.width != nil && f.maxWidth == nil,
                    vertical: f.height != nil && f.maxHeight == nil
                )
                .frame(maxWidth: f.maxWidth?.cgFloat, maxHeight: f.maxHeight?.cgFloat)
                .ifLet(f.minWidth != nil || f.minHeight != nil ? f : nil) {
                    $0.frame(minWidth: $1.minWidth, minHeight: $1.minHeight)
                }
        } else { content }
    }
}

struct BorderMod: ViewModifier {
    let b: BorderConfig?; let cr: CGFloat?
    func body(content: Content) -> some View {
        if let b = b {
            content.overlay(RoundedRectangle(cornerRadius: cr ?? 0)
                .stroke(Color(hex: b.color ?? "#888"), lineWidth: b.width ?? 1))
        } else { content }
    }
}

struct ClipShapeMod: ViewModifier {
    let shape: String?; let cr: CGFloat?
    func body(content: Content) -> some View {
        switch shape {
        case "circle": AnyView(content.clipShape(Circle()))
        case "capsule": AnyView(content.clipShape(Capsule()))
        case "rectangle", "roundedRectangle": AnyView(content.clipShape(RoundedRectangle(cornerRadius: cr ?? 0)))
        case "ellipse": AnyView(content.clipShape(Ellipse()))
        default: AnyView(content)
        }
    }
}

struct FlexMod: ViewModifier {
    let flex: CGFloat?
    func body(content: Content) -> some View {
        if (flex ?? 0) > 0 {
            // Take remaining HStack/VStack space. Do NOT raise layoutPriority —
            // that compresses trailing labels (Today / $15.99) to zero width.
            content.frame(maxWidth: .infinity)
        } else { content }
    }
}

struct OpacityMod: ViewModifier {
    let o: Double?
    func body(content: Content) -> some View {
        if let o = o { content.opacity(o) } else { content }
    }
}

/// `accessibilityLabel` / `privacySensitive` from the shared element style.
struct A11yMod: ViewModifier {
    let label: String?
    let privacy: Bool
    func body(content: Content) -> some View {
        let labeled: AnyView = {
            if let label, !label.isEmpty {
                return AnyView(content.accessibilityElement(children: .combine).accessibilityLabel(Text(label)))
            }
            return AnyView(content)
        }()
        if privacy { labeled.privacySensitive() } else { labeled }
    }
}

/// Post-layout style shared by every element: `margin` (outside background / border),
/// `rotation`, `scale`, `offsetX/Y`, `accentable`, `layoutPriority`. Unset = untouched.
struct PlacementMod: ViewModifier {
    let el: WidgetElement
    func body(content: Content) -> some View {
        content
            .ifLet(el.rotation) { $0.rotationEffect(.degrees($1)) }
            .ifLet(el.scale) { $0.scaleEffect($1) }
            .ifLet(el.offsetX != nil || el.offsetY != nil ? el : nil) {
                $0.offset(x: $1.offsetX ?? 0, y: $1.offsetY ?? 0)
            }
            .ifLet(el.margin) { $0.modifier(PaddingMod(p: $1)) }
            .ifLet(el.accentable == true ? true : nil) { v, _ in v.widgetAccentable() }
            .ifLet(el.layoutPriority) { $0.layoutPriority($1) }
    }
}

extension View {
    /// Apply `transform` only when `value` is set — keeps unset styles pixel-identical.
    @ViewBuilder
    func ifLet<T, V: View>(_ value: T?, _ transform: (Self, T) -> V) -> some View {
        if let value { transform(self, value) } else { self }
    }
}

/// `transition` hint (contract v3): `numeric` rolls changed digits, `opacity` / `slide`
/// cross-fade / push the content when `swapId` (the entry date at the root) changes,
/// `none` turns the system animation off. Unset = untouched.
struct TransitionMod<ID: Hashable>: ViewModifier {
    let kind: String?
    var swapId: ID? = nil
    func body(content: Content) -> some View {
        switch kind {
        case "numeric": content.contentTransition(.numericText())
        case "opacity": content.transition(.opacity).ifLet(swapId) { $0.id($1) }
        case "slide": content.transition(.push(from: .bottom)).ifLet(swapId) { $0.id($1) }
        case "none": content.contentTransition(.identity).transaction { $0.animation = nil }
        default: content
        }
    }
}

extension TransitionMod where ID == Int {
    init(kind: String?) { self.kind = kind }
}
