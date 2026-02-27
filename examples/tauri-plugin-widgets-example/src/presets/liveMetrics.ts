import type { WidgetConfig, WidgetElement, ChartDataPoint } from "tauri-plugin-widgets-api";
import type { PresetDef } from "./types";

interface ServerStatus {
  name: string;
  up: boolean;
}

function tierColor(pct: number): string {
  if (pct >= 85) return "#ef4444";
  if (pct >= 65) return "#f59e0b";
  return "#22c55e";
}

function tierLabel(pct: number): string {
  if (pct >= 85) return "CRITICAL";
  if (pct >= 65) return "WARNING";
  return "NORMAL";
}

function wave(t: number, freq: number, offset: number, min: number, max: number): number {
  const raw = Math.sin(t * freq + offset) * 0.5 + 0.5;
  const jitter = Math.sin(t * freq * 7.3 + offset * 3.1) * 0.08;
  return Math.min(max, Math.max(min, min + (max - min) * (raw + jitter)));
}

function buildChartData(t: number, freq: number, offset: number, min: number, max: number): ChartDataPoint[] {
  const labels = ["-6s", "-5s", "-4s", "-3s", "-2s", "-1s", "now"];
  return labels.map((label, i) => ({
    label,
    value: Math.round(wave(t - (6 - i), freq, offset, min, max)),
  }));
}

function serverDot(s: ServerStatus): WidgetElement {
  return {
    type: "hstack", spacing: 3, children: [
      { type: "shape", shapeType: "circle", fill: s.up ? "#22c55e" : "#ef4444", size: 6 },
      { type: "text", content: s.name, textStyle: "caption2", color: s.up ? "secondaryLabel" : "#ef4444" },
    ],
  };
}

function serverCard(s: ServerStatus): WidgetElement {
  return {
    type: "vstack", spacing: 3, alignment: "center", padding: 8,
    background: s.up
      ? { light: "#DCFCE7", dark: "#132f1e" }
      : { light: "#FEE2E2", dark: "#3b1111" },
    cornerRadius: 8,
    border: { color: s.up ? "#22c55e" : "#ef4444", width: 1 },
    children: [
      { type: "shape", shapeType: "circle", fill: s.up ? "#22c55e" : "#ef4444", size: 10 },
      { type: "text", content: s.name, textStyle: "caption2", fontWeight: "semibold", color: s.up ? "#86efac" : "#fca5a5" },
      { type: "text", content: s.up ? "ONLINE" : "DOWN", fontSize: 8, fontWeight: "bold", color: s.up ? "#22c55e" : "#ef4444" },
    ],
  };
}

function buildLiveMetrics(): WidgetConfig {
  const t = Date.now() / 1000;
  const cpu = Math.round(wave(t, 0.3, 0, 15, 95));
  const ram = Math.round(wave(t, 0.15, 2, 40, 88));
  const disk = Math.round(wave(t, 0.05, 5, 55, 78));
  const net = Math.round(wave(t, 0.8, 1, 2, 48));

  const cpuColor = tierColor(cpu);
  const ramColor = tierColor(ram);
  const diskColor = tierColor(disk);

  const servers: ServerStatus[] = [
    { name: "web-01", up: wave(t, 0.1, 0, 0, 1) > 0.15 },
    { name: "web-02", up: wave(t, 0.08, 3, 0, 1) > 0.1 },
    { name: "db-01", up: wave(t, 0.12, 6, 0, 1) > 0.2 },
    { name: "cache", up: wave(t, 0.07, 9, 0, 1) > 0.12 },
  ];

  const overallStatus = cpu >= 85 || ram >= 85 ? "CRITICAL" : cpu >= 65 || ram >= 65 ? "WARNING" : "HEALTHY";
  const overallColor = cpu >= 85 || ram >= 85 ? "#ef4444" : cpu >= 65 || ram >= 65 ? "#f59e0b" : "#22c55e";

  const small: WidgetElement = {
    type: "vstack", padding: 12, spacing: 6, cornerRadius: 16,
    background: { light: "#F0F4FF", dark: "#0f172a" },
    children: [
      { type: "hstack", spacing: 6, children: [
        { type: "shape", shapeType: "circle", fill: overallColor, size: 8 },
        { type: "text", content: overallStatus, textStyle: "caption2", fontWeight: "bold", color: overallColor },
      ] },
      { type: "gauge", value: cpu / 100, min: 0, max: 1, currentValueLabel: `${cpu}%`, label: "CPU", gaugeStyle: "circular", tint: cpuColor },
      { type: "hstack", spacing: 8, children: [
        { type: "vstack", spacing: 1, alignment: "center", children: [
          { type: "text", content: `${ram}%`, fontSize: 13, fontWeight: "bold", color: ramColor },
          { type: "text", content: "RAM", textStyle: "caption2", color: "secondaryLabel" },
        ] },
        { type: "vstack", spacing: 1, alignment: "center", children: [
          { type: "text", content: `${net}`, fontSize: 13, fontWeight: "bold", color: "#38bdf8" },
          { type: "text", content: "MB/s", textStyle: "caption2", color: "secondaryLabel" },
        ] },
      ] },
    ],
  };

  const medium: WidgetElement = {
    type: "hstack", padding: 14, spacing: 14, cornerRadius: 16,
    background: { light: "#F0F4FF", dark: "#0f172a" },
    children: [
      { type: "vstack", spacing: 6, alignment: "center", children: [
        { type: "hstack", spacing: 6, children: [
          { type: "shape", shapeType: "circle", fill: overallColor, size: 8 },
          { type: "text", content: overallStatus, textStyle: "caption2", fontWeight: "bold", color: overallColor },
        ] },
        { type: "gauge", value: cpu / 100, min: 0, max: 1, currentValueLabel: `${cpu}%`, label: "CPU", gaugeStyle: "circular", tint: cpuColor },
      ] },
      { type: "divider", color: "separator" },
      { type: "vstack", spacing: 5, alignment: "leading", flex: 1, children: [
        { type: "progress", value: ram / 100, tint: ramColor, label: `RAM: ${ram}%`, color: "secondaryLabel" },
        { type: "progress", value: disk / 100, tint: diskColor, label: `Disk: ${disk}%`, color: "secondaryLabel" },
        { type: "hstack", spacing: 4, children: [
          { type: "text", content: `Network: ${net} MB/s`, textStyle: "caption2", color: "#38bdf8" },
        ] },
        { type: "hstack", spacing: 6, children: servers.map(serverDot) },
      ] },
    ],
  };

  const large: WidgetElement = {
    type: "vstack", padding: 16, spacing: 10, cornerRadius: 16,
    background: { light: "#F0F4FF", dark: "#0f172a" },
    children: [
      { type: "hstack", spacing: 8, children: [
        { type: "shape", shapeType: "circle", fill: overallColor, size: 10 },
        { type: "text", content: "Live Metrics", textStyle: "headline", color: "label" },
        { type: "spacer" },
        { type: "text", content: overallStatus, textStyle: "caption", fontWeight: "bold", color: overallColor },
      ] },
      { type: "divider", color: "separator" },
      { type: "hstack", spacing: 12, children: [
        { type: "gauge", value: cpu / 100, min: 0, max: 1, currentValueLabel: `${cpu}%`, label: "CPU", gaugeStyle: "circular", tint: cpuColor, flex: 1 },
        { type: "gauge", value: ram / 100, min: 0, max: 1, currentValueLabel: `${ram}%`, label: "RAM", gaugeStyle: "circular", tint: ramColor, flex: 1 },
        { type: "gauge", value: disk / 100, min: 0, max: 1, currentValueLabel: `${disk}%`, label: "Disk", gaugeStyle: "circular", tint: diskColor, flex: 1 },
        { type: "vstack", spacing: 2, alignment: "center", flex: 1, children: [
          { type: "text", content: `${net}`, fontSize: 24, fontWeight: "bold", color: "#38bdf8" },
          { type: "text", content: "MB/s", textStyle: "caption2", color: "secondaryLabel" },
        ] },
      ] },
      { type: "divider", color: "separator" },
      { type: "text", content: "CPU History", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
      { type: "chart", chartType: "area", tint: cpuColor, chartData: buildChartData(t, 0.3, 0, 15, 95) },
      { type: "text", content: "Servers", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
      { type: "grid", columns: 4, spacing: 8, rowSpacing: 8, children: servers.map(serverCard) },
      { type: "hstack", spacing: 8, children: [
        { type: "text", content: `CPU: ${tierLabel(cpu)}`, textStyle: "caption2", fontWeight: "bold", color: cpuColor },
        { type: "text", content: `RAM: ${tierLabel(ram)}`, textStyle: "caption2", fontWeight: "bold", color: ramColor },
        { type: "text", content: `Disk: ${tierLabel(disk)}`, textStyle: "caption2", fontWeight: "bold", color: diskColor },
      ] },
    ],
  };

  return { small, medium, large };
}

export const liveMetrics: PresetDef = {
  icon: "\u{1F4C8}",
  name: "Live Metrics",
  config: buildLiveMetrics(),
  builder: buildLiveMetrics,
  intervalMs: 1000,
};
