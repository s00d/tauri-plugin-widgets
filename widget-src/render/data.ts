import { applyStyle, resolveColor, svg, tintTrack } from "../style";
import { emitAction } from "../ctx";
import type { ElNode } from "../types";

export function renderProgress(d: ElNode): HTMLElement | SVGElement {
  var pct = ((d.value / (d.total || 1)) * 100).toFixed(1);
  var tint = d.tint ? resolveColor(d.tint) : "#4CAF50";
  var track = tintTrack(d.tint);
  if (d.barStyle === "circular") {
    var w = document.createElement("div");
    w.style.cssText = "width:40px;height:40px;position:relative";
    var s = svg("svg", { viewBox: "0 0 36 36" });
    s.style.cssText = "width:100%;height:100%";
    s.appendChild(
      svg("path", {
        d: "M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831",
        fill: "none",
        stroke: track,
        "stroke-width": "3",
      })
    );
    s.appendChild(
      svg("path", {
        d: "M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831",
        fill: "none",
        stroke: tint,
        "stroke-width": "3",
        "stroke-dasharray": pct + ", 100",
        "stroke-linecap": "round",
      })
    );
    w.appendChild(s);
    applyStyle(w, d);
    return w;
  }
  var e = document.createElement("div");
  e.style.cssText =
    "flex:1 1 auto;width:100%;min-width:48px;align-self:stretch;box-sizing:border-box";
  if (d.label) {
    var l = document.createElement("div");
    l.textContent = d.label;
    l.style.cssText = "font-size:10px;margin-bottom:2px";
    l.style.color = d.color ? resolveColor(d.color) : tint;
    e.appendChild(l);
  }
  var tr = document.createElement("div");
  tr.style.cssText =
    "height:6px;border-radius:3px;overflow:hidden;width:100%";
  tr.style.background = track;
  var f = document.createElement("div");
  f.style.cssText = "height:100%;border-radius:3px";
  f.style.width = pct + "%";
  f.style.background = tint;
  tr.appendChild(f);
  e.appendChild(tr);
  applyStyle(e, d);
  return e;
}

export function renderGauge(d: ElNode): HTMLElement | SVGElement {
  var v = d.value || 0,
    lo = d.min || 0,
    hi = d.max || 1;
  var pct = ((v - lo) / Math.max(hi - lo, 0.0001)) * 100;
  var tint = d.tint ? resolveColor(d.tint) : "#4CAF50";
  var track = tintTrack(d.tint);
  var clr = d.color ? resolveColor(d.color) : tint;
  if (d.gaugeStyle === "linear") {
    var e = document.createElement("div");
    e.style.cssText =
      "flex:1 1 auto;width:100%;min-width:48px;align-self:stretch;box-sizing:border-box";
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
    tr.appendChild(f);
    e.appendChild(tr);
    applyStyle(e, d);
    return e;
  }
  var wr = document.createElement("div");
  wr.style.textAlign = "center";
  var sw = document.createElement("div");
  sw.style.cssText = "width:56px;height:56px;position:relative;margin:0 auto";
  var s = svg("svg", { viewBox: "0 0 36 36" });
  s.style.cssText = "width:100%;height:100%";
  s.appendChild(
    svg("path", {
      d: "M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831",
      fill: "none",
      stroke: track,
      "stroke-width": "4",
    })
  );
  s.appendChild(
    svg("path", {
      d: "M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831",
      fill: "none",
      stroke: tint,
      "stroke-width": "4",
      "stroke-dasharray": pct.toFixed(1) + ", 100",
      "stroke-linecap": "round",
    })
  );
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

export function renderChart(d: ElNode): HTMLElement | SVGElement {
  var pts = d.chartData || [];
  var maxV = Math.max.apply(
    null,
    pts.map(function (p: ElNode) {
      return p.value;
    }).concat([1])
  );
  var tint = d.tint ? resolveColor(d.tint) : "#4CAF50";
  if (d.chartType === "line" || d.chartType === "area") {
    var w = 200,
      h = 60;
    var s = svg("svg", { viewBox: "0 0 " + w + " " + h });
    s.style.cssText = "width:100%;height:60px";
    var pathD = pts
      .map(function (p: ElNode, i: number) {
        var x = (i / Math.max(pts.length - 1, 1)) * w;
        var y = h - (p.value / maxV) * h;
        return (i === 0 ? "M" : "L") + x + "," + y;
      })
      .join(" ");
    if (d.chartType === "area") {
      var aD =
        "M0," +
        h +
        " " +
        pts
          .map(function (p: ElNode, i: number) {
            return (
              "L" +
              (i / Math.max(pts.length - 1, 1)) * w +
              "," +
              (h - (p.value / maxV) * h)
            );
          })
          .join(" ") +
        " L" +
        w +
        "," +
        h +
        " Z";
      s.appendChild(svg("path", { d: aD, fill: tint, opacity: "0.3" }));
    }
    s.appendChild(
      svg("path", {
        d: pathD,
        fill: "none",
        stroke: tint,
        "stroke-width": "2",
      })
    );
    applyStyle(s, d);
    return s;
  }
  if (d.chartType === "pie") {
    var total = pts.reduce(function (s: number, p: ElNode) {
      return s + p.value;
    }, 0);
    var dc = [
      "#3b82f6",
      "#22c55e",
      "#f97316",
      "#ef4444",
      "#a855f7",
      "#eab308",
      "#ec4899",
      "#14b8a6",
    ];
    var r = 40,
      cx = 50,
      cy = 50,
      ca = -90;
    var s = svg("svg", { viewBox: "0 0 100 100" });
    s.style.cssText = "width:80px;height:80px";
    pts.forEach(function (p: ElNode, i: number) {
      var angle = (p.value / Math.max(total, 1)) * 360;
      var sr = (ca * Math.PI) / 180,
        er = ((ca + angle) * Math.PI) / 180;
      var x1 = cx + r * Math.cos(sr),
        y1 = cy + r * Math.sin(sr);
      var x2 = cx + r * Math.cos(er),
        y2 = cy + r * Math.sin(er);
      var lf = angle > 180 ? 1 : 0;
      s.appendChild(
        svg("path", {
          d:
            "M" +
            cx +
            "," +
            cy +
            " L" +
            x1 +
            "," +
            y1 +
            " A" +
            r +
            "," +
            r +
            " 0 " +
            lf +
            ",1 " +
            x2 +
            "," +
            y2 +
            " Z",
          fill: p.color ? resolveColor(p.color) : dc[i % dc.length],
        })
      );
      ca += angle;
    });
    applyStyle(s, d);
    return s;
  }
  var e = document.createElement("div");
  e.style.cssText = "display:flex;align-items:flex-end;gap:4px;height:70px";
  pts.forEach(function (p: ElNode) {
    var col = document.createElement("div");
    col.style.cssText =
      "flex:1;display:flex;flex-direction:column;align-items:center;gap:2px";
    var bar = document.createElement("div");
    bar.style.width = "100%";
    bar.style.height = Math.max((p.value / maxV) * 60, 2) + "px";
    bar.style.background = p.color ? resolveColor(p.color) : tint;
    bar.style.borderRadius = "2px";
    col.appendChild(bar);
    var lb = document.createElement("span");
    lb.textContent = p.label;
    lb.style.cssText = "font-size:8px;color:#999";
    col.appendChild(lb);
    e.appendChild(col);
  });
  applyStyle(e, d);
  return e;
}

export function renderList(d: ElNode): HTMLElement {
  var e = document.createElement("div");
  e.style.cssText =
    "display:flex;flex-direction:column;gap:" + (d.spacing || 4) + "px";
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
