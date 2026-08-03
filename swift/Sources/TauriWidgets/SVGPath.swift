import SwiftUI

/// Minimal SVG path parser (M/m L/l H/h V/v C/c Q/q A/a Z/z).
/// Accepts compact syntax like `M10 10L20 20` (command letter glued to first number).
func parseSVGPath(_ data: String) -> Path {
    var path = Path()
    var tokens: [String] = []
    var num = ""
    func flushNum() {
        if !num.isEmpty { tokens.append(num); num = "" }
    }
    for ch in data.replacingOccurrences(of: ",", with: " ") {
        if ch.isLetter {
            flushNum()
            tokens.append(String(ch))
        } else if ch.isWhitespace {
            flushNum()
        } else if ch == "-" || ch == "+" || ch == "." || ch.isNumber {
            // Start a new number when sign follows a digit/dot (e.g. 10-5 → 10, -5).
            if (ch == "-" || ch == "+"), !num.isEmpty {
                flushNum()
            }
            num.append(ch)
        } else {
            flushNum()
        }
    }
    flushNum()
    var i = 0
    var cx: CGFloat = 0
    var cy: CGFloat = 0
    var lastCubicCtrl: CGPoint?
    var lastQuadCtrl: CGPoint?
    func nextNumber() -> CGFloat? {
        guard i < tokens.count, let v = Double(tokens[i]) else { return nil }
        i += 1
        return CGFloat(v)
    }
    while i < tokens.count {
        let cmd = tokens[i]
        if cmd.count == 1, let c = cmd.first, c.isLetter {
            i += 1
            switch c {
            case "M":
                lastCubicCtrl = nil; lastQuadCtrl = nil
                var firstMove = true
                while let x = nextNumber(), let y = nextNumber() {
                    if firstMove {
                        cx = x; cy = y; path.move(to: CGPoint(x: x, y: y))
                        firstMove = false
                    } else {
                        cx = x; cy = y; path.addLine(to: CGPoint(x: x, y: y))
                    }
                }
            case "m":
                lastCubicCtrl = nil; lastQuadCtrl = nil
                var firstMove = true
                while let x = nextNumber(), let y = nextNumber() {
                    if firstMove {
                        cx += x; cy += y; path.move(to: CGPoint(x: cx, y: cy))
                        firstMove = false
                    } else {
                        cx += x; cy += y; path.addLine(to: CGPoint(x: cx, y: cy))
                    }
                }
            case "L":
                lastCubicCtrl = nil; lastQuadCtrl = nil
                while let x = nextNumber(), let y = nextNumber() {
                    cx = x; cy = y; path.addLine(to: CGPoint(x: x, y: y))
                }
            case "l":
                lastCubicCtrl = nil; lastQuadCtrl = nil
                while let x = nextNumber(), let y = nextNumber() {
                    cx += x; cy += y; path.addLine(to: CGPoint(x: cx, y: cy))
                }
            case "H":
                lastCubicCtrl = nil; lastQuadCtrl = nil
                while let x = nextNumber() { cx = x; path.addLine(to: CGPoint(x: cx, y: cy)) }
            case "h":
                lastCubicCtrl = nil; lastQuadCtrl = nil
                while let x = nextNumber() { cx += x; path.addLine(to: CGPoint(x: cx, y: cy)) }
            case "V":
                lastCubicCtrl = nil; lastQuadCtrl = nil
                while let y = nextNumber() { cy = y; path.addLine(to: CGPoint(x: cx, y: cy)) }
            case "v":
                lastCubicCtrl = nil; lastQuadCtrl = nil
                while let y = nextNumber() { cy += y; path.addLine(to: CGPoint(x: cx, y: cy)) }
            case "Z", "z":
                lastCubicCtrl = nil; lastQuadCtrl = nil
                path.closeSubpath()
            case "C":
                while let x1 = nextNumber(), let y1 = nextNumber(),
                      let x2 = nextNumber(), let y2 = nextNumber(),
                      let x = nextNumber(), let y = nextNumber() {
                    path.addCurve(
                        to: CGPoint(x: x, y: y),
                        control1: CGPoint(x: x1, y: y1),
                        control2: CGPoint(x: x2, y: y2)
                    )
                    lastCubicCtrl = CGPoint(x: x2, y: y2)
                    lastQuadCtrl = nil
                    cx = x; cy = y
                }
            case "c":
                while let dx1 = nextNumber(), let dy1 = nextNumber(),
                      let dx2 = nextNumber(), let dy2 = nextNumber(),
                      let dx = nextNumber(), let dy = nextNumber() {
                    let x1 = cx + dx1; let y1 = cy + dy1
                    let x2 = cx + dx2; let y2 = cy + dy2
                    let x = cx + dx; let y = cy + dy
                    path.addCurve(
                        to: CGPoint(x: x, y: y),
                        control1: CGPoint(x: x1, y: y1),
                        control2: CGPoint(x: x2, y: y2)
                    )
                    lastCubicCtrl = CGPoint(x: x2, y: y2)
                    lastQuadCtrl = nil
                    cx = x; cy = y
                }
            case "S":
                while let x2 = nextNumber(), let y2 = nextNumber(),
                      let x = nextNumber(), let y = nextNumber() {
                    let x1 = lastCubicCtrl.map { 2 * cx - $0.x } ?? cx
                    let y1 = lastCubicCtrl.map { 2 * cy - $0.y } ?? cy
                    path.addCurve(
                        to: CGPoint(x: x, y: y),
                        control1: CGPoint(x: x1, y: y1),
                        control2: CGPoint(x: x2, y: y2)
                    )
                    lastCubicCtrl = CGPoint(x: x2, y: y2)
                    lastQuadCtrl = nil
                    cx = x; cy = y
                }
            case "s":
                while let dx2 = nextNumber(), let dy2 = nextNumber(),
                      let dx = nextNumber(), let dy = nextNumber() {
                    let x1 = lastCubicCtrl.map { 2 * cx - $0.x } ?? cx
                    let y1 = lastCubicCtrl.map { 2 * cy - $0.y } ?? cy
                    let x2 = cx + dx2; let y2 = cy + dy2
                    let x = cx + dx; let y = cy + dy
                    path.addCurve(
                        to: CGPoint(x: x, y: y),
                        control1: CGPoint(x: x1, y: y1),
                        control2: CGPoint(x: x2, y: y2)
                    )
                    lastCubicCtrl = CGPoint(x: x2, y: y2)
                    lastQuadCtrl = nil
                    cx = x; cy = y
                }
            case "Q":
                while let x1 = nextNumber(), let y1 = nextNumber(),
                      let x = nextNumber(), let y = nextNumber() {
                    path.addQuadCurve(to: CGPoint(x: x, y: y), control: CGPoint(x: x1, y: y1))
                    lastQuadCtrl = CGPoint(x: x1, y: y1)
                    lastCubicCtrl = nil
                    cx = x; cy = y
                }
            case "q":
                while let dx1 = nextNumber(), let dy1 = nextNumber(),
                      let dx = nextNumber(), let dy = nextNumber() {
                    let x1 = cx + dx1; let y1 = cy + dy1
                    let x = cx + dx; let y = cy + dy
                    path.addQuadCurve(to: CGPoint(x: x, y: y), control: CGPoint(x: x1, y: y1))
                    lastQuadCtrl = CGPoint(x: x1, y: y1)
                    lastCubicCtrl = nil
                    cx = x; cy = y
                }
            case "T":
                while let x = nextNumber(), let y = nextNumber() {
                    let x1 = lastQuadCtrl.map { 2 * cx - $0.x } ?? cx
                    let y1 = lastQuadCtrl.map { 2 * cy - $0.y } ?? cy
                    path.addQuadCurve(to: CGPoint(x: x, y: y), control: CGPoint(x: x1, y: y1))
                    lastQuadCtrl = CGPoint(x: x1, y: y1)
                    lastCubicCtrl = nil
                    cx = x; cy = y
                }
            case "t":
                while let dx = nextNumber(), let dy = nextNumber() {
                    let x1 = lastQuadCtrl.map { 2 * cx - $0.x } ?? cx
                    let y1 = lastQuadCtrl.map { 2 * cy - $0.y } ?? cy
                    let x = cx + dx; let y = cy + dy
                    path.addQuadCurve(to: CGPoint(x: x, y: y), control: CGPoint(x: x1, y: y1))
                    lastQuadCtrl = CGPoint(x: x1, y: y1)
                    lastCubicCtrl = nil
                    cx = x; cy = y
                }
            case "A":
                lastCubicCtrl = nil; lastQuadCtrl = nil
                while let rx = nextNumber(), let ry = nextNumber(),
                      let rot = nextNumber(), let large = nextNumber(), let sweep = nextNumber(),
                      let x = nextNumber(), let y = nextNumber() {
                    addSVGArc(&path, &cx, &cy, rx: rx, ry: ry, rotation: rot,
                              largeArc: large != 0, sweep: sweep != 0, end: CGPoint(x: x, y: y))
                }
            case "a":
                lastCubicCtrl = nil; lastQuadCtrl = nil
                while let rx = nextNumber(), let ry = nextNumber(),
                      let rot = nextNumber(), let large = nextNumber(), let sweep = nextNumber(),
                      let dx = nextNumber(), let dy = nextNumber() {
                    let end = CGPoint(x: cx + dx, y: cy + dy)
                    addSVGArc(&path, &cx, &cy, rx: rx, ry: ry, rotation: rot,
                              largeArc: large != 0, sweep: sweep != 0, end: end)
                }
            default:
                break
            }
        } else {
            i += 1
        }
    }
    return path
}

/// Approximate SVG elliptical arc as a cubic (good enough for widget canvas glyphs).
func addSVGArc(
    _ path: inout Path,
    _ cx: inout CGFloat,
    _ cy: inout CGFloat,
    rx: CGFloat,
    ry: CGFloat,
    rotation: CGFloat,
    largeArc: Bool,
    sweep: Bool,
    end: CGPoint
) {
    let start = CGPoint(x: cx, y: cy)
    let rrx = abs(rx) < 0.001 ? 0.001 : abs(rx)
    let rry = abs(ry) < 0.001 ? 0.001 : abs(ry)
    // Degenerate: line
    if abs(start.x - end.x) < 0.001, abs(start.y - end.y) < 0.001 {
        return
    }
    // Midpoint-ish control using radii (not a full SVG arc convert — widget fidelity).
    let mid = CGPoint(x: (start.x + end.x) / 2, y: (start.y + end.y) / 2)
    let dx = end.x - start.x
    let dy = end.y - start.y
    let len = max(hypot(dx, dy), 0.001)
    let nx = -dy / len
    let ny = dx / len
    let bulge = (largeArc ? 1.0 : 0.35) * min(rrx, rry) * (sweep ? 1.0 : -1.0)
    let _ = rotation // rotation ignored in this approximation
    let ctrl = CGPoint(x: mid.x + nx * bulge, y: mid.y + ny * bulge)
    path.addQuadCurve(to: end, control: ctrl)
    cx = end.x
    cy = end.y
}
