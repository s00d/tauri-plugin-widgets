import { applyStyle, resolveColor, svg, tintTrack } from "../style";
import { addTimer, boundEmit } from "../ctx";
import { renderEl } from "./child";
import type { ElNode } from "../types";

/** Fraction of [startAt, endAt] elapsed now (0…1), or null when not time-driven. */
function timeFraction(d: ElNode): number | null {
  if (d.startAt == null || d.endAt == null || !(d.endAt > d.startAt)) return null;
  return Math.max(0, Math.min(1, (Date.now() - d.startAt) / (d.endAt - d.startAt)));
}

/** Re-run `paint` every second while a time-driven bar is still moving (session timer). */
function tickWhileRunning(d: ElNode, paint: () => void): void {
  if (timeFraction(d) === null || Date.now() >= d.endAt) return;
  var id = setInterval(function () {
    paint();
    if (Date.now() >= d.endAt) clearInterval(id);
  }, 1000);
  addTimer(id);
}

var RING = "M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831";

export function renderProgress(d: ElNode): HTMLElement | SVGElement {
  var frac = function () {
    var t = timeFraction(d);
    return t !== null ? t : d.value / (d.total || 1);
  };
  var pct = function () {
    return (frac() * 100).toFixed(1);
  };
  var tint = d.tint ? resolveColor(d.tint) : "#4CAF50";
  var track = d.trackColor ? resolveColor(d.trackColor) : tintTrack(d.tint);
  if (d.barStyle === "circular") {
    var w = document.createElement("div");
    w.style.cssText = "width:40px;height:40px;position:relative";
    var s = svg("svg", { viewBox: "0 0 36 36" });
    s.style.cssText = "width:100%;height:100%";
    // thickness is in points; the 36-unit ring is drawn 40px wide.
    var ringW = d.thickness > 0 ? +((d.thickness * 36) / 40).toFixed(2) : 3;
    s.appendChild(
      svg("path", {
        d: RING,
        fill: "none",
        stroke: track,
        "stroke-width": ringW,
      })
    );
    var arc = svg("path", {
      d: RING,
      fill: "none",
      stroke: tint,
      "stroke-width": ringW,
      "stroke-dasharray": pct() + ", 100",
      "stroke-linecap": "round",
    });
    s.appendChild(arc);
    tickWhileRunning(d, function () {
      arc.setAttribute("stroke-dasharray", pct() + ", 100");
    });
    w.appendChild(s);
    applyStyle(w, d);
    return w;
  }
  var e = document.createElement("div");
  e.style.cssText =
    "width:100%;min-width:48px;align-self:stretch;box-sizing:border-box";
  if (d.label) {
    var l = document.createElement("div");
    l.textContent = d.label;
    l.style.cssText = "font-size:10px;margin-bottom:2px";
    l.style.color = d.color ? resolveColor(d.color) : tint;
    e.appendChild(l);
  }
  var th = d.thickness > 0 ? d.thickness : 6;
  var tr = document.createElement("div");
  tr.style.cssText =
    "height:6px;border-radius:3px;overflow:hidden;width:100%";
  tr.style.height = th + "px";
  tr.style.borderRadius = th / 2 + "px";
  tr.style.background = track;
  var f = document.createElement("div");
  f.style.cssText = "height:100%;border-radius:3px";
  f.style.borderRadius = th / 2 + "px";
  f.style.width = pct() + "%";
  f.style.background = tint;
  tickWhileRunning(d, function () {
    f.style.width = pct() + "%";
  });
  tr.appendChild(f);
  e.appendChild(tr);
  applyStyle(e, d);
  return e;
}

/** `minLabel` … `maxLabel` row (null when neither is set). */
function rangeLabels(d: ElNode, clr: string): HTMLElement | null {
  if (d.minLabel == null && d.maxLabel == null) return null;
  var row = document.createElement("div");
  row.style.cssText =
    "display:flex;justify-content:space-between;gap:6px;font-size:9px;opacity:0.7;margin-top:2px";
  row.style.color = clr;
  [d.minLabel, d.maxLabel].forEach(function (t) {
    var sp = document.createElement("span");
    sp.textContent = t == null ? "" : String(t);
    row.appendChild(sp);
  });
  return row;
}

export function renderGauge(d: ElNode): HTMLElement | SVGElement {
  var lo = d.min || 0,
    hi = d.max || 1;
  var pctNow = function () {
    var t = timeFraction(d);
    var v = t !== null ? lo + t * (hi - lo) : d.value || 0;
    return ((v - lo) / Math.max(hi - lo, 0.0001)) * 100;
  };
  var pct = pctNow();
  var tint = d.tint ? resolveColor(d.tint) : "#4CAF50";
  var track = tintTrack(d.tint);
  // Like Swift (`color ?? .primary`): labels inherit the text color unless `color` is set.
  var clr = d.color ? resolveColor(d.color) : "";
  if (d.gaugeStyle === "linear") {
    var e = document.createElement("div");
    e.style.cssText =
      "width:100%;min-width:48px;align-self:stretch;box-sizing:border-box";
    var top = document.createElement("div");
    top.style.cssText =
      "display:flex;justify-content:space-between;align-items:baseline;gap:6px;margin-bottom:2px";
    if (d.label) {
      var l = document.createElement("div");
      l.textContent = d.label;
      l.style.cssText = "font-size:10px;opacity:0.7";
      l.style.color = clr;
      top.appendChild(l);
    }
    if (d.currentValueLabel) {
      var cv = document.createElement("div");
      cv.textContent = d.currentValueLabel;
      cv.style.cssText = "font-size:11px;font-weight:600";
      cv.style.color = clr;
      top.appendChild(cv);
    }
    if (top.childNodes.length) e.appendChild(top);
    var tr = document.createElement("div");
    tr.style.cssText =
      "height:6px;border-radius:3px;overflow:hidden;width:100%";
    tr.style.background = track;
    var f = document.createElement("div");
    f.style.cssText = "height:100%;border-radius:3px";
    f.style.width = Math.max(Math.min(pct, 100), 0) + "%";
    f.style.background = tint;
    tickWhileRunning(d, function () {
      f.style.width = Math.max(Math.min(pctNow(), 100), 0) + "%";
    });
    tr.appendChild(f);
    e.appendChild(tr);
    var rl = rangeLabels(d, clr);
    if (rl) e.appendChild(rl);
    applyStyle(e, d);
    return e;
  }
  var wr = document.createElement("div");
  wr.style.textAlign = "center";
  var sw = document.createElement("div");
  sw.style.cssText = "width:56px;height:56px;position:relative;margin:0 auto";
  // A fixed frame sizes the ring (an island's 18 pt compact gauge), not the default 56 px.
  var fw = d.frame && (d.frame.width || d.frame.height);
  if (fw) {
    var ring = Math.min(d.frame.width || fw, d.frame.height || fw);
    sw.style.width = sw.style.height = ring + "px";
  }
  var s = svg("svg", { viewBox: "0 0 36 36" });
  s.style.cssText = "width:100%;height:100%";
  s.appendChild(
    svg("path", {
      d: RING,
      fill: "none",
      stroke: track,
      "stroke-width": "4",
    })
  );
  var arc = svg("path", {
    d: RING,
    fill: "none",
    stroke: tint,
    "stroke-width": "4",
    "stroke-dasharray": pct.toFixed(1) + ", 100",
    "stroke-linecap": "round",
  });
  s.appendChild(arc);
  tickWhileRunning(d, function () {
    arc.setAttribute("stroke-dasharray", pctNow().toFixed(1) + ", 100");
  });
  sw.appendChild(s);
  if (d.currentValueLabel) {
    var cv = document.createElement("div");
    cv.style.cssText =
      "position:absolute;inset:0;display:flex;align-items:center;justify-content:center;font-size:11px;font-weight:600";
    cv.style.color = clr;
    cv.textContent = d.currentValueLabel;
    sw.appendChild(cv);
  }
  wr.appendChild(sw);
  var rl2 = rangeLabels(d, clr);
  if (rl2) {
    rl2.style.width = "56px";
    rl2.style.margin = "2px auto 0";
    wr.appendChild(rl2);
  }
  if (d.label) {
    var l = document.createElement("div");
    l.style.cssText = "font-size:10px;margin-top:2px;opacity:0.7";
    l.style.color = clr;
    l.textContent = d.label;
    wr.appendChild(l);
  }
  applyStyle(wr, d);
  return wr;
}

var CHART_COLORS = [
  "#3b82f6",
  "#22c55e",
  "#f97316",
  "#ef4444",
  "#a855f7",
  "#eab308",
  "#ec4899",
  "#14b8a6",
];

type Series = { name: string; color: string; points: ElNode[] };

/** Points grouped by `series` (first-seen order); color = a named series' first point
 *  color, else tint for a lone series, else the palette. */
function chartSeries(pts: ElNode[], tint: string): Series[] {
  var out: Series[] = [];
  var byName: Record<string, Series> = {};
  pts.forEach(function (p) {
    var name = p.series == null ? "" : String(p.series);
    var sr = byName[name];
    if (!sr) {
      sr = byName[name] = { name: name, color: "", points: [] };
      out.push(sr);
    }
    // Unnamed (legacy) series keep the tint; per-point colors still apply to bars.
    if (!sr.color && p.color && name) sr.color = resolveColor(p.color);
    sr.points.push(p);
  });
  out.forEach(function (sr, i) {
    if (!sr.color) sr.color = out.length > 1 ? CHART_COLORS[i % CHART_COLORS.length] : tint;
  });
  return out;
}

/** SVG path through `xy`: straight, step (hold then jump), or monotone cubic (Fritsch–Carlson). */
function curvePath(xy: number[][], mode: string | undefined): string {
  if (!xy.length) return "";
  var d = "M" + xy[0][0] + "," + xy[0][1];
  if (mode === "step") {
    for (var i = 1; i < xy.length; i++) d += " L" + xy[i][0] + "," + xy[i - 1][1] + " L" + xy[i][0] + "," + xy[i][1];
    return d;
  }
  if (mode !== "monotone" || xy.length < 3) {
    for (var j = 1; j < xy.length; j++) d += " L" + xy[j][0] + "," + xy[j][1];
    return d;
  }
  var n = xy.length,
    sl: number[] = [],
    m: number[] = [];
  for (var k = 0; k < n - 1; k++) sl.push((xy[k + 1][1] - xy[k][1]) / (xy[k + 1][0] - xy[k][0] || 1));
  m[0] = sl[0];
  m[n - 1] = sl[n - 2];
  for (var q = 1; q < n - 1; q++) m[q] = sl[q - 1] * sl[q] <= 0 ? 0 : (sl[q - 1] + sl[q]) / 2;
  for (var r = 0; r < n - 1; r++) {
    if (sl[r] === 0) {
      m[r] = m[r + 1] = 0;
      continue;
    }
    var al = m[r] / sl[r],
      be = m[r + 1] / sl[r],
      h = al * al + be * be;
    if (h > 9) {
      var t = 3 / Math.sqrt(h);
      m[r] = t * al * sl[r];
      m[r + 1] = t * be * sl[r];
    }
  }
  for (var c = 0; c < n - 1; c++) {
    var dx = (xy[c + 1][0] - xy[c][0]) / 3;
    d +=
      " C" +
      (xy[c][0] + dx) + "," + (xy[c][1] + m[c] * dx) + " " +
      (xy[c + 1][0] - dx) + "," + (xy[c + 1][1] - m[c + 1] * dx) + " " +
      xy[c + 1][0] + "," + xy[c + 1][1];
  }
  return d;
}

function axisText(t: string): HTMLElement {
  var sp = document.createElement("span");
  sp.textContent = t;
  sp.style.cssText = "font-size:8px;color:#999;white-space:nowrap";
  return sp;
}

function fmtNum(v: number): string {
  return String(+v.toFixed(2));
}

/** Legend row: swatch + name per entry. */
function legendRow(items: { name: string; color: string }[]): HTMLElement {
  var row = document.createElement("div");
  row.style.cssText = "display:flex;flex-wrap:wrap;gap:2px 8px;margin-top:3px";
  items.forEach(function (it) {
    var cell = document.createElement("span");
    cell.style.cssText = "display:inline-flex;align-items:center;gap:3px";
    var sw = document.createElement("span");
    sw.style.cssText = "width:6px;height:6px;border-radius:3px;flex-shrink:0";
    sw.style.background = it.color;
    cell.appendChild(sw);
    cell.appendChild(axisText(it.name));
    row.appendChild(cell);
  });
  return row;
}

/** Wrap `plot` with y-axis labels (left), x labels (below) and a legend when asked;
 *  returns `plot` itself when nothing is added (legacy DOM). `plotH` 0 = fill the box
 *  height (Swift Charts mode). */
function chartFrame(
  d: ElNode,
  plot: HTMLElement | SVGElement,
  plotH: number,
  yLo: number,
  yHi: number,
  xLabels: string[] | null,
  legend: { name: string; color: string }[] | null,
): HTMLElement | SVGElement {
  var yAxis = d.showYAxis === true;
  if (!yAxis && !xLabels && !legend) return plot;
  var fill = plotH <= 0;
  var box = document.createElement("div");
  box.style.cssText = "display:flex;flex-direction:column;min-width:0" + (fill ? ";align-self:stretch;min-height:0" : "");
  var body = document.createElement("div");
  body.style.cssText = "display:flex;gap:3px;min-width:0" + (fill ? ";flex:1;min-height:0" : "");
  if (yAxis) {
    var col = document.createElement("div");
    col.style.cssText = "display:flex;flex-direction:column;justify-content:space-between;align-items:flex-end";
    if (!fill) col.style.height = plotH + "px";
    col.appendChild(axisText(fmtNum(yHi)));
    col.appendChild(axisText(fmtNum(yLo)));
    if (fill && xLabels) {
      // The column spans the plot only: a hidden x-label row keeps it off the labels.
      var ycol = document.createElement("div");
      ycol.style.cssText = "display:flex;flex-direction:column";
      col.style.flex = "1";
      ycol.appendChild(col);
      var gap = axisText("0");
      gap.style.visibility = "hidden";
      gap.style.marginTop = "1px";
      ycol.appendChild(gap);
      body.appendChild(ycol);
    } else body.appendChild(col);
  }
  var right = document.createElement("div");
  right.style.cssText = "display:flex;flex-direction:column;flex:1;min-width:0" + (fill ? ";min-height:0" : "");
  right.appendChild(plot);
  if (xLabels) {
    var xr = document.createElement("div");
    xr.style.cssText = "display:flex;justify-content:space-between;margin-top:1px";
    xLabels.forEach(function (t) {
      xr.appendChild(axisText(t));
    });
    right.appendChild(xr);
  }
  body.appendChild(right);
  box.appendChild(body);
  if (legend) box.appendChild(legendRow(legend));
  return box;
}

export function renderChart(d: ElNode): HTMLElement | SVGElement {
  var pts: ElNode[] = d.chartData || [];
  var tint = resolveColor(d.tint || "accent"); // Swift: `tint ?? Color.accentColor`
  var series = chartSeries(pts, tint);
  var vals = pts.map(function (p) {
    return Number(p.value) || 0;
  });
  var yLo = d.yMin != null ? d.yMin : Math.min.apply(null, vals.concat([0]));
  var yHi = d.yMax != null ? d.yMax : Math.max.apply(null, vals.concat([1]));
  var span = yHi - yLo || 1;
  var norm = function (v: number) {
    return Math.max(0, Math.min(1, (v - yLo) / span));
  };
  var legendOf = function (): { name: string; color: string }[] | null {
    return d.showLegend === true && series.some((sr) => sr.name) ? series : null;
  };
  // Distinct labels in first-seen order (x axis).
  var labels: string[] = [];
  pts.forEach(function (p) {
    var l = p.label == null ? "" : String(p.label);
    if (labels.indexOf(l) < 0) labels.push(l);
  });

  if (d.chartType === "line" || d.chartType === "area" || d.chartType === "point") {
    var w = 200,
      h = 60;
    var s = svg("svg", { viewBox: "0 0 " + w + " " + h });
    s.style.cssText = "width:100%;height:60px";
    // Extended charts fill their box; strokes / dots keep their size via non-scaling strokes.
    var ext =
      series.length > 1 || d.showYAxis === true || d.showXAxis === true || d.showLegend === true || d.chartType === "point";
    if (ext) {
      s.setAttribute("preserveAspectRatio", "none");
      s.style.overflow = "visible"; // dots / strokes on the edges stay whole
      // Like Swift Charts: the plot takes the height the stack gives it.
      s.style.height = "auto";
      s.style.flex = "1";
      s.style.minHeight = "40px";
    }
    // Point charts draw HTML dots in a box that takes the svg's place.
    var dots = document.createElement("div");
    dots.style.cssText = "position:relative;width:100%;flex:1;min-height:40px";
    series.forEach(function (sr) {
      var xy = sr.points.map(function (p, i) {
        return [(i / Math.max(sr.points.length - 1, 1)) * w, h - norm(Number(p.value) || 0) * h];
      });
      if (d.chartType === "point") {
        // Round HTML dots over the plot box: a zero-length round-capped SVG stroke in a
        // stretched (preserveAspectRatio none) svg turns into a tall spike on WebKitGTK.
        xy.forEach(function (pt) {
          var dot = document.createElement("span");
          dot.style.cssText =
            "position:absolute;width:6px;height:6px;border-radius:3px;transform:translate(-50%,-50%)";
          dot.style.left = (pt[0] / w) * 100 + "%";
          dot.style.top = (pt[1] / h) * 100 + "%";
          dot.style.background = sr.color;
          dots.appendChild(dot);
        });
        return;
      }
      var line = curvePath(xy, d.interpolation);
      if (d.chartType === "area" && xy.length) {
        s.appendChild(
          svg("path", {
            d: line + " L" + xy[xy.length - 1][0] + "," + h + " L" + xy[0][0] + "," + h + " Z",
            fill: sr.color,
            opacity: "0.3",
          })
        );
      }
      s.appendChild(
        svg("path", {
          d: line,
          fill: "none",
          stroke: sr.color,
          "stroke-width": "2",
          "vector-effect": ext ? "non-scaling-stroke" : null,
        })
      );
    });
    // x labels: Apple / desktop show them unless `showXAxis: false` (docs); the legacy
    // sparkline (no v2 option) stays bare.
    var xs = ext ? (d.showXAxis !== false ? labels : null) : d.showXAxis === true ? labels : null;
    var framed = chartFrame(d, d.chartType === "point" ? dots : s, ext ? 0 : h, yLo, yHi, xs, legendOf());
    applyStyle(framed, d);
    return framed;
  }
  if (d.chartType === "pie") {
    var total = pts.reduce(function (sum: number, p: ElNode) {
      return sum + p.value;
    }, 0);
    var r = 40,
      cx = 50,
      cy = 50,
      ca = -90;
    var ri = d.innerRadius > 0 && d.innerRadius < 1 ? r * d.innerRadius : 0;
    var pie = svg("svg", { viewBox: "0 0 100 100" });
    pie.style.cssText = "width:80px;height:80px";
    var slices: { name: string; color: string }[] = [];
    pts.forEach(function (p: ElNode, i: number) {
      var angle = (p.value / Math.max(total, 1)) * 360;
      var sr = (ca * Math.PI) / 180,
        er = ((ca + angle) * Math.PI) / 180;
      var x1 = cx + r * Math.cos(sr),
        y1 = cy + r * Math.sin(sr);
      var x2 = cx + r * Math.cos(er),
        y2 = cy + r * Math.sin(er);
      var lf = angle > 180 ? 1 : 0;
      var color = p.color ? resolveColor(p.color) : CHART_COLORS[i % CHART_COLORS.length];
      // Donut: annular sector (outer arc, then the inner arc back).
      var pd = ri
        ? "M" + x1 + "," + y1 + " A" + r + "," + r + " 0 " + lf + ",1 " + x2 + "," + y2 +
          " L" + (cx + ri * Math.cos(er)) + "," + (cy + ri * Math.sin(er)) +
          " A" + ri + "," + ri + " 0 " + lf + ",0 " + (cx + ri * Math.cos(sr)) + "," + (cy + ri * Math.sin(sr)) + " Z"
        : "M" + cx + "," + cy + " L" + x1 + "," + y1 + " A" + r + "," + r + " 0 " + lf + ",1 " + x2 + "," + y2 + " Z";
      pie.appendChild(svg("path", { d: pd, fill: color }));
      slices.push({ name: p.label == null ? "" : String(p.label), color: color });
      ca += angle;
    });
    var pieOut = chartFrame(d, pie, 80, 0, 0, null, d.showLegend === true ? slices : null);
    applyStyle(pieOut, d);
    return pieOut;
  }
  var e = document.createElement("div");
  e.style.cssText = "display:flex;align-items:flex-end;gap:4px;height:70px";
  var showX = d.showXAxis !== false;
  var bar = function (v: number, color: string): HTMLElement {
    var b = document.createElement("div");
    b.style.width = "100%";
    b.style.height = Math.max(norm(v) * 60, 2) + "px";
    b.style.background = color;
    b.style.borderRadius = "2px";
    return b;
  };
  var column = function (bars: HTMLElement, label: string): void {
    var col = document.createElement("div");
    col.style.cssText =
      "flex:1;display:flex;flex-direction:column;align-items:center;gap:2px";
    col.appendChild(bars);
    if (showX) {
      var lb = document.createElement("span");
      lb.textContent = label;
      lb.style.cssText = "font-size:8px;color:#999";
      col.appendChild(lb);
    }
    e.appendChild(col);
  };
  if (series.length > 1) {
    // Grouped: one bar per series under each distinct label.
    labels.forEach(function (label) {
      var grp = document.createElement("div");
      grp.style.cssText = "display:flex;align-items:flex-end;gap:1px;width:100%";
      series.forEach(function (sr) {
        var p = sr.points.find(function (q) {
          return String(q.label == null ? "" : q.label) === label;
        });
        grp.appendChild(bar(p ? Number(p.value) || 0 : 0, sr.color));
      });
      column(grp, label);
    });
  } else {
    // One series: a column per point (duplicate labels stay separate), per-point colors.
    pts.forEach(function (p: ElNode) {
      column(bar(Number(p.value) || 0, p.color ? resolveColor(p.color) : tint), p.label);
    });
  }
  var barOut = chartFrame(d, e, 70, yLo, yHi, null, legendOf());
  applyStyle(barOut, d);
  return barOut;
}

export function renderList(d: ElNode): HTMLElement {
  var emitAction = boundEmit();
  var e = document.createElement("div");
  e.style.cssText =
    "display:flex;flex-direction:column;gap:" + (d.spacing || 4) + "px";
  // Rich rows (children) or typed items.
  if (d.children && d.children.length) {
    d.children.forEach(function (c: ElNode) {
      e.appendChild(renderEl(c, "vertical"));
    });
    applyStyle(e, d);
    return e;
  }
  var items = d.items || [];
  var anyCheck = items.some(function (it: ElNode) {
    return Object.prototype.hasOwnProperty.call(it, "checked");
  });
  items.forEach(function (it: ElNode) {
    var row = document.createElement("div");
    row.style.cssText = "display:flex;align-items:center;gap:6px;min-width:0";
    var hasCheck = Object.prototype.hasOwnProperty.call(it, "checked");
    if (hasCheck || anyCheck) {
      var check = document.createElement("span");
      check.style.cssText =
        "font-size:12px;width:14px;text-align:center;flex-shrink:0;color:" +
        (hasCheck ? (it.checked ? "#22c55e" : "#9ca3af") : "transparent");
      check.textContent = hasCheck ? (it.checked ? "\u2713" : "\u25cb") : "\u00a0";
      row.appendChild(check);
    }
    var txt = document.createElement("span");
    txt.textContent = it.text || "";
    txt.style.cssText =
      "font-size:" +
      (d.fontSize || 13) +
      "px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;min-width:0;flex:1";
    txt.style.color = d.color ? resolveColor(d.color) : "inherit";
    row.appendChild(txt);
    if (it.action) {
      row.style.cursor = "pointer";
      row.onclick = function () {
        emitAction(it.action, it.payload);
      };
    }
    e.appendChild(row);
  });
  applyStyle(e, d);
  return e;
}
