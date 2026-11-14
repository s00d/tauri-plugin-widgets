import {
  applyStyle,
  fw,
  resolveColor,
  TEXT_STYLE_BOLD,
  TEXT_STYLE_SIZES,
} from "../style";
import { addTimer } from "../ctx";
import { systemGlyph } from "./media";
import type { ElNode, ParentAxis } from "../types";

export function formatRelativeDate(date: Date): string {
  var diffMs = date.getTime() - Date.now();
  var abs = Math.abs(diffMs);
  var sec = Math.round(abs / 1000),
    min = Math.round(sec / 60),
    hr = Math.round(min / 60);
  var day = Math.round(hr / 24),
    mon = Math.round(day / 30),
    yr = Math.round(day / 365);
  var unit: Intl.RelativeTimeFormatUnit = "second",
    val = sec;
  if (yr >= 1) {
    unit = "year";
    val = yr;
  } else if (mon >= 1) {
    unit = "month";
    val = mon;
  } else if (day >= 1) {
    unit = "day";
    val = day;
  } else if (hr >= 1) {
    unit = "hour";
    val = hr;
  } else if (min >= 1) {
    unit = "minute";
    val = min;
  }
  var signed = diffMs < 0 ? -val : val;
  try {
    return new Intl.RelativeTimeFormat(undefined, { numeric: "auto" }).format(
      signed,
      unit
    );
  } catch (_) {
    return (
      (signed < 0 ? "" : "+") +
      signed +
      " " +
      unit +
      (Math.abs(signed) === 1 ? "" : "s")
    );
  }
}

export function renderText(d: ElNode, parentAxis?: ParentAxis): HTMLElement {
  var e = document.createElement("span");
  e.textContent = d.content || "";
  var ts = d.textStyle && TEXT_STYLE_SIZES[d.textStyle];
  e.style.fontSize = (ts || d.fontSize || 14) + "px";
  e.style.fontWeight = String(
    d.textStyle && TEXT_STYLE_BOLD[d.textStyle] ? 700 : fw(d.fontWeight));
  e.style.color = d.color ? resolveColor(d.color) : "inherit";
  var hasFixedBox = !!(d.frame && d.frame.width && d.frame.height);
  var align =
    d.alignment ||
    (hasFixedBox || d.clipShape === "circle" ? "center" : "leading");
  e.style.textAlign =
    align === "center"
      ? "center"
      : align === "trailing"
        ? "right"
        : "left";
  e.style.display = d.lineLimit ? "-webkit-box" : hasFixedBox ? "flex" : "block";
  if (hasFixedBox) {
    e.style.alignItems = "center";
    e.style.justifyContent =
      align === "trailing" ? "flex-end" : align === "center" ? "center" : "flex-start";
  }
  e.style.boxSizing = "border-box";
  e.style.flexShrink = "0";
  // In hstack hug content width so siblings (divider / trailing text) stay visible.
  // In vstack (or unknown axis) stretch full width so textAlign leading/center/trailing works.
  // Fixed frame badges keep explicit width from applyStyle — don't force 100%.
  if (hasFixedBox) {
    // width/height come from applyStyle(frame)
  } else if (parentAxis === "horizontal") {
    e.style.width = "auto";
  } else {
    e.style.width = "100%";
    e.style.alignSelf = "stretch";
  }
  if (d.fontDesign === "monospaced") e.style.fontFamily = "monospace";
  else if (d.fontDesign === "serif") e.style.fontFamily = "serif";
  else if (d.fontDesign === "rounded")
    e.style.fontFamily =
      'ui-rounded, "SF Pro Rounded", system-ui, sans-serif';
  if (d.lineLimit) {
    (e.style as CSSStyleDeclaration & { webkitLineClamp?: string; webkitBoxOrient?: string }).webkitLineClamp = String(d.lineLimit);
    (e.style as CSSStyleDeclaration & { webkitBoxOrient?: string }).webkitBoxOrient = "vertical";
    e.style.overflow = "hidden";
  }
  if (d.lineHeight > 0) e.style.lineHeight = d.lineHeight + "px";
  if (d.letterSpacing) e.style.letterSpacing = d.letterSpacing + "px";
  var deco = (d.underline ? " underline" : "") + (d.strikethrough ? " line-through" : "");
  if (deco) e.style.textDecorationLine = deco.trim();
  if (d.italic) e.style.fontStyle = "italic";
  if (d.monospacedDigit) e.style.fontVariantNumeric = "tabular-nums";
  // head / middle cut only on one line (more lines: tail, like the line clamp).
  if (d.lineLimit === 1 && (d.truncationMode === "head" || d.truncationMode === "middle")) {
    e.style.display = "block";
    e.style.whiteSpace = "nowrap";
    e.style.overflow = "hidden";
    var text = String(d.content || "");
    if (d.truncationMode === "head") {
      // RTL box puts the ellipsis at the start and clips the beginning; the inner
      // isolate keeps the text itself in its own (LTR) order.
      e.style.direction = "rtl";
      e.style.textOverflow = "ellipsis";
      e.textContent = "";
      var inner = document.createElement("bdi");
      inner.textContent = text;
      e.appendChild(inner);
    } else {
      // First half shrinks with a tail ellipsis, second half keeps its width.
      e.style.display = "flex";
      e.style.justifyContent =
        e.style.textAlign === "center" ? "center" : e.style.textAlign === "right" ? "flex-end" : "flex-start";
      e.textContent = "";
      var cut = Math.ceil(text.length / 2);
      var a = document.createElement("span");
      a.textContent = text.slice(0, cut);
      a.style.cssText = "overflow:hidden;text-overflow:ellipsis;white-space:pre;min-width:0;flex:0 1 auto";
      var b = document.createElement("span");
      b.textContent = text.slice(cut);
      b.style.cssText = "white-space:pre;flex:0 0 auto";
      e.appendChild(a);
      e.appendChild(b);
    }
  }
  applyStyle(e, d);
  if (d.minimumScaleFactor > 0 && d.minimumScaleFactor < 1) {
    // Let a hugging text be squeezed, then shrink the font until it fits.
    e.style.flexShrink = "1";
    e.style.minWidth = "0";
    shrinkToFit(e, parseFloat(e.style.fontSize), d.minimumScaleFactor);
  }
  return e;
}

/** Smallest font in [base·min, base] (0.5px steps) at which `e` stops overflowing,
 *  re-checked on every resize. */
function shrinkToFit(e: HTMLElement, base: number, min: number): void {
  var ro = new ResizeObserver(function () {
    if (!e.isConnected) {
      ro.disconnect();
      return;
    }
    var size = base;
    e.style.fontSize = size + "px";
    while (
      size - 0.5 >= base * min &&
      (e.scrollWidth > e.clientWidth + 1 || e.scrollHeight > e.clientHeight + 1)
    ) {
      size -= 0.5;
      e.style.fontSize = size + "px";
    }
  });
  ro.observe(e);
}

export function renderDate(d: ElNode): HTMLElement {
  var e = document.createElement("span");
  var date = new Date(d.date);
  var style = (d.dateStyle || "").toLowerCase();
  if (style === "time")
    e.textContent = date.toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
    });
  else if (style === "date")
    e.textContent = date.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  else if (style === "relative" || style === "offset")
    e.textContent = formatRelativeDate(date);
  else if (style === "timer") {
    var diff = Math.abs(date.getTime() - Date.now());
    var h = Math.floor(diff / 3600000),
      m = Math.floor((diff % 3600000) / 60000),
      s = Math.floor((diff % 60000) / 1000);
    e.textContent =
      String(h).padStart(2, "0") +
      ":" +
      String(m).padStart(2, "0") +
      ":" +
      String(s).padStart(2, "0");
  } else e.textContent = date.toLocaleString();
  e.style.fontSize = (d.fontSize || 14) + "px";
  e.style.color = d.color ? resolveColor(d.color) : "inherit";
  applyStyle(e, d);
  return e;
}

export function renderTimer(d: ElNode): HTMLElement {
  var e = document.createElement("span");
  e.style.fontSize = (d.fontSize || 14) + "px";
  e.style.fontWeight = String(fw(d.fontWeight));
  e.style.color = d.color ? resolveColor(d.color) : "inherit";
  e.style.fontVariantNumeric = "tabular-nums";
  var target = new Date(d.targetDate).getTime();
  var start = d.startDate ? new Date(d.startDate).getTime() : NaN;
  function tick() {
    var now = Date.now(),
      diff = Math.abs(target - now);
    if (d.counting === "up" && !isNaN(start)) {
      // Range timer: counts from startDate and stops at targetDate.
      diff = Math.max(0, Math.min(now, target) - start);
    } else if (d.counting !== "up" && now >= target) {
      if (d.completedText != null) {
        e.textContent = d.completedText;
        return;
      }
      diff = 0;
    }
    var h = Math.floor(diff / 3600000),
      m = Math.floor((diff % 3600000) / 60000),
      s = Math.floor((diff % 60000) / 1000);
    // showHours:false drops a zero hour; otherwise hours always show (legacy HH:MM:SS).
    e.textContent =
      (d.showHours === false && h === 0 ? "" : String(h).padStart(2, "0") + ":") +
      String(m).padStart(2, "0") +
      ":" +
      String(s).padStart(2, "0");
  }
  tick();
  addTimer(setInterval(tick, 1000));
  applyStyle(e, d);
  return e;
}

export function renderLabel(d: ElNode): HTMLElement {
  var e = document.createElement("div");
  e.style.cssText =
    "display:flex;align-items:center;gap:" + (d.spacing || 4) + "px";
  var ic = document.createElement("span");
  // Same glyph map as `image` (Apple draws the SF Symbol; `●` only for unknown names).
  ic.textContent = systemGlyph(d.systemName);
  ic.style.fontSize = (d.fontSize || 14) * 1.1 + "px";
  ic.style.color = d.iconColor
    ? resolveColor(d.iconColor)
    : d.color
      ? resolveColor(d.color)
      : "inherit";
  e.appendChild(ic);
  var t = document.createElement("span");
  t.textContent = d.text || "";
  t.style.fontSize = (d.fontSize || 14) + "px";
  t.style.fontWeight = String(fw(d.fontWeight));
  t.style.color = d.color ? resolveColor(d.color) : "inherit";
  e.appendChild(t);
  applyStyle(e, d);
  return e;
}
