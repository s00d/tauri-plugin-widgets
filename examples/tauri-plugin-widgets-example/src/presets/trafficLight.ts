import type { WidgetConfig, WidgetElement, CanvasDrawCommand } from "tauri-plugin-widgets-api";
import type { PresetDef } from "./types";

type Phase = "red" | "yellow" | "green";

const CYCLE: { phase: Phase; dur: number }[] = [
  { phase: "red", dur: 3 },
  { phase: "yellow", dur: 1 },
  { phase: "green", dur: 3 },
  { phase: "yellow", dur: 1 },
];
const TOTAL = CYCLE.reduce((s, c) => s + c.dur, 0);

const BRIGHT: Record<Phase, string> = { red: "#ef4444", yellow: "#facc15", green: "#22c55e" };
const DIM: Record<Phase, string> = { red: "#3b1111", yellow: "#3b3511", green: "#0b3b1a" };

function getPhase(): { phase: Phase; elapsed: number; remaining: number } {
  const t = Math.floor(Date.now() / 1000) % TOTAL;
  let acc = 0;
  for (const c of CYCLE) {
    if (t < acc + c.dur) return { phase: c.phase, elapsed: t - acc, remaining: c.dur - (t - acc) };
    acc += c.dur;
  }
  return { phase: "red", elapsed: 0, remaining: 3 };
}

function light(phase: Phase, active: Phase): WidgetElement {
  const on = phase === active;
  const commands: CanvasDrawCommand[] = [
    { draw: "circle", cx: 24, cy: 24, r: 22, fill: on ? BRIGHT[phase] : DIM[phase], stroke: "#52525b", strokeWidth: 2 },
    ...(on ? [{ draw: "circle" as const, cx: 24, cy: 24, r: 16, fill: BRIGHT[phase], stroke: "none", strokeWidth: 0 }] : []),
  ];
  return { type: "canvas", width: 48, height: 48, elements: commands };
}

function signalCard(phase: Phase, currentPhase: Phase, label: string, dur: string): WidgetElement {
  const active = phase === currentPhase;
  return {
    type: "vstack", spacing: 4, alignment: "center", padding: 10, cornerRadius: 10,
    background: active ? DIM[phase] : "#1c1c1e",
    border: active ? { color: BRIGHT[phase], width: 2 } : undefined,
    children: [
      { type: "shape", shapeType: "circle", fill: active ? BRIGHT[phase] : DIM[phase], size: 20 },
      { type: "text", content: label, fontSize: 11, fontWeight: "bold", color: active ? BRIGHT[phase] : "#52525b" },
      { type: "text", content: dur, fontSize: 10, color: "#71717a" },
    ],
  };
}

function buildTrafficLight(): WidgetConfig {
  const { phase, remaining } = getPhase();
  const label = phase === "red" ? "STOP" : phase === "yellow" ? "CAUTION" : "GO";
  const color = BRIGHT[phase];
  const progress = remaining / (phase === "yellow" ? 1 : 3);

  const lightsV: WidgetElement = {
    type: "vstack", spacing: 6, alignment: "center",
    padding: 10, background: "#27272a", cornerRadius: 12, children: [
      light("red", phase), light("yellow", phase), light("green", phase),
    ],
  };

  const lightsH: WidgetElement = {
    type: "hstack", spacing: 8, alignment: "center",
    padding: 10, background: "#27272a", cornerRadius: 12, children: [
      light("red", phase), light("yellow", phase), light("green", phase),
    ],
  };

  const small: WidgetElement = {
    type: "vstack", padding: 12, spacing: 8, cornerRadius: 16,
    background: { gradientType: "linear", colors: ["#18181b", "#27272a"], direction: "topToBottom" },
    children: [
      { type: "text", content: "Traffic Light", fontSize: 11, fontWeight: "semibold", color: "#a1a1aa", alignment: "center" },
      lightsV,
      { type: "text", content: label, fontSize: 18, fontWeight: "bold", color, alignment: "center" },
    ],
  };

  const medium: WidgetElement = {
    type: "hstack", padding: 14, spacing: 16, cornerRadius: 16,
    background: { gradientType: "linear", colors: ["#18181b", "#27272a"], direction: "leadingToTrailing" },
    children: [
      lightsV,
      { type: "vstack", spacing: 6, alignment: "leading", children: [
        { type: "text", content: "Traffic Light", fontSize: 14, fontWeight: "bold", color: "#e4e4e7" },
        { type: "text", content: label, fontSize: 28, fontWeight: "bold", color },
        { type: "progress", value: progress, tint: color, label: `${remaining}s remaining`, color: "#a1a1aa" },
        { type: "spacer" },
        { type: "hstack", spacing: 8, children: [
          { type: "shape", shapeType: "circle", fill: phase === "red" ? BRIGHT.red : DIM.red, size: 10 },
          { type: "shape", shapeType: "circle", fill: phase === "yellow" ? BRIGHT.yellow : DIM.yellow, size: 10 },
          { type: "shape", shapeType: "circle", fill: phase === "green" ? BRIGHT.green : DIM.green, size: 10 },
          { type: "text", content: `Cycle: ${TOTAL}s`, fontSize: 11, color: "#71717a" },
        ] },
      ] },
    ],
  };

  const large: WidgetElement = {
    type: "vstack", padding: 16, spacing: 12, cornerRadius: 16,
    background: { gradientType: "linear", colors: ["#18181b", "#27272a"], direction: "topToBottom" },
    children: [
      { type: "hstack", spacing: 8, children: [
        { type: "text", content: "Traffic Light Simulator", fontSize: 16, fontWeight: "bold", color: "#e4e4e7" },
        { type: "spacer" },
        { type: "text", content: `Cycle: ${TOTAL}s`, fontSize: 12, color: "#71717a" },
      ] },
      { type: "divider", color: "#3f3f46" },
      { type: "hstack", spacing: 20, children: [
        lightsV,
        { type: "vstack", spacing: 8, alignment: "leading", children: [
          { type: "text", content: label, fontSize: 36, fontWeight: "bold", color },
          { type: "text", content: `Signal: ${phase.toUpperCase()}`, fontSize: 13, color: "#a1a1aa" },
          { type: "progress", value: progress, tint: color, label: `Next change in ${remaining}s`, color: "#a1a1aa" },
        ] },
      ] },
      { type: "divider", color: "#3f3f46" },
      { type: "text", content: "Signal States", fontSize: 12, fontWeight: "semibold", color: "#a1a1aa" },
      { type: "grid", columns: 3, spacing: 12, rowSpacing: 8, children: [
        signalCard("red", phase, "STOP", "3s"),
        signalCard("yellow", phase, "CAUTION", "1s"),
        signalCard("green", phase, "GO", "3s"),
      ] },
      lightsH,
    ],
  };

  return { small, medium, large };
}

export const trafficLight: PresetDef = {
  icon: "\u{1F6A6}",
  name: "Traffic Light",
  config: buildTrafficLight(),
  builder: buildTrafficLight,
  intervalMs: 1000,
};
