import {
  applyStyle,
  fw,
  resolveColor,
  TEXT_STYLE_BOLD,
  TEXT_STYLE_SIZES,
} from "../style";
import { timerIds } from "../ctx";
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
  e.style.textAlign =
    d.alignment === "center"
      ? "center"
      : d.alignment === "trailing"
        ? "right"
        : "left";
  e.style.display = d.lineLimit ? "-webkit-box" : "block";
  e.style.boxSizing = "border-box";
  e.style.flexShrink = "0";
  // In hstack hug content width so siblings (divider / trailing text) stay visible.
  // In vstack (or unknown axis) stretch full width so textAlign leading/center/trailing works.
  if (parentAxis === "horizontal") {
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
  applyStyle(e, d);
  return e;
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
  function tick() {
    var target = new Date(d.targetDate).getTime(),
      now = Date.now(),
      diff = Math.abs(target - now);
    var h = Math.floor(diff / 3600000),
      m = Math.floor((diff % 3600000) / 60000),
      s = Math.floor((diff % 60000) / 1000);
    e.textContent =
      String(h).padStart(2, "0") +
      ":" +
      String(m).padStart(2, "0") +
      ":" +
      String(s).padStart(2, "0");
  }
  tick();
  timerIds.push(setInterval(tick, 1000));
  applyStyle(e, d);
  return e;
}

export function renderLabel(d: ElNode): HTMLElement {
  var e = document.createElement("div");
  e.style.cssText =
    "display:flex;align-items:center;gap:" + (d.spacing || 4) + "px";
  var ic = document.createElement("span");
  ic.textContent = "\u25CF";
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
