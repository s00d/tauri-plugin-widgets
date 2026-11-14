import { applyStyle, resolveColor, tintTrack } from "../style";
import { systemGlyph } from "./media";
import { boundTap } from "../ctx";
import { renderEl } from "./element";
import type { ElNode } from "../types";

export function renderButton(d: ElNode): HTMLElement {
  var tap = boundTap(d);
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
  var accent = d.backgroundColor ? resolveColor(d.backgroundColor) : "#2196F3";
  // No variant = legacy filled look (same as `prominent`).
  if (d.variant === "bordered") {
    e.style.background = tintTrack(d.backgroundColor || "#2196F3");
    e.style.color = d.color ? resolveColor(d.color) : accent;
  } else if (d.variant === "plain" || d.variant === "outline") {
    e.style.background = "transparent";
    e.style.color = d.color ? resolveColor(d.color) : accent;
    if (d.variant === "outline") e.style.border = "1px solid currentColor";
  } else {
    e.style.background = accent;
    e.style.color = d.color ? resolveColor(d.color) : "#fff";
  }
  e.style.borderRadius = (d.cornerRadius || 8) + "px";
  if (d.systemName) {
    var ic = document.createElement("span");
    ic.textContent = systemGlyph(d.systemName);
    ic.style.cssText = "margin-right:4px;line-height:1";
    e.insertBefore(ic, e.firstChild);
  }
  if (d.disabled === true) {
    e.style.opacity = "0.4";
    e.style.cursor = "default";
    e.setAttribute("aria-disabled", "true");
  } else {
    e.onmousedown = function () {
      e.style.filter = "brightness(0.85)";
    };
    e.onmouseup = e.onmouseleave = function () {
      e.style.filter = "";
    };
    e.onclick = function () {
      if (d.action || d.set) tap();
      if (!d.action && d.url) openHttpUrl(d.url);
    };
  }
  // applyStyle would clobber the padding we just set — skip padding there for buttons.
  var padSave = d.padding;
  d.padding = undefined;
  applyStyle(e, d);
  d.padding = padSave;
  return e;
}

export function renderToggle(d: ElNode): HTMLElement {
  var tap = boundTap(d);
  var e = document.createElement("div");
  e.style.cssText = "display:flex;align-items:center;gap:6px";
  var tint = d.tint ? resolveColor(d.tint) : "#4CAF50";
  var off = d.uncheckedTint ? resolveColor(d.uncheckedTint) : null;
  var c = document.createElement("div");
  if (d.variant === "switch") {
    c.style.cssText =
      "width:32px;height:18px;border-radius:9px;position:relative;flex-shrink:0;box-sizing:border-box";
    c.style.background = d.isOn ? tint : off || "rgba(120,120,128,0.32)";
    var knob = document.createElement("span");
    knob.style.cssText =
      "position:absolute;top:2px;width:14px;height:14px;border-radius:7px;background:#fff;box-shadow:0 1px 2px rgba(0,0,0,.3)";
    knob.style.left = d.isOn ? "16px" : "2px";
    c.appendChild(knob);
  } else {
    // checkbox = rounded square; no variant = legacy round check.
    var box = d.variant === "checkbox";
    c.style.cssText =
      "width:18px;height:18px;border-radius:9px;display:flex;align-items:center;justify-content:center;flex-shrink:0";
    if (box) {
      c.style.width = c.style.height = "16px";
      c.style.borderRadius = "4px";
    }
    c.style.border = "2px solid " + (d.isOn ? tint : off || "#999");
    c.style.background = d.isOn ? tint : "transparent";
    if (d.isOn) {
      var m = document.createElement("span");
      m.textContent = "\u2713";
      m.style.cssText = "color:#fff;font-size:12px";
      c.appendChild(m);
    }
  }
  e.appendChild(c);
  if (d.label) {
    var l = document.createElement("span");
    l.textContent = d.label;
    l.style.fontSize = "14px";
    l.style.color = d.color ? resolveColor(d.color) : "inherit";
    e.appendChild(l);
  }
  if (d.action || d.set) {
    e.style.cursor = "pointer";
    e.onclick = function () {
      tap(String(!d.isOn));
    };
  }
  applyStyle(e, d);
  return e;
}

/** Open only http(s) URLs — block javascript:/data:/etc. from widget config. */
function openHttpUrl(url: string): void {
  var u = String(url).trim();
  var lower = u.toLowerCase();
  if (lower.indexOf("http://") !== 0 && lower.indexOf("https://") !== 0) return;
  window.open(u, "_blank");
}

export function renderLink(d: ElNode): HTMLElement {
  var tap = boundTap(d);
  var e = document.createElement("div");
  e.style.cursor = "pointer";
  e.onclick = function () {
    if (d.action || d.set) tap();
    if (!d.action && d.url) openHttpUrl(d.url);
  };
  applyStyle(e, d);
  (d.children || []).forEach(function (c: ElNode) {
    e.appendChild(renderEl(c));
  });
  return e;
}
