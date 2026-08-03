import SwiftUI

// MARK: - Color helpers

extension Color {
    public init(hex: String) {
        var h = hex.trimmingCharacters(in: CharacterSet(charactersIn: "#"))
        if h.count == 3 || h.count == 4 {
            h = h.map { "\($0)\($0)" }.joined()
        }
        var rgb: UInt64 = 0; Scanner(string: h).scanHexInt64(&rgb)
        let r, g, b, a: Double
        switch h.count {
        case 6:
            r = Double((rgb >> 16) & 0xFF) / 255
            g = Double((rgb >> 8) & 0xFF) / 255
            b = Double(rgb & 0xFF) / 255; a = 1
        case 8:
            r = Double((rgb >> 24) & 0xFF) / 255
            g = Double((rgb >> 16) & 0xFF) / 255
            b = Double((rgb >> 8) & 0xFF) / 255
            a = Double(rgb & 0xFF) / 255
        default:
            r = 0; g = 0; b = 0; a = 1
        }
        self.init(red: r, green: g, blue: b, opacity: a)
    }

    static func adaptive(light: String, dark: String) -> Color {
        #if canImport(UIKit)
        return Color(UIColor { traits in
            traits.userInterfaceStyle == .dark
                ? UIColor(Color(hex: dark))
                : UIColor(Color(hex: light))
        })
        #elseif canImport(AppKit)
        return Color(NSColor(name: nil) { appearance in
            appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
                ? NSColor(Color(hex: dark))
                : NSColor(Color(hex: light))
        })
        #else
        return Color(hex: light)
        #endif
    }

    static func semantic(_ name: String) -> Color? {
        switch name {
        case "label": return .primary
        case "secondaryLabel": return .secondary
        case "accent": return .accentColor
        #if canImport(UIKit)
        case "systemBackground": return Color(UIColor.systemBackground)
        case "secondarySystemBackground": return Color(UIColor.secondarySystemBackground)
        case "separator": return Color(UIColor.separator)
        #elseif canImport(AppKit)
        case "systemBackground": return Color(NSColor.windowBackgroundColor)
        case "secondarySystemBackground": return Color(NSColor.controlBackgroundColor)
        case "separator": return Color(NSColor.separatorColor)
        #endif
        default: return nil
        }
    }
}

func resolveColor(_ cv: ColorValue?) -> Color? {
    guard let cv = cv else { return nil }
    switch cv {
    case .solid(let s):
        if let sem = Color.semantic(s) { return sem }
        return Color(hex: s)
    case .adaptive(let l, let d):
        return Color.adaptive(light: l, dark: d)
    }
}
