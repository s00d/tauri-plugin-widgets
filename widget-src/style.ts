import { THEME } from "./ctx";
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
    THEME ||
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
  if (d.background) {
    if (typeof d.background === "string") el.style.background = hex(d.background);
    else if (d.background.colors) {
      const g = d.background;
      const cols = g.colors.map(hex).join(", ");
      const dirs: Record<string, string> = {
        topToBottom: "to bottom",
        bottomToTop: "to top",
        leadingToTrailing: "to right",
        trailingToLeading: "to left",
        topLeadingToBottomTrailing: "to bottom right",
        topTrailingToBottomLeading: "to bottom left",
      };
      if (g.gradientType === "radial")
        el.style.background = "radial-gradient(circle, " + cols + ")";
      else if (g.gradientType === "angular")
        el.style.background = "conic-gradient(" + cols + ")";
      else
        el.style.background =
          "linear-gradient(" + (dirs[g.direction] || "to bottom") + ", " + cols + ")";
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
  }
  if (d.clipShape) {
    el.style.overflow = "hidden";
    if (d.clipShape === "circle") el.style.borderRadius = "50%";
    else if (d.clipShape === "capsule") el.style.borderRadius = "9999px";
    else if (d.clipShape === "rectangle" && d.cornerRadius)
      el.style.borderRadius = d.cornerRadius + "px";
  }
  if (d.flex && d.flex > 0) {
    el.style.flex = String(d.flex);
    el.style.minWidth = "0";
  }
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
