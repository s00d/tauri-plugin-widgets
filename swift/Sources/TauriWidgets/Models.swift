import Foundation
import SwiftUI

// MARK: - Top-level config

public struct WidgetUIConfig: Codable {
    public let version: Int?
    public let small: WidgetElement?
    public let medium: WidgetElement?
    public let large: WidgetElement?

    public init(version: Int? = nil, small: WidgetElement? = nil,
                medium: WidgetElement? = nil, large: WidgetElement? = nil) {
        self.version = version; self.small = small; self.medium = medium; self.large = large
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

// MARK: - Element (flat Codable struct)

public struct WidgetElement: Codable {
    public let type: String

    // Container
    public let contentAlignment: String?

    // Stack containers
    public let children: [WidgetElement]?
    public let spacing: CGFloat?
    public let alignment: String?
    public let columns: Int?
    public let rowSpacing: CGFloat?

    // Text / Label
    public let content: String?
    public let text: String?
    public let fontSize: CGFloat?
    public let fontWeight: String?
    public let fontDesign: String?
    public let textStyle: String?
    public let lineLimit: Int?

    // Image / Label
    public let systemName: String?
    public let data: String?
    public let url: String?
    public let size: CGFloat?
    public let contentMode: String?
    public let iconColor: ColorValue?

    // Progress / Gauge
    public let value: Double?
    public let total: Double?
    public let barStyle: String?
    public let tint: ColorValue?
    public let label: String?
    public let min: Double?
    public let max: Double?
    public let currentValueLabel: String?
    public let gaugeStyle: String?

    // Button
    public let backgroundColor: ColorValue?
    public let textAlignment: String?

    // Toggle
    public let isOn: Bool?
    public let action: String?

    // Divider
    public let thickness: CGFloat?

    // Spacer
    public let minLength: CGFloat?

    // Date
    public let date: String?
    public let dateStyle: String?

    // Chart
    public let chartType: String?
    public let chartData: [ChartDataPoint]?
    public let items: [ListItem]?

    // Shape
    public let shapeType: String?
    public let fill: ColorValue?
    public let stroke: ColorValue?
    public let strokeWidth: CGFloat?

    // Timer
    public let targetDate: String?
    public let counting: String?

    // Canvas
    public let width: CGFloat?
    public let height: CGFloat?
    public let elements: [CanvasDrawCommand]?

    // Common style
    public let color: ColorValue?
    public let padding: PaddingValue?
    public let background: BackgroundValue?
    public let cornerRadius: CGFloat?
    public let opacity: Double?
    public let frame: FrameConfig?
    public let border: BorderConfig?
    public let shadow: ShadowConfig?
    public let clipShape: String?
    public let flex: CGFloat?
}

// MARK: - Supporting types

public enum BackgroundValue: Codable {
    case solid(String)
    case gradient(GradientConfig)
    case adaptive(light: String, dark: String)

    public init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if let s = try? c.decode(String.self) { self = .solid(s); return }
        if let g = try? GradientConfig(from: decoder), !g.colors.isEmpty { self = .gradient(g); return }
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
