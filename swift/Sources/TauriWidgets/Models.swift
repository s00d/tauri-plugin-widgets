import Foundation
import SwiftUI

// MARK: - Top-level config
// Wire `type` enum catalog: Generated/WireCatalog.swift (`pnpm codegen`).

public struct WidgetUIConfig: Codable {
    public let version: Int?
    public let small: WidgetElement?
    public let medium: WidgetElement?
    public let large: WidgetElement?
    /// Lock Screen / watch-style families (`accessoryCircular` / `Rectangular` / `Inline`).
    public let accessory: WidgetElement?
    /// How content changes between entries: `none` | `opacity` | `slide` | `numeric`.
    public let transition: String?

    public init(version: Int? = nil, small: WidgetElement? = nil,
                medium: WidgetElement? = nil, large: WidgetElement? = nil,
                accessory: WidgetElement? = nil, transition: String? = nil) {
        self.version = version; self.small = small; self.medium = medium; self.large = large
        self.accessory = accessory; self.transition = transition
    }
}

// MARK: - Color value (hex, semantic name, or adaptive pair)

public enum ColorValue: Codable {
    case solid(String)
    case adaptive(light: String, dark: String)

    public init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if let s = try? c.decode(String.self) { self = .solid(s); return }
        let obj = try AdaptivePair(from: decoder)
        self = .adaptive(light: obj.light, dark: obj.dark)
    }

    public func encode(to encoder: Encoder) throws {
        switch self {
        case .solid(let s):
            var c = encoder.singleValueContainer(); try c.encode(s)
        case .adaptive(let l, let d):
            try AdaptivePair(light: l, dark: d).encode(to: encoder)
        }
    }

    private struct AdaptivePair: Codable { let light: String; let dark: String }
}

// MARK: - Element

// `WidgetElement` is generated from the Rust IR: Generated/WidgetElement.swift (`pnpm codegen`).


// MARK: - Supporting types

public enum BackgroundValue: Codable {
    case solid(String)
    case gradient(GradientConfig)
    case adaptive(light: String, dark: String)

    public init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if let s = try? c.decode(String.self) { self = .solid(s); return }
        if let g = try? GradientConfig(from: decoder), !g.isEmpty { self = .gradient(g); return }
        let obj = try AdaptivePair(from: decoder)
        self = .adaptive(light: obj.light, dark: obj.dark)
    }

    public func encode(to encoder: Encoder) throws {
        switch self {
        case .solid(let s): var c = encoder.singleValueContainer(); try c.encode(s)
        case .gradient(let g): try g.encode(to: encoder)
        case .adaptive(let l, let d): try AdaptivePair(light: l, dark: d).encode(to: encoder)
        }
    }

    public var solidColor: String? {
        if case .solid(let c) = self { return c }; return nil
    }

    private struct AdaptivePair: Codable { let light: String; let dark: String }
}

public struct GradientConfig: Codable {
    public let gradientType: String
    public let colors: [String]
    public let direction: String?
    /// Colors at explicit positions; replace `colors` when set.
    public let stops: [GradientStop]?
    /// Unit-space start / end points; override `direction` (linear only).
    public let startPoint: GradientPoint?
    public let endPoint: GradientPoint?

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        gradientType = try c.decode(String.self, forKey: .gradientType)
        colors = try c.decodeIfPresent([String].self, forKey: .colors) ?? []
        direction = try c.decodeIfPresent(String.self, forKey: .direction)
        stops = try c.decodeIfPresent([GradientStop].self, forKey: .stops)
        startPoint = try c.decodeIfPresent(GradientPoint.self, forKey: .startPoint)
        endPoint = try c.decodeIfPresent(GradientPoint.self, forKey: .endPoint)
    }

    /// No colors at all (neither `colors` nor `stops`): not a gradient.
    public var isEmpty: Bool { colors.isEmpty && (stops ?? []).isEmpty }

    public var gradient: Gradient {
        if let stops, !stops.isEmpty {
            return Gradient(stops: stops.map { .init(color: Color(hex: $0.color), location: $0.location) })
        }
        return Gradient(colors: colors.map { Color(hex: $0) })
    }

    public var linearStart: UnitPoint {
        if let p = startPoint { return UnitPoint(x: p.x, y: p.y) }
        switch direction {
        case "bottomToTop": return .bottom
        case "leadingToTrailing": return .leading
        case "trailingToLeading": return .trailing
        case "topLeadingToBottomTrailing": return .topLeading
        case "topTrailingToBottomLeading": return .topTrailing
        default: return .top
        }
    }

    public var linearEnd: UnitPoint {
        if let p = endPoint { return UnitPoint(x: p.x, y: p.y) }
        switch direction {
        case "bottomToTop": return .top
        case "leadingToTrailing": return .trailing
        case "trailingToLeading": return .leading
        case "topLeadingToBottomTrailing": return .bottomTrailing
        case "topTrailingToBottomLeading": return .bottomLeading
        default: return .bottom
        }
    }

    @ViewBuilder
    public func asView() -> some View {
        switch gradientType.lowercased() {
        case "radial":
            RadialGradient(gradient: gradient, center: .center, startRadius: 0, endRadius: 160)
        case "angular":
            AngularGradient(gradient: gradient, center: .center)
        default:
            LinearGradient(gradient: gradient, startPoint: linearStart, endPoint: linearEnd)
        }
    }

    @ViewBuilder
    public func filledShape(cornerRadius: CGFloat) -> some View {
        let shape = RoundedRectangle(cornerRadius: cornerRadius)
        switch gradientType.lowercased() {
        case "radial":
            shape.fill(RadialGradient(gradient: gradient, center: .center, startRadius: 0, endRadius: 160))
        case "angular":
            shape.fill(AngularGradient(gradient: gradient, center: .center))
        default:
            shape.fill(LinearGradient(gradient: gradient, startPoint: linearStart, endPoint: linearEnd))
        }
    }
}

public struct GradientStop: Codable {
    public let color: String
    public let location: CGFloat
}

/// Unit-space point (`0…1`, origin top leading).
public struct GradientPoint: Codable {
    public let x: CGFloat
    public let y: CGFloat
}

public struct ShadowConfig: Codable {
    public let color: String?
    public let radius: CGFloat?
    public let x: CGFloat?
    public let y: CGFloat?
}

public enum PaddingValue: Codable {
    case uniform(CGFloat)
    case edges(top: CGFloat?, bottom: CGFloat?, leading: CGFloat?, trailing: CGFloat?)

    public init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if let v = try? c.decode(CGFloat.self) { self = .uniform(v); return }
        let obj = try PaddingEdges(from: decoder)
        self = .edges(top: obj.top, bottom: obj.bottom, leading: obj.leading, trailing: obj.trailing)
    }

    public func encode(to encoder: Encoder) throws {
        switch self {
        case .uniform(let v): var c = encoder.singleValueContainer(); try c.encode(v)
        case .edges(let t, let b, let l, let tr):
            try PaddingEdges(top: t, bottom: b, leading: l, trailing: tr).encode(to: encoder)
        }
    }

    private struct PaddingEdges: Codable {
        let top: CGFloat?; let bottom: CGFloat?; let leading: CGFloat?; let trailing: CGFloat?
    }
}

public struct FrameConfig: Codable {
    public let width: CGFloat?
    public let height: CGFloat?
    public let maxWidth: FlexDimension?
    public let maxHeight: FlexDimension?
    public let minWidth: CGFloat?
    public let minHeight: CGFloat?
}

public enum FlexDimension: Codable {
    case fixed(CGFloat)
    case infinity

    public init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if let s = try? c.decode(String.self), s == "infinity" { self = .infinity }
        else { self = .fixed(try c.decode(CGFloat.self)) }
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.singleValueContainer()
        switch self { case .fixed(let v): try c.encode(v); case .infinity: try c.encode("infinity") }
    }

    public var cgFloat: CGFloat? {
        switch self { case .fixed(let v): return v; case .infinity: return .infinity }
    }
}

public struct BorderConfig: Codable {
    public let color: String?
    public let width: CGFloat?
}

public struct ChartDataPoint: Codable {
    public let label: String
    public let value: Double
    public let color: ColorValue?
    /// Points with the same series form one line / bar group.
    public let series: String?
}

public struct ListItem: Codable {
    public let text: String
    public let checked: Bool?
    public let action: String?
    public let payload: String?
}

public struct CanvasDrawCommand: Codable {
    public let draw: String
    public let cx: CGFloat?; public let cy: CGFloat?; public let r: CGFloat?
    public let x: CGFloat?; public let y: CGFloat?
    public let x1: CGFloat?; public let y1: CGFloat?
    public let x2: CGFloat?; public let y2: CGFloat?
    public let width: CGFloat?; public let height: CGFloat?
    public let startAngle: CGFloat?; public let endAngle: CGFloat?
    public let fill: ColorValue?; public let stroke: ColorValue?; public let strokeWidth: CGFloat?
    public let lineCap: String?; public let cornerRadius: CGFloat?
    public let content: String?; public let fontSize: CGFloat?
    public let color: ColorValue?; public let anchor: String?
    public let d: String?
}
