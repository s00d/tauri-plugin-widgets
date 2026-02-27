import type { WidgetConfig, WidgetElement, CanvasDrawCommand } from "tauri-plugin-widgets-api";
import type { PresetDef } from "./types";

function buildClockFace(size: number, hAngle: number, mAngle: number, sAngle: number): CanvasDrawCommand[] {
  const cx = size / 2;
  const cy = size / 2;
  const outerR = size * 0.47;
  const majorInnerR = size * 0.37;
  const minorInnerR = size * 0.41;
  const markerOuterR = size * 0.43;
  const hourLen = size * 0.23;
  const minLen = size * 0.31;
  const secLen = size * 0.35;

  const markers: CanvasDrawCommand[] = [];
  for (let i = 0; i < 12; i++) {
    const a = (i * 30 - 90) * Math.PI / 180;
    const r1 = i % 3 === 0 ? majorInnerR : minorInnerR;
    const r2 = markerOuterR;
    markers.push({
      draw: "line",
      x1: cx + r1 * Math.cos(a), y1: cy + r1 * Math.sin(a),
      x2: cx + r2 * Math.cos(a), y2: cy + r2 * Math.sin(a),
      stroke: "#e2e8f0", strokeWidth: i % 3 === 0 ? Math.max(1.4, size * 0.015) : Math.max(1, size * 0.01), lineCap: "round",
    });
  }

  return [
    { draw: "circle", cx, cy, r: outerR, fill: "#1e293b", stroke: "#475569", strokeWidth: Math.max(1.5, size * 0.015) },
    ...markers,
    { draw: "line", x1: cx, y1: cy, x2: cx + hourLen * Math.cos(hAngle), y2: cy + hourLen * Math.sin(hAngle), stroke: "#f1f5f9", strokeWidth: Math.max(2.4, size * 0.03), lineCap: "round" },
    { draw: "line", x1: cx, y1: cy, x2: cx + minLen * Math.cos(mAngle), y2: cy + minLen * Math.sin(mAngle), stroke: "#cbd5e1", strokeWidth: Math.max(1.6, size * 0.02), lineCap: "round" },
    { draw: "line", x1: cx, y1: cy, x2: cx + secLen * Math.cos(sAngle), y2: cy + secLen * Math.sin(sAngle), stroke: "#ef4444", strokeWidth: Math.max(1, size * 0.01), lineCap: "round" },
    { draw: "circle", cx, cy, r: Math.max(2.5, size * 0.026), fill: "#ef4444" },
  ];
}

function buildClockConfig(): WidgetConfig {
  const now = new Date();
  const h = now.getHours() % 12;
  const m = now.getMinutes();
  const s = now.getSeconds();
  const hAngle = ((h + m / 60) * 30 - 90) * Math.PI / 180;
  const mAngle = ((m + s / 60) * 6 - 90) * Math.PI / 180;
  const sAngle = (s * 6 - 90) * Math.PI / 180;
  const smallSize = 136;
  const mediumSize = 288;
  const largeSize = 326;
  const clockCanvasSmall: WidgetElement = { type: "canvas", width: smallSize, height: smallSize, elements: buildClockFace(smallSize, hAngle, mAngle, sAngle) };
  const clockCanvasMedium: WidgetElement = { type: "canvas", width: mediumSize, height: mediumSize, elements: buildClockFace(mediumSize, hAngle, mAngle, sAngle) };
  const clockCanvasLarge: WidgetElement = { type: "canvas", width: largeSize, height: largeSize, elements: buildClockFace(largeSize, hAngle, mAngle, sAngle) };
  const timeStr = `${String(now.getHours()).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  const dateStr = now.toLocaleDateString("en-US", { weekday: "long", month: "short", day: "numeric" });
  const tzOffset = now.getTimezoneOffset();
  const tz = "UTC" + (tzOffset <= 0 ? "+" : "-") + Math.abs(tzOffset / 60);

  const small: WidgetElement = {
    type: "vstack", spacing: 6, padding: 12, cornerRadius: 16,
    background: { gradientType: "linear", colors: ["#0f172a", "#1e293b"], direction: "topToBottom" },
    children: [
      clockCanvasSmall,
      { type: "text", content: timeStr, fontSize: 13, fontWeight: "semibold", color: "#94a3b8", alignment: "center" },
    ],
  };

  const medium: WidgetElement = {
    type: "hstack", spacing: 16, padding: 14, cornerRadius: 16,
    background: { gradientType: "linear", colors: ["#0f172a", "#1e293b"], direction: "leadingToTrailing" },
    children: [
      { ...clockCanvasMedium },
      { type: "vstack", spacing: 8, flex: 1, children: [
        { type: "text", content: timeStr, fontSize: 28, fontWeight: "bold", color: "#f1f5f9" },
        { type: "text", content: dateStr, fontSize: 13, color: "#94a3b8" },
        { type: "spacer" },
        { type: "label", text: tz, systemName: "globe", fontSize: 12, color: "#64748b" },
      ] },
    ],
  };

  const large: WidgetElement = {
    type: "vstack", spacing: 8, padding: 14, cornerRadius: 16,
    background: { gradientType: "linear", colors: ["#0f172a", "#1e293b", "#0f172a"], direction: "topToBottom" },
    children: [
      { type: "text", content: "World Clock", fontSize: 16, fontWeight: "bold", color: "#f1f5f9", alignment: "center" },
      { type: "hstack", spacing: 12, children: [
        { type: "vstack", spacing: 4, flex: 1, children: [
          { ...clockCanvasLarge },
          { type: "text", content: "Local", fontSize: 11, color: "#94a3b8", alignment: "center" },
        ] },
        { type: "vstack", spacing: 4, flex: 1, children: [
          { ...clockCanvasLarge },
          { type: "text", content: "Tokyo", fontSize: 11, color: "#94a3b8", alignment: "center" },
        ] },
      ] },
      { type: "text", content: now.toLocaleDateString("en-US", { weekday: "short", month: "short", day: "numeric" }), fontSize: 12, color: "#64748b", alignment: "center" },
    ],
  };

  return { small, medium, large };
}

export const analogClock: PresetDef = {
  icon: "\u{1F570}",
  name: "Analog Clock",
  config: buildClockConfig(),
  builder: buildClockConfig,
  intervalMs: 1000,
};
