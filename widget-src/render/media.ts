import { applyStyle, resolveColor, svg } from "../style";
import type { ElNode } from "../types";

export function systemGlyph(name: string | null | undefined): string {
  var n = String(name || "").toLowerCase();
  var map: Record<string, string> = {
    star: "\u2b50",
    "star.fill": "\u2b50",
    heart: "\u2764",
    "heart.fill": "\u2764",
    person: "\u1f464",
    "person.fill": "\u1f464",
    "person.2.fill": "\u1f465",
    "person.badge.plus": "\u1f464",
    gear: "\u2699",
    gearshape: "\u2699",
    "gearshape.fill": "\u2699",
    checkmark: "\u2713",
    "checkmark.circle": "\u2713",
    "checkmark.circle.fill": "\u2713",
    xmark: "\u2715",
    "xmark.circle": "\u2715",
    plus: "+",
    "plus.circle": "+",
    minus: "\u2212",
    "minus.circle": "\u2212",
    bell: "\u1f514",
    "bell.fill": "\u1f514",
    house: "\u2302",
    "house.fill": "\u2302",
    magnifyingglass: "\u1f50d",
    calendar: "\u1f4c5",
    "calendar.badge.clock": "\u1f4c5",
    "calendar.circle": "\u1f4c5",
    clock: "\u23f1",
    "clock.fill": "\u23f1",
    globe: "\u1f310",
    "cloud.fill": "\u2601",
    "cloud.sun.fill": "\u26c5",
    "cloud.rain.fill": "\u2614",
    "cloud.bolt.fill": "\u26a1",
    "sun.max": "\u2600",
    "sun.max.fill": "\u2600",
    "moon.fill": "\u263e",
    "moon.stars.fill": "\u263e",
    bolt: "\u26a1",
    "bolt.fill": "\u26a1",
    "location.fill": "\u2302",
    "flame.fill": "\u1f525",
    "drop.fill": "\u25cf",
    "leaf.fill": "\u273f",
    "eye.fill": "\u25c9",
    "bubble.left.fill": "\u25d1",
    "music.note": "\u266a",
    "music.note.list": "\u266a",
    hourglass: "\u23f3",
    "hourglass.bottomhalf.filled": "\u23f3",
    "figure.run": "\u27a4",
    "paintpalette.fill": "\u2698",
    "quote.opening": "\u201c",
    "bitcoinsign.circle.fill": "\u20bf",
    "battery.100": "\u25ae",
    iphone: "\u25af",
    ipad: "\u25ad",
    applewatch: "\u231a",
    airpodspro: "\u25cb",
    desktopcomputer: "\u25a3",
    internaldrive: "\u25a0",
    network: "\u2261",
    "envelope.fill": "\u2709",
    envelope: "\u2709",
    "phone.fill": "\u260e",
    phone: "\u260e",
    "message.fill": "\u1f4ac",
    trash: "\u1f5d1",
    "trash.fill": "\u1f5d1",
    folder: "\u1f4c1",
    "folder.fill": "\u1f4c1",
    doc: "\u1f4c4",
    "doc.fill": "\u1f4c4",
    photo: "\u1f5bc",
    "photo.fill": "\u1f5bc",
    camera: "\u1f4f7",
    "camera.fill": "\u1f4f7",
    map: "\u1f5fa",
    "map.fill": "\u1f5fa",
    cart: "\u1f6d2",
    "cart.fill": "\u1f6d2",
    creditcard: "\u1f4b3",
    "creditcard.fill": "\u1f4b3",
    wifi: "\u1f4f6",
    "antenna.radiowaves.left.and.right": "\u1f4e1",
    "lock.fill": "\u1f512",
    lock: "\u1f512",
    "lock.open.fill": "\u1f513",
    "key.fill": "\u1f511",
    "paperplane.fill": "\u27a4",
    "arrow.right": "\u2192",
    "arrow.left": "\u2190",
    "arrow.up": "\u2191",
    "arrow.down": "\u2193",
    "chevron.right": "\u203a",
    "chevron.left": "\u2039",
    "chevron.up": "\u02c6",
    "chevron.down": "\u02c7",
    "info.circle": "\u2139",
    "info.circle.fill": "\u2139",
    "exclamationmark.triangle": "\u26a0",
    "exclamationmark.triangle.fill": "\u26a0",
    "play.fill": "\u25b6",
    "pause.fill": "\u23f8",
    "stop.fill": "\u23f9",
    "forward.fill": "\u23e9",
    "backward.fill": "\u23ea",
  };
  if (map[n]) return map[n];
  // Never first-letter fallback — looks like a bug.
  return "\u25CF";
}

export function renderImage(d: ElNode): HTMLElement {
  var e = document.createElement("span");
  e.style.display = "inline-flex";
  e.style.alignItems = "center";
  e.style.justifyContent = "center";
  if (d.data) {
    var img = document.createElement("img");
    img.src =
      String(d.data).indexOf("data:") === 0
        ? d.data
        : "data:image/png;base64," + d.data;
    img.width = d.size || 24;
    img.height = d.size || 24;
    img.style.objectFit = d.contentMode === "fill" ? "cover" : "contain";
    e.appendChild(img);
  } else if (d.url) {
    var img = document.createElement("img");
    img.src = d.url;
    img.width = d.size || 24;
    img.height = d.size || 24;
    img.style.objectFit = d.contentMode === "fill" ? "cover" : "contain";
    e.appendChild(img);
  } else if (d.systemName) {
    var sz = d.size || 24;
    e.textContent = systemGlyph(d.systemName);
    e.style.fontSize = sz + "px";
    e.style.lineHeight = "1";
    e.style.width = sz + "px";
    e.style.height = sz + "px";
    e.style.minWidth = sz + "px";
    e.style.flexShrink = "0";
    e.style.boxSizing = "border-box";
    e.style.color = d.color ? resolveColor(d.color) : "inherit";
  } else {
    e.textContent = "\u25CF";
    e.style.fontSize = (d.size || 24) + "px";
    e.style.color = d.color ? resolveColor(d.color) : "inherit";
  }
  applyStyle(e, d);
  return e;
}

export function renderShape(d: ElNode): HTMLElement {
  var sz = d.size || 24,
    fill = d.fill ? resolveColor(d.fill) : "#4CAF50";
  var stroke = d.stroke ? resolveColor(d.stroke) : undefined,
    sw = d.strokeWidth || 1;
  var e = document.createElement("div");
  e.style.background = fill;
  e.style.flexShrink = "0";
  if (stroke) e.style.border = sw + "px solid " + stroke;
  if (d.shapeType === "circle") {
    e.style.width = sz + "px";
    e.style.height = sz + "px";
    e.style.borderRadius = "50%";
  } else if (d.shapeType === "capsule") {
    e.style.width = sz * 2 + "px";
    e.style.height = sz + "px";
    e.style.borderRadius = sz / 2 + "px";
  } else {
    e.style.width = sz + "px";
    e.style.height = sz + "px";
    if (d.cornerRadius) e.style.borderRadius = d.cornerRadius + "px";
  }
  applyStyle(e, d);
  return e;
}

export function renderCanvas(d: ElNode): SVGElement {
  var cw = d.width || 100,
    ch = d.height || 100;
  var s = svg("svg", {
    width: cw,
    height: ch,
    viewBox: "0 0 " + cw + " " + ch,
  });
  (d.elements || []).forEach(function (cmd: ElNode) {
    switch (cmd.draw) {
      case "circle":
        s.appendChild(
          svg("circle", {
            cx: cmd.cx,
            cy: cmd.cy,
            r: cmd.r,
            fill: cmd.fill ? resolveColor(cmd.fill) : "none",
            stroke: cmd.stroke ? resolveColor(cmd.stroke) : "none",
            "stroke-width": cmd.strokeWidth || 1,
          })
        );
        break;
      case "line":
        s.appendChild(
          svg("line", {
            x1: cmd.x1,
            y1: cmd.y1,
            x2: cmd.x2,
            y2: cmd.y2,
            stroke: cmd.stroke ? resolveColor(cmd.stroke) : "#fff",
            "stroke-width": cmd.strokeWidth || 1,
            "stroke-linecap": cmd.lineCap || "butt",
          })
        );
        break;
      case "rect":
        s.appendChild(
          svg("rect", {
            x: cmd.x,
            y: cmd.y,
            width: cmd.width,
            height: cmd.height,
            rx: cmd.cornerRadius || 0,
            ry: cmd.cornerRadius || 0,
            fill: cmd.fill ? resolveColor(cmd.fill) : "none",
            stroke: cmd.stroke ? resolveColor(cmd.stroke) : "none",
            "stroke-width": cmd.strokeWidth || 1,
          })
        );
        break;
      case "arc": {
        var cx2 = cmd.cx || 0,
          cy2 = cmd.cy || 0,
          r2 = cmd.r || 10;
        var sa = ((cmd.startAngle || 0) * Math.PI) / 180,
          ea = ((cmd.endAngle || 360) * Math.PI) / 180;
        var sx = cx2 + r2 * Math.cos(sa),
          sy = cy2 + r2 * Math.sin(sa);
        var ex = cx2 + r2 * Math.cos(ea),
          ey = cy2 + r2 * Math.sin(ea);
        var lf = ea - sa > Math.PI ? 1 : 0;
        var pd = cmd.fill
          ? "M" +
            cx2 +
            "," +
            cy2 +
            " L" +
            sx +
            "," +
            sy +
            " A" +
            r2 +
            "," +
            r2 +
            " 0 " +
            lf +
            " 1 " +
            ex +
            "," +
            ey +
            " Z"
          : "M" +
            sx +
            "," +
            sy +
            " A" +
            r2 +
            "," +
            r2 +
            " 0 " +
            lf +
            " 1 " +
            ex +
            "," +
            ey;
        s.appendChild(
          svg("path", {
            d: pd,
            fill: cmd.fill ? resolveColor(cmd.fill) : "none",
            stroke: cmd.stroke ? resolveColor(cmd.stroke) : "none",
            "stroke-width": cmd.strokeWidth || 1,
          })
        );
        break;
      }
      case "text": {
        var t = svg("text", {
          x: cmd.x,
          y: cmd.y,
          "font-size": cmd.fontSize || 12,
          fill: cmd.color ? resolveColor(cmd.color) : "#fff",
          "text-anchor":
            cmd.anchor === "middle"
              ? "middle"
              : cmd.anchor === "end"
                ? "end"
                : "start",
        });
        t.textContent = cmd.content || "";
        s.appendChild(t);
        break;
      }
      case "path":
        s.appendChild(
          svg("path", {
            d: cmd.d,
            fill: cmd.fill ? resolveColor(cmd.fill) : "none",
            stroke: cmd.stroke ? resolveColor(cmd.stroke) : "none",
            "stroke-width": cmd.strokeWidth || 1,
          })
        );
        break;
    }
  });
  applyStyle(s, d);
  return s;
}
