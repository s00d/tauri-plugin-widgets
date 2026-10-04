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
        let align: HorizontalAlignment = element.alignment == "leading" ? .leading
            : element.alignment == "trailing" ? .trailing : .center
        VStack(alignment: align, spacing: element.spacing ?? 0) { renderChildren(axis: .vertical) }
    }

    @ViewBuilder func renderHStack() -> some View {
        let align: VerticalAlignment = element.alignment == "top" ? .top
            : element.alignment == "bottom" ? .bottom : .center
        let stack = HStack(alignment: align, spacing: element.spacing ?? 0) {
            renderChildren(axis: .horizontal)
        }
        // In a VStack: span width so flex:1 pushes trailing labels, but hug height
        // so padded cards don't eat leftover space and look top-heavy.
        // IMPORTANT: frame(maxWidth:) before fixedSize — otherwise the HStack stays
        // content-sized and `alignment: .leading` parks Today mid-row.
        if parentAxis == .vertical {
            stack
                .frame(maxWidth: .infinity)
                .fixedSize(horizontal: false, vertical: true)
        } else {
            stack
        }
    }

    @ViewBuilder func renderZStack() -> some View {
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
        .frame(maxWidth: .infinity, maxHeight: .infinity)
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
            return (child.shapeType ?? "rectangle") == "rectangle"
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

    @ViewBuilder func renderChildren(axis: Axis? = nil) -> some View {
        if let children = element.children {
            ForEach(children.indices, id: \.self) {
                DynamicElementView(element: children[$0], parentAxis: axis)
            }
        }
    }
}
