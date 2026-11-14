import { getTheme } from "./ctx";
import type { ColorValue, ElNode } from "./types";

export function expandHex(h: string): string {
  if (h.length === 3) return h[0] + h[0] + h[1] + h[1] + h[2] + h[2];
  if (h.length === 4)
    return h[0] + h[0] + h[1] + h[1] + h[2] + h[2] + h[3] + h[3];
  return h;
}

export function isDark(): boolean {
  // Prefer explicit theme: URL ?theme= → data-theme → OS preference.
  const t =
    getTheme() ||
    document.documentElement.getAttribute("data-theme") ||
    document.documentElement.dataset?.theme ||
    "";
  if (t === "light") return false;
  if (t === "dark") return true;
  return !!(
    window.matchMedia &&
    window.matchMedia("(prefers-color-scheme:dark)").matches
  );
}

const SEMANTIC_COLORS: Record<string, [string, string]> = {
  label: ["#000000", "#FFFFFF"],
  secondaryLabel: ["#3C3C43", "#EBEBF5"],
  systemBackground: ["#FFFFFF", "#000000"],
  secondarySystemBackground: ["#F2F2F7", "#1C1C1E"],
  accent: ["#007AFF", "#0A84FF"],
  separator: ["#C6C6C8", "#545458"],
};

export function hex(c: string | null | undefined): string {
  if (!c) return "inherit";
  if (SEMANTIC_COLORS[c]) {
    return isDark() ? SEMANTIC_COLORS[c][1] : SEMANTIC_COLORS[c][0];
  }
  const h = expandHex(c.replace("#", ""));
  if (h.length === 8)
    return (
      "rgba(" +
      parseInt(h.slice(0, 2), 16) +
      "," +
      parseInt(h.slice(2, 4), 16) +
      "," +
      parseInt(h.slice(4, 6), 16) +
      "," +
      (parseInt(h.slice(6, 8), 16) / 255).toFixed(2) +
      ")"
    );
  return "#" + h;
}

export function resolveColor(cv: ColorValue | null | undefined): string {
  if (!cv) return "inherit";
  if (typeof cv === "string") return hex(cv);
  if (cv.light && cv.dark) return isDark() ? hex(cv.dark) : hex(cv.light);
  return "inherit";
}

export function tintTrack(tint: ColorValue | null | undefined): string {
  const resolved = tint ? resolveColor(tint) : null;
  if (!resolved || resolved === "inherit") return "rgba(120,120,128,0.2)";
  const h = expandHex(resolved.replace("#", ""));
  if (h.length >= 6) {
    const r = parseInt(h.slice(0, 2), 16);
    const g = parseInt(h.slice(2, 4), 16);
    const b = parseInt(h.slice(4, 6), 16);
    return "rgba(" + r + "," + g + "," + b + ",0.2)";
  }
  return "rgba(120,120,128,0.2)";
}

export const TEXT_STYLE_SIZES: Record<string, number> = {
  largeTitle: 34,
  title: 28,
  title2: 22,
  title3: 20,
  headline: 17,
  subheadline: 15,
  body: 17,
  callout: 16,
  footnote: 13,
  caption: 12,
  caption2: 11,
};
export const TEXT_STYLE_BOLD: Record<string, boolean> = { headline: true };

/** `padding` / `margin` value (number or per-edge object) → CSS box shorthand. */
function edges(v: number | Record<string, number | undefined>): string {
  if (typeof v === "number") return v + "px";
  return (v.top || 0) + "px " + (v.trailing || 0) + "px " + (v.bottom || 0) + "px " + (v.leading || 0) + "px";
}

type Gradient = {
  gradientType?: string;
  colors?: string[];
  direction?: string;
  stops?: { color: string; location: number }[];
  startPoint?: { x: number; y: number };
  endPoint?: { x: number; y: number };
};

/** CSS gradient for a `GradientConfig`. Unit points map onto a square box: the CSS angle
 *  is the start→end direction and stops are re-projected onto the CSS gradient line, so a
 *  non-square element is approximate (SwiftUI stretches the unit square). */
export function gradientCss(g: Gradient): string {
  const stops = g.stops && g.stops.length ? g.stops : null;
  const list = function (pos: (loc: number) => number): string {
    if (!stops) return (g.colors || []).map(hex).join(", ");
    return stops
      .map(function (s) {
        return hex(s.color) + " " + +(pos(s.location) * 100).toFixed(2) + "%";
      })
      .join(", ");
  };
  if (g.gradientType === "radial") {
    const c = g.startPoint;
    return "radial-gradient(circle" + (c ? " at " + c.x * 100 + "% " + c.y * 100 + "%" : "") + ", " + list((l) => l) + ")";
  }
  if (g.gradientType === "angular") return "conic-gradient(" + list((l) => l) + ")";
  const sp = g.startPoint,
    ep = g.endPoint;
  if (sp && ep && (sp.x !== ep.x || sp.y !== ep.y)) {
    const dx = ep.x - sp.x,
      dy = ep.y - sp.y;
    const a = Math.atan2(dx, -dy); // CSS: 0deg = up, clockwise
    const ux = Math.sin(a),
      uy = -Math.cos(a);
    const len = Math.abs(ux) + Math.abs(uy); // gradient line length in a unit square
    const t = function (p: { x: number; y: number }) {
      return ((p.x - 0.5) * ux + (p.y - 0.5) * uy) / len + 0.5;
    };
    const ts = t(sp),
      te = t(ep);
    return (
      "linear-gradient(" +
      +((a * 180) / Math.PI).toFixed(2) +
      "deg, " +
      list(function (l) {
        return ts + l * (te - ts);
      }) +
      ")"
    );
  }
  const dirs: Record<string, string> = {
    topToBottom: "to bottom",
    bottomToTop: "to top",
    leadingToTrailing: "to right",
    trailingToLeading: "to left",
    topLeadingToBottomTrailing: "to bottom right",
    topTrailingToBottomLeading: "to bottom left",
  };
  return "linear-gradient(" + (dirs[g.direction || ""] || "to bottom") + ", " + list((l) => l) + ")";
}

export function applyStyle(el: HTMLElement | SVGElement, d: ElNode): void {
  if (d.padding != null) {
    if (typeof d.padding === "number") el.style.padding = d.padding + "px";
    else {
      el.style.paddingTop = (d.padding.top || 0) + "px";
      el.style.paddingBottom = (d.padding.bottom || 0) + "px";
      el.style.paddingLeft = (d.padding.leading || 0) + "px";
      el.style.paddingRight = (d.padding.trailing || 0) + "px";
    }
  }
  if (d.margin != null) el.style.margin = edges(d.margin);
  if (d.background) {
    if (typeof d.background === "string") el.style.background = hex(d.background);
    else if (d.background.colors || d.background.stops) {
      el.style.background = gradientCss(d.background);
    } else if (d.background.light && d.background.dark) {
      el.style.background = isDark()
        ? hex(d.background.dark)
        : hex(d.background.light);
    }
  }
  if (d.cornerRadius) {
    el.style.borderRadius = d.cornerRadius + "px";
    el.style.overflow = "hidden";
  }
  if (d.opacity != null) el.style.opacity = String(d.opacity);
  if (d.border)
    el.style.border =
      (d.border.width || 1) + "px solid " + hex(d.border.color);
  if (d.shadow) {
    const s = d.shadow;
    el.style.boxShadow =
      (s.x || 0) +
      "px " +
      (s.y || 2) +
      "px " +
      (s.radius || 4) +
      "px " +
      (s.color ? hex(s.color) : "rgba(0,0,0,.3)");
  }
  if (d.frame) {
    if (d.frame.width) {
      el.style.width = d.frame.width + "px";
      el.style.minWidth = d.frame.width + "px";
      el.style.flexShrink = "0";
    }
    if (d.frame.height) {
      el.style.height = d.frame.height + "px";
      el.style.minHeight = d.frame.height + "px";
    }
    if (d.frame.maxWidth === "infinity") el.style.maxWidth = "100%";
    else if (d.frame.maxWidth) el.style.maxWidth = d.frame.maxWidth + "px";
    if (d.frame.minWidth) el.style.minWidth = d.frame.minWidth + "px";
    if (d.frame.minHeight) el.style.minHeight = d.frame.minHeight + "px";
  }
  if (d.clipShape) {
    el.style.overflow = "hidden";
    if (d.clipShape === "circle" || d.clipShape === "ellipse") el.style.borderRadius = "50%";
    else if (d.clipShape === "capsule") el.style.borderRadius = "9999px";
    else if ((d.clipShape === "rectangle" || d.clipShape === "roundedRectangle") && d.cornerRadius)
      el.style.borderRadius = d.cornerRadius + "px";
  }
  if (d.flex && d.flex > 0) {
    el.style.flex = String(d.flex);
    if (!(d.frame && d.frame.minWidth)) el.style.minWidth = "0";
  }
  if (d.aspectRatio > 0) el.style.aspectRatio = String(d.aspectRatio);
  // Higher priority keeps its size in a crowded stack; lower gives way first.
  if (typeof d.layoutPriority === "number" && d.layoutPriority !== 0) {
    if (d.layoutPriority > 0) el.style.flexShrink = "0";
    else {
      el.style.flexShrink = String(1 - d.layoutPriority);
      if (!(d.frame && d.frame.minWidth)) el.style.minWidth = "0";
    }
  }
  var tf = "";
  if (d.offsetX || d.offsetY) tf += "translate(" + (d.offsetX || 0) + "px," + (d.offsetY || 0) + "px) ";
  if (d.rotation) tf += "rotate(" + d.rotation + "deg) ";
  if (d.scale != null && d.scale !== 1) tf += "scale(" + d.scale + ")";
  if (tf) el.style.transform = tf.trim();
  // Desktop has no tinted Home Screen; keep the marker so hosts can style it.
  if (d.accentable === true) el.setAttribute("data-accentable", "");
  if (typeof d.accessibilityLabel === "string" && d.accessibilityLabel) {
    el.setAttribute("aria-label", d.accessibilityLabel);
    el.setAttribute("role", el.getAttribute("role") || "img");
  }
  // Desktop has no locked-screen redaction; keep the marker so hosts can style it.
  if (d.privacySensitive === true) el.setAttribute("data-privacy-sensitive", "");
}

export function fw(w: string | null | undefined): number {
  return (
    (
      {
        ultralight: 100,
        thin: 200,
        light: 300,
        regular: 400,
        medium: 500,
        semibold: 600,
        bold: 700,
        heavy: 800,
        black: 900,
      } as Record<string, number>
    )[w || ""] || 400
  );
}

export function svg(
  tag: string,
  a: Record<string, string | number | null | undefined>,
): SVGElement {
  const e = document.createElementNS("http://www.w3.org/2000/svg", tag);
  for (const k in a) if (a[k] != null) e.setAttribute(k, String(a[k]));
  return e;
}
