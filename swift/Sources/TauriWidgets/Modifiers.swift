import SwiftUI

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
            content.background(RoundedRectangle(cornerRadius: cr ?? 0).fill(c))
        case .gradient(let g):
            content.background(g.filledShape(cornerRadius: cr ?? 0))
        case .adaptive(let l, let d):
            content.background(RoundedRectangle(cornerRadius: cr ?? 0)
                .fill(Color.adaptive(light: l, dark: d)))
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
            content.frame(width: f.width, height: f.height, alignment: .center)
                .frame(maxWidth: f.maxWidth?.cgFloat, maxHeight: f.maxHeight?.cgFloat)
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
        case "rectangle": AnyView(content.clipShape(RoundedRectangle(cornerRadius: cr ?? 0)))
        default: AnyView(content)
        }
    }
}

struct FlexMod: ViewModifier {
    let flex: CGFloat?
    func body(content: Content) -> some View {
        if let f = flex, f > 0 {
            content.frame(maxWidth: .infinity).layoutPriority(Double(f))
        } else { content }
    }
}

struct OpacityMod: ViewModifier {
    let o: Double?
    func body(content: Content) -> some View {
        if let o = o { content.opacity(o) } else { content }
    }
}
