import { applyStyle, resolveColor } from "../style";
import { emitAction } from "../ctx";
import { renderEl } from "./element";
import type { ElNode } from "../types";

export function renderButton(d: ElNode): HTMLElement {
  var e = document.createElement("div");
  e.textContent = d.label || "";
  var align = (d.textAlignment || d.alignment || "center").toLowerCase();
  if (align === "middle") align = "center";
  if (align === "end" || align === "trailing") align = "right";
  if (align === "start" || align === "leading") align = "left";
  // Intrinsic height + centered label — avoid stretch in default hstack (tall empty pill, label stuck to top).
  var padCss = "6px 12px";
  if (d.padding != null) {
    if (typeof d.padding === "number") padCss = d.padding + "px";
    else if (typeof d.padding === "object") {
      var pt =
        d.padding.top != null
          ? d.padding.top
          : d.padding.vertical != null
            ? d.padding.vertical
            : 6;
      var pb =
        d.padding.bottom != null
          ? d.padding.bottom
          : d.padding.vertical != null
            ? d.padding.vertical
            : 6;
      var pl =
        d.padding.leading != null
          ? d.padding.leading
          : d.padding.left != null
            ? d.padding.left
            : d.padding.horizontal != null
              ? d.padding.horizontal
              : 12;
      var pr =
        d.padding.trailing != null
          ? d.padding.trailing
          : d.padding.right != null
            ? d.padding.right
            : d.padding.horizontal != null
              ? d.padding.horizontal
              : 12;
      padCss = pt + "px " + pr + "px " + pb + "px " + pl + "px";
    }
  }
  e.style.cssText =
    "display:inline-flex;align-items:center;justify-content:" +
    (align === "left"
      ? "flex-start"
      : align === "right"
        ? "flex-end"
        : "center") +
    ";align-self:center;flex-shrink:0;padding:" +
    padCss +
    ";text-align:" +
    align +
    ";cursor:pointer;font-weight:500;transition:filter .1s;box-sizing:border-box";
  e.style.fontSize = (d.fontSize || 14) + "px";
  e.style.background = d.backgroundColor
    ? resolveColor(d.backgroundColor)
    : "#2196F3";
  e.style.color = d.color ? resolveColor(d.color) : "#fff";
  e.style.borderRadius = (d.cornerRadius || 8) + "px";
  e.onmousedown = function () {
    e.style.filter = "brightness(0.85)";
  };
  e.onmouseup = e.onmouseleave = function () {
    e.style.filter = "";
  };
  e.onclick = function () {
    if (d.action) emitAction(d.action);
    else if (d.url) window.open(d.url, "_blank");
  };
  // applyStyle would clobber the padding we just set — skip padding there for buttons.
  var padSave = d.padding;
  d.padding = undefined;
  applyStyle(e, d);
  d.padding = padSave;
  return e;
}

export function renderToggle(d: ElNode): HTMLElement {
  var e = document.createElement("div");
  e.style.cssText = "display:flex;align-items:center;gap:6px";
  var tint = d.tint ? resolveColor(d.tint) : "#4CAF50";
  var c = document.createElement("div");
  c.style.cssText =
    "width:18px;height:18px;border-radius:9px;display:flex;align-items:center;justify-content:center;flex-shrink:0";
  c.style.border = "2px solid " + (d.isOn ? tint : "#999");
  c.style.background = d.isOn ? tint : "transparent";
  if (d.isOn) {
    var m = document.createElement("span");
    m.textContent = "\u2713";
    m.style.cssText = "color:#fff;font-size:12px";
    c.appendChild(m);
  }
  e.appendChild(c);
  if (d.label) {
    var l = document.createElement("span");
    l.textContent = d.label;
    l.style.fontSize = "14px";
    l.style.color = d.color ? resolveColor(d.color) : "inherit";
    e.appendChild(l);
  }
  if (d.action) {
    e.style.cursor = "pointer";
    e.onclick = function () {
      emitAction(d.action, String(!d.isOn));
    };
  }
  applyStyle(e, d);
  return e;
}

export function renderLink(d: ElNode): HTMLElement {
  var e = document.createElement("div");
  e.style.cursor = "pointer";
  e.onclick = function () {
    if (d.action) emitAction(d.action);
    else if (d.url) window.open(d.url, "_blank");
  };
  applyStyle(e, d);
  (d.children || []).forEach(function (c: ElNode) {
    e.appendChild(renderEl(c));
  });
  return e;
}
