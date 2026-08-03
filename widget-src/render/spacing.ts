import { applyStyle, resolveColor } from "../style";
import type { ElNode, ParentAxis } from "../types";

export function renderDivider(d: ElNode, parentAxis?: ParentAxis): HTMLElement {
  var e = document.createElement("hr");
  e.style.border = "none";
  var t = d.thickness || 1;
  e.style.background = d.color ? resolveColor(d.color) : "#e0e0e0";
  e.style.flexShrink = "0";
  if (parentAxis === "horizontal") {
    e.style.width = t + "px";
    e.style.height = "auto";
    e.style.alignSelf = "stretch";
  } else {
    e.style.width = "100%";
    e.style.height = t + "px";
    // Match Android section breath around horizontal rules.
    e.style.marginTop = "4px";
    e.style.marginBottom = "4px";
  }
  applyStyle(e, d);
  return e;
}

export function renderSpacer(d: ElNode): HTMLElement {
  var e = document.createElement("div");
  e.style.flex = "1";
  e.style.minHeight = (d.minLength || 0) + "px";
  return e;
}
