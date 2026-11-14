import type { WidgetConfig, WidgetElement, CanvasDrawCommand } from "tauri-plugin-widgets-api";
import type { PresetDef } from "./types";

/** Static dial (rim + hour marks): one `templates` entry per size, shared by every face. */
function dial(size: number): CanvasDrawCommand[] {
  return clockCommands(size, NaN, NaN, NaN, true);
}

/** Hands (and the hub) only. */
function hands(size: number, hAngle: number, mAngle: number, sAngle: number): CanvasDrawCommand[] {
  return clockCommands(size, hAngle, mAngle, sAngle, false);
}

function clockCommands(size: number, hAngle: number, mAngle: number, sAngle: number, dialOnly: boolean): CanvasDrawCommand[] {
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

  if (dialOnly) {
    return roundCommands([
      { draw: "circle", cx, cy, r: outerR, fill: "#1e293b", stroke: "#475569", strokeWidth: Math.max(1.5, size * 0.015) },
      ...markers,
    ]);
  }
  return roundCommands([
    { draw: "line", x1: cx, y1: cy, x2: cx + hourLen * Math.cos(hAngle), y2: cy + hourLen * Math.sin(hAngle), stroke: "#f1f5f9", strokeWidth: Math.max(2.4, size * 0.03), lineCap: "round" },
    { draw: "line", x1: cx, y1: cy, x2: cx + minLen * Math.cos(mAngle), y2: cy + minLen * Math.sin(mAngle), stroke: "#cbd5e1", strokeWidth: Math.max(1.6, size * 0.02), lineCap: "round" },
    ...(Number.isNaN(sAngle)
      ? []
      : [{ draw: "line", x1: cx, y1: cy, x2: cx + secLen * Math.cos(sAngle), y2: cy + secLen * Math.sin(sAngle), stroke: "#ef4444", strokeWidth: Math.max(1, size * 0.01), lineCap: "round" } as CanvasDrawCommand]),
    { draw: "circle", cx, cy, r: Math.max(2.5, size * 0.026), fill: "#ef4444" },
  ]);
}

/** One decimal is plenty for a face (keeps 15 timeline entries small). */
function roundCommands(cmds: CanvasDrawCommand[]): CanvasDrawCommand[] {
  return cmds.map(
    (c) =>
      Object.fromEntries(
        Object.entries(c).map(([k, v]) => [k, typeof v === "number" ? Math.round(v * 10) / 10 : v]),
      ) as CanvasDrawCommand,
  );
}

const SIZES = [120, 128, 148];

/** Dial template + a hands canvas on top (same size, both shrink alike). */
function clockCanvas(size: number, hAngle: number, mAngle: number, sAngle: number): WidgetElement {
  return {
    type: "zstack",
    // Let the canvas shrink inside WidgetKit families without clipping.
    flex: 1,
    children: [
      { type: "use", template: `dial${size}` },
      { type: "canvas", width: size, height: size, flex: 1, elements: hands(size, hAngle, mAngle, sAngle) },
    ],
  };
}

/** Digital time that runs on the device: a timer counting up from local midnight. */
function liveTime(now: Date, fontSize: number, fontWeight: "semibold" | "bold", color: string): WidgetElement {
  const midnight = new Date(now);
  midnight.setHours(0, 0, 0, 0);
  const end = new Date(midnight.getTime() + 24 * 3600 * 1000);
  return {
    type: "timer", counting: "up", showHours: true,
    startDate: midnight.toISOString(), targetDate: end.toISOString(),
    fontSize, fontWeight, color,
  };
}

/** Faces for `now`. `seconds: false` (WidgetKit timeline entries) drops the second hand,
 *  which WidgetKit cannot redraw every second. */
function clockLayouts(now: Date, seconds: boolean): Pick<WidgetConfig, "small" | "medium" | "large"> {
  const h = now.getHours() % 12;
  const m = now.getMinutes();
  // Sweeping second hand: milliseconds too (the preset ticks every 100 ms).
  const sf = now.getSeconds() + now.getMilliseconds() / 1000;
  const hAngle = ((h + m / 60) * 30 - 90) * Math.PI / 180;
  const mAngle = ((m + (seconds ? sf : 0) / 60) * 6 - 90) * Math.PI / 180;
  const sAngle = seconds ? (sf * 6 - 90) * Math.PI / 180 : NaN;
  // Logical design sizes — renderer fits them into the family with uniform scale.
  // Keep under typical content areas: small≈130, medium height≈140, large half≈150.
  const smallSize = 120;
  const mediumSize = 128;
  const largeSize = 148;
  const dateStr = now.toLocaleDateString("en-US", { weekday: "long", month: "short", day: "numeric" });
  const tzOffset = now.getTimezoneOffset();
  const tz = "UTC" + (tzOffset <= 0 ? "+" : "-") + Math.abs(tzOffset / 60);

  const small: WidgetElement = {
    type: "vstack", spacing: 6, padding: 12, cornerRadius: 16, alignment: "center",
    background: { gradientType: "linear", colors: ["#0f172a", "#1e293b"], direction: "topToBottom" },
    children: [
      clockCanvas(smallSize, hAngle, mAngle, sAngle),
      liveTime(now, 13, "semibold", "#94a3b8"),
    ],
  };

  const medium: WidgetElement = {
    type: "hstack", spacing: 12, padding: 14, cornerRadius: 16, alignment: "center",
    background: { gradientType: "linear", colors: ["#0f172a", "#1e293b"], direction: "leadingToTrailing" },
    children: [
      clockCanvas(mediumSize, hAngle, mAngle, sAngle),
      { type: "vstack", spacing: 6, alignment: "leading", flex: 1, children: [
        liveTime(now, 22, "bold", "#f1f5f9"),
        { type: "text", content: dateStr, fontSize: 12, color: "#94a3b8", lineLimit: 1 },
        { type: "spacer" },
        { type: "label", text: tz, systemName: "globe", fontSize: 11, color: "#64748b" },
      ] },
    ],
  };

  const large: WidgetElement = {
    type: "vstack", spacing: 8, padding: 14, cornerRadius: 16, alignment: "center",
    background: { gradientType: "linear", colors: ["#0f172a", "#1e293b", "#0f172a"], direction: "topToBottom" },
    children: [
      { type: "text", content: "World Clock", fontSize: 15, fontWeight: "bold", color: "#f1f5f9", alignment: "center" },
      { type: "hstack", spacing: 10, alignment: "center", children: [
        { type: "vstack", spacing: 4, alignment: "center", flex: 1, children: [
          clockCanvas(largeSize, hAngle, mAngle, sAngle),
          { type: "text", content: "Local", fontSize: 11, color: "#94a3b8", alignment: "center" },
        ] },
        { type: "vstack", spacing: 4, alignment: "center", flex: 1, children: [
          // Tokyo ≈ UTC+9 — show same face for demo (hands already live via builder tick).
          clockCanvas(largeSize, hAngle, mAngle, sAngle),
          { type: "text", content: "Tokyo", fontSize: 11, color: "#94a3b8", alignment: "center" },
        ] },
      ] },
      { type: "text", content: now.toLocaleDateString("en-US", { weekday: "short", month: "short", day: "numeric" }), fontSize: 12, color: "#64748b", alignment: "center" },
    ],
  };

  return { small, medium, large };
}

/** Now with a live second hand (desktop window: 100 ms ticks) plus one timeline entry per
 *  minute for the next 15 minutes (WidgetKit asks again after the last one), so WidgetKit moves the hands without reloads. */
function buildClockConfig(): WidgetConfig {
  const now = new Date();
  const next = new Date(now);
  next.setSeconds(0, 0);
  const entries = Array.from({ length: 15 }, (_, i) => {
    const at = new Date(next.getTime() + (i + 1) * 60_000);
    return { at: at.getTime(), ...clockLayouts(at, false) };
  });
  const templates = Object.fromEntries(
    SIZES.map((sz) => [`dial${sz}`, { type: "canvas", width: sz, height: sz, flex: 1, elements: dial(sz) } as WidgetElement]),
  );
  // No second hand anywhere: WidgetKit cannot redraw it every second, so it would stand
  // still; the digital time (a device-driven timer) shows the seconds instead.
  return { ...clockLayouts(now, false), entries, templates };
}

export const analogClock: PresetDef = {
  icon: "\u{1F570}",
  name: "Analog Clock",
  config: buildClockConfig(),
  builder: buildClockConfig,
  intervalMs: 100,
};
