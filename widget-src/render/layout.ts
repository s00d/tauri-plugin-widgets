import { applyStyle } from "../style";
import { renderEl } from "./child";
import type { ElNode, ParentAxis } from "../types";

/**
 * SwiftUI flexibility: does `c` take all the space offered along `dim` ("w" / "h") when its
 * parent lays out on `parentAxis`? Flexible views share an HStack's leftover width equally
 * and fill its height; a stack is flexible when any child is (text spans a VStack's width).
 */
export function greedy(c: ElNode, dim: "w" | "h", parentAxis: ParentAxis): boolean {
  if (dim === "w" && c.flex > 0) return true;
  // `.frame(maxWidth/maxHeight: .infinity)` takes all it is offered.
  if (c.frame && c.frame[dim === "w" ? "maxWidth" : "maxHeight"] === "infinity") return true;
  if (c.frame && c.frame[dim === "w" ? "width" : "height"]) return false;
  var kids = function (axis: ParentAxis) {
    return (c.children || []).some(function (k: ElNode) {
      return greedy(k, dim, axis);
    });
  };
  switch (c.type) {
    case "spacer":
      return (dim === "w") === (parentAxis === "horizontal");
    case "divider":
      return (dim === "h") === (parentAxis === "horizontal");
    case "text":
    case "label":
    case "date":
    case "timer":
      return dim === "w" && parentAxis === "vertical" && !(c.frame && c.frame.width && c.frame.height);
    case "button":
      return dim === "w";
    case "progress":
      return dim === "w" && c.barStyle !== "circular";
    case "container":
    case "zstack":
      return true;
    case "grid":
      return dim === "w";
    case "chart":
      // Swift Charts (any v2 option, see `usesSwiftCharts` in RenderData.swift) fill both axes.
      return (
        c.chartType === "point" ||
        c.showXAxis != null ||
        c.showYAxis != null ||
        c.showLegend != null ||
        c.yMin != null ||
        c.yMax != null ||
        c.interpolation != null ||
        c.innerRadius != null ||
        (c.chartData || []).some(function (p: ElNode) {
          return p.series != null;
        })
      );
    case "vstack":
      return kids("vertical");
    case "hstack":
      return (dim === "w" && parentAxis === "vertical") || kids("horizontal");
    default:
      return false;
  }
}

export function renderStack(d: ElNode, dir: "column" | "row", parentAxis?: ParentAxis): HTMLElement {
  var e = document.createElement("div");
  e.style.display = "flex";
  e.style.flexDirection = dir;
  e.style.gap = (d.spacing || 0) + "px";
  // VStack: default stretch (text fills the width). Explicit alignment only when set.
  if (dir === "column") {
    e.style.alignItems =
      d.alignment === "trailing"
        ? "flex-end"
        : d.alignment === "leading"
          ? "flex-start"
          : d.alignment === "center"
            ? "center"
            : "stretch";
  } else {
    // SwiftUI HStack centers by default; flexible-height children stretch below.
    e.style.alignItems =
      d.alignment === "top" ? "flex-start" : d.alignment === "bottom" ? "flex-end" : "center";
  }
  var justify: Record<string, string> = {
    start: "flex-start",
    center: "center",
    end: "flex-end",
    spaceBetween: "space-between",
    spaceAround: "space-around",
    spaceEvenly: "space-evenly",
  };
  if (d.justify && justify[d.justify]) e.style.justifyContent = justify[d.justify];
  applyStyle(e, d);
  var axis: ParentAxis = dir === "row" ? "horizontal" : "vertical";
  // Spacers only take what flexible views leave (WidgetKit: a progress bar beats a spacer).
  var flexKids = (d.children || []).some(function (c: ElNode) {
    return c.type !== "spacer" && greedy(c, "w", axis);
  });
  // A row in a column spans it (maxWidth:.infinity) and centers content that doesn't fill it.
  if (dir === "row" && parentAxis === "vertical" && !d.justify && !greedy(d, "w", undefined))
    e.style.justifyContent = "center";
  (d.children || []).forEach(function (c: ElNode) {
    var k = renderEl(c, axis) as HTMLElement;
    if (dir === "row") {
      if (c.type === "spacer") {
        if (flexKids) k.style.flex = "0 0 auto";
      } else if (!(c.flex > 0) && greedy(c, "w", axis)) k.style.flex = "1 1 0";
      if (greedy(c, "h", axis)) k.style.alignSelf = "stretch";
    } else if (greedy(c, "w", axis)) {
      // Flexible width spans the column whatever its `alignment` (hstack rows, buttons).
      k.style.alignSelf = "stretch";
      // A row that only spans (content-sized in SwiftUI) sits where the column aligns it.
      var colAlign = (d.alignment || "").toLowerCase();
      // Unset = leading, as RenderLayout.swift (`renderVStack`) and Glance Column.
      if (c.type === "hstack" && k.style.justifyContent === "center" && colAlign !== "center")
        k.style.justifyContent = colAlign === "trailing" ? "flex-end" : "flex-start";
    }
    e.appendChild(k);
  });
  return e;
}

export function renderZStack(d: ElNode): HTMLElement {
  var e = document.createElement("div");
  e.style.display = "grid";
  var a = (d.alignment || "center").toLowerCase();
  var place: Record<string, string> = {
    center: "center",
    top: "start center",
    bottom: "end center",
    leading: "center start",
    trailing: "center end",
    topleading: "start start",
    toptrailing: "start end",
    bottomleading: "end start",
    bottomtrailing: "end end",
  };
  e.style.placeItems = place[a] || "center";
  applyStyle(e, d);
  (d.children || []).forEach(function (c: ElNode) {
    var w = document.createElement("div");
    w.style.gridArea = "1 / 1";
    // Stacks / bare rectangle underlays fill the cell so spacers can push corners.
    var fill =
      c.type === "vstack" ||
      c.type === "hstack" ||
      c.type === "container" ||
      c.type === "grid" ||
      c.type === "zstack" ||
      (c.type === "shape" &&
        (c.shapeType === "rectangle" || !c.shapeType) &&
        !c.size &&
        !(c.frame && c.frame.width && c.frame.height));
    if (fill) {
      w.style.width = "100%";
      w.style.height = "100%";
      w.style.display = "flex";
      w.style.flexDirection = "column";
    }
    var child = renderEl(c);
    if (fill) {
      child.style.flex = "1";
      child.style.width = "100%";
      child.style.height = "100%";
      child.style.minHeight = "0";
    }
    w.appendChild(child);
    e.appendChild(w);
  });
  return e;
}

export function renderGrid(d: ElNode): HTMLElement {
  var e = document.createElement("div");
  e.style.display = "grid";
  e.style.gridTemplateColumns = "repeat(" + (d.columns || 2) + ", 1fr)";
  e.style.columnGap = (d.spacing || 4) + "px";
  e.style.rowGap = (d.rowSpacing || d.spacing || 4) + "px";
  // LazyVGrid centers each cell's content; flexible children fill the cell.
  e.style.placeItems = "center";
  applyStyle(e, d);
  (d.children || []).forEach(function (c: ElNode) {
    var k = renderEl(c) as HTMLElement;
    // `flex` here is SwiftUI's maxWidth:.infinity frame — it centers, never resizes the view.
    if (greedy(Object.assign({}, c, { flex: 0 }), "w", undefined)) k.style.justifySelf = "stretch";
    if (greedy(c, "h", undefined)) k.style.alignSelf = "stretch";
    e.appendChild(k);
  });
  return e;
}

export function renderContainer(d: ElNode): HTMLElement {
  var e = document.createElement("div");
  e.style.display = "flex";
  var a = d.contentAlignment || "center";
  var vert: Record<string, string> = {
    top: "flex-start",
    bottom: "flex-end",
    topLeading: "flex-start",
    topTrailing: "flex-start",
    bottomLeading: "flex-end",
    bottomTrailing: "flex-end",
  };
  var horiz: Record<string, string> = {
    leading: "flex-start",
    trailing: "flex-end",
    topLeading: "flex-start",
    topTrailing: "flex-end",
    bottomLeading: "flex-start",
    bottomTrailing: "flex-end",
  };
  e.style.alignItems = vert[a] || "center";
  e.style.justifyContent = horiz[a] || "center";
  applyStyle(e, d);
  (d.children || []).forEach(function (ch: ElNode) {
    e.appendChild(renderEl(ch));
  });
  return e;
}
