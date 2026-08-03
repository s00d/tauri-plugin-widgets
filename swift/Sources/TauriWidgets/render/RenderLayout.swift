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
        HStack(alignment: align, spacing: element.spacing ?? 0) { renderChildren(axis: .horizontal) }
    }

    @ViewBuilder func renderZStack() -> some View {
        ZStack(alignment: parseAlignment(element.alignment)) {
            if let children = element.children {
                ForEach(children.indices, id: \.self) { idx in
                    DynamicElementView(element: children[idx], inZStack: true)
                }
            }
        }
    }

    @ViewBuilder func renderGrid() -> some View {
        let cols = element.columns ?? 2; let sp = element.spacing ?? 4
        LazyVGrid(
            columns: Array(repeating: GridItem(.flexible(), spacing: sp), count: Int(cols)),
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
