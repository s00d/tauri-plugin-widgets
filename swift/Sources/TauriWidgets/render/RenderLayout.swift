import SwiftUI

// MARK: - Containers

extension DynamicElementView {
    @ViewBuilder func renderLayout() -> some View {
        switch element.type {
        case "vstack":      renderVStack()
        case "hstack":      renderHStack()
        case "zstack":      renderZStack()
        case "grid":        renderGrid()
        case "container":   renderContainer()
        default: EmptyView()
        }
    }

    @ViewBuilder func renderVStack() -> some View {
        // Unset = stretch (desktop flex default, Glance Column Start): children start leading.
        let align: HorizontalAlignment = element.alignment == "center" ? .center
            : element.alignment == "trailing" ? .trailing : .leading
        if let j = Self.justifySpacers(element.justify) {
            VStack(alignment: align, spacing: 0) { renderJustified(axis: .vertical, j) }
        } else {
            VStack(alignment: align, spacing: element.spacing ?? 0) { renderChildren(axis: .vertical) }
        }
    }

    @ViewBuilder func renderHStack() -> some View {
        let align: VerticalAlignment = element.alignment == "top" ? .top
            : element.alignment == "bottom" ? .bottom : .center
        let stack = HStack(alignment: align, spacing: Self.justifySpacers(element.justify) == nil ? element.spacing ?? 0 : 0) {
            if let j = Self.justifySpacers(element.justify) {
                renderJustified(axis: .horizontal, j)
            } else {
                renderChildren(axis: .horizontal)
            }
        }
        // In a VStack: span width so flex:1 pushes trailing labels, but hug height
        // so padded cards don't eat leftover space and look top-heavy.
        // IMPORTANT: frame(maxWidth:) before fixedSize — otherwise the HStack stays
        // content-sized and `alignment: .leading` parks Today mid-row.
        // Content sits where the parent VStack puts it: leading unless the parent centers / trails.
        if parentAxis == .vertical {
            let rowAlign: Alignment = parentAlign == "center" ? .center
                : parentAlign == "trailing" ? .trailing : .leading
            stack
                .frame(maxWidth: .infinity, alignment: rowAlign)
                .fixedSize(horizontal: false, vertical: true)
        } else {
            stack
        }
    }

    @ViewBuilder func renderZStack() -> some View {
        // Full height only when a child fills it; a badge (sized circle + icon) hugs, or it
        // takes the whole column and starves sibling spacers (Android RenderLayout does the same).
        let fills = isWidgetRoot || element.children?.contains {
            Self.zstackChildFills($0) || $0.type == "vstack" || $0.type == "hstack"
        } ?? false
        ZStack(alignment: parseAlignment(element.alignment)) {
            if let children = element.children {
                ForEach(children.indices, id: \.self) { idx in
                    let child = children[idx]
                    let view = DynamicElementView(element: child, inZStack: true)
                    // Bare rectangle underlays fill here. Stacks fill after padding in applyStyle.
                    if Self.zstackChildFills(child) {
                        view.frame(maxWidth: .infinity, maxHeight: .infinity)
                    } else {
                        view
                    }
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: fills ? .infinity : nil)
    }

    /// Whether a ZStack child should expand to the proposed slot size.
    private static func zstackChildFills(_ child: WidgetElement) -> Bool {
        switch child.type {
        case "container", "grid", "zstack":
            return true
        case "vstack", "hstack":
            // Filled after PaddingMod in DynamicElementView.applyStyle.
            return false
        case "shape":
            let hasFrame = child.frame?.width != nil && child.frame?.height != nil
            let hasSize = child.size != nil
            if hasFrame || hasSize { return false }
            return ["rectangle", "roundedRectangle", "ellipse"].contains(child.shapeType ?? "rectangle")
        default:
            return false
        }
    }

    @ViewBuilder func renderGrid() -> some View {
        let cols = element.columns ?? 2; let sp = element.spacing ?? 4
        LazyVGrid(
            columns: Array(repeating: GridItem(.flexible(), spacing: sp), count: max(1, Int(cols))),
            spacing: element.rowSpacing ?? sp
        ) { renderChildren() }
    }

    @ViewBuilder func renderContainer() -> some View {
        let a = parseAlignment(element.contentAlignment)
        if let children = element.children, !children.isEmpty {
            ZStack(alignment: a) {
                ForEach(children.indices, id: \.self) { idx in
                    DynamicElementView(element: children[idx])
                }
            }
            // Expand so contentAlignment centers inside frame/padding, not intrinsic hug.
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: a)
        } else {
            EmptyView()
        }
    }

    /// `justify` as flexible-spacer counts (leading, between, trailing). Equal spacers share
    /// leftover space equally, so `spaceAround` = 1 / 2 / 1 halves the edge gaps.
    static func justifySpacers(_ j: String?) -> (lead: Int, between: Int, trail: Int)? {
        switch j {
        case "start": return (0, 0, 1)
        case "center": return (1, 0, 1)
        case "end": return (1, 0, 0)
        case "spaceBetween": return (0, 1, 0)
        case "spaceAround": return (1, 2, 1)
        case "spaceEvenly": return (1, 1, 1)
        default: return nil
        }
    }

    /// Children with spacers for `justify`; stack spacing becomes a fixed gap (spacing 0
    /// on the stack so adjacent spacers stay equal).
    @ViewBuilder func renderJustified(axis: Axis, _ j: (lead: Int, between: Int, trail: Int)) -> some View {
        let children = element.children ?? []
        let gap = element.spacing ?? 0
        ForEach(children.indices, id: \.self) { i in
            ForEach(0..<(i == 0 ? j.lead : j.between), id: \.self) { _ in Spacer(minLength: 0) }
            if i > 0 && gap > 0 {
                Color.clear.frame(width: axis == .horizontal ? gap : 0, height: axis == .vertical ? gap : 0)
            }
            DynamicElementView(element: children[i], parentAxis: axis,
                               parentAlign: axis == .vertical ? element.alignment : nil)
            if i == children.count - 1 {
                ForEach(0..<j.trail, id: \.self) { _ in Spacer(minLength: 0) }
            }
        }
    }

    @ViewBuilder func renderChildren(axis: Axis? = nil) -> some View {
        if let children = element.children {
            ForEach(children.indices, id: \.self) {
                DynamicElementView(element: children[$0], parentAxis: axis,
                                   parentAlign: axis == .vertical ? element.alignment : nil)
            }
        }
    }
}
