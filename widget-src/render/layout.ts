import { applyStyle } from "../style";
import { renderEl } from "./element";
import type { ElNode, ParentAxis } from "../types";

export function renderStack(d: ElNode, dir: "column" | "row"): HTMLElement {
  var e = document.createElement("div");
  e.style.display = "flex";
  e.style.flexDirection = dir;
  e.style.gap = (d.spacing || 0) + "px";
  // Default stretch (SwiftUI/Glance-like). Explicit alignment only when set.
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
    e.style.alignItems =
      d.alignment === "top"
        ? "flex-start"
        : d.alignment === "bottom"
          ? "flex-end"
          : d.alignment === "center"
            ? "center"
            : "stretch";
  }
  applyStyle(e, d);
  var axis: ParentAxis = dir === "row" ? "horizontal" : "vertical";
  (d.children || []).forEach(function (c: ElNode) {
    e.appendChild(renderEl(c, axis));
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
    w.appendChild(renderEl(c));
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
  applyStyle(e, d);
  (d.children || []).forEach(function (c: ElNode) {
    e.appendChild(renderEl(c));
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
