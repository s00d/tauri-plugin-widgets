import type { PresetDef } from "./types";

export const system: PresetDef = {
  icon: "\u{1F4BB}",
  name: "System",
  onAction(action, _payload, log) {
    if (action === "refresh_system") log("System metrics refreshed");
  },
  config: {
    small: {
      type: "vstack", padding: 12, spacing: 8, cornerRadius: 16,
      background: { light: "#F0F4FF", dark: "#1a1b26" },
      children: [
        { type: "text", content: "System Monitor", textStyle: "caption", fontWeight: "semibold", color: "#7aa2f7" },
        { type: "hstack", spacing: 12, children: [
          { type: "gauge", value: 0.45, min: 0, max: 1, currentValueLabel: "45%", label: "CPU", gaugeStyle: "circular", tint: "#7aa2f7" },
          { type: "gauge", value: 0.78, min: 0, max: 1, currentValueLabel: "78%", label: "RAM", gaugeStyle: "circular", tint: "#bb9af7" },
        ]},
      ],
    },
    medium: {
      type: "hstack", padding: 14, spacing: 14, cornerRadius: 16,
      background: { light: "#F0F4FF", dark: "#1a1b26" },
      children: [
        { type: "vstack", spacing: 8, alignment: "center", children: [
          { type: "gauge", value: 0.45, min: 0, max: 1, currentValueLabel: "45%", label: "CPU", gaugeStyle: "circular", tint: "#7aa2f7" },
          { type: "gauge", value: 0.78, min: 0, max: 1, currentValueLabel: "78%", label: "RAM", gaugeStyle: "circular", tint: "#bb9af7" },
        ]},
        { type: "divider", color: "separator" },
        { type: "vstack", spacing: 4, alignment: "leading", flex: 1, children: [
          { type: "hstack", spacing: 4, children: [
            { type: "image", systemName: "internaldrive", color: "#7aa2f7", size: 12 },
            { type: "text", content: "Disk: 234 GB / 512 GB", textStyle: "caption2", color: "secondaryLabel" },
          ]},
          { type: "progress", value: 0.46, tint: "#7aa2f7", color: "secondaryLabel" },
          { type: "hstack", spacing: 4, children: [
            { type: "image", systemName: "network", color: "#bb9af7", size: 12 },
            { type: "text", content: "Network: 12.4 MB/s", textStyle: "caption2", color: "secondaryLabel" },
          ]},
          { type: "hstack", spacing: 4, children: [
            { type: "image", systemName: "clock", color: { light: "#6B7280", dark: "#7a83a0" }, size: 12 },
            { type: "text", content: "Uptime: 4d 12h", textStyle: "caption2", color: "secondaryLabel" },
          ]},
        ]},
      ],
    },
    large: {
      type: "vstack", padding: 16, spacing: 10, cornerRadius: 16,
      background: { light: "#F0F4FF", dark: "#1a1b26" },
      children: [
        { type: "hstack", spacing: 8, children: [
          { type: "image", systemName: "desktopcomputer", color: "#7aa2f7", size: 20 },
          { type: "text", content: "System Monitor", textStyle: "headline", color: "label" },
          { type: "spacer" },
          { type: "button", label: "Refresh", action: "refresh_system",
            backgroundColor: "#7aa2f7", color: { light: "#F0F4FF", dark: "#1a1b26" }, fontSize: 11, cornerRadius: 6 },
        ]},
        { type: "divider", color: "separator" },
        { type: "hstack", spacing: 12, children: [
          { type: "gauge", value: 0.45, min: 0, max: 1, currentValueLabel: "45%", label: "CPU", gaugeStyle: "circular", tint: "#7aa2f7", flex: 1 },
          { type: "gauge", value: 0.78, min: 0, max: 1, currentValueLabel: "78%", label: "RAM", gaugeStyle: "circular", tint: "#bb9af7", flex: 1 },
          { type: "gauge", value: 0.46, min: 0, max: 1, currentValueLabel: "46%", label: "Disk", gaugeStyle: "circular", tint: "#7dcfff", flex: 1 },
          { type: "gauge", value: 0.62, min: 0, max: 1, currentValueLabel: "62\u00B0C", label: "Temp", gaugeStyle: "circular", tint: "#e0af68", flex: 1 },
        ]},
        { type: "divider", color: "separator" },
        { type: "text", content: "CPU Usage (24h)", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
        { type: "chart", chartType: "line", tint: "#7aa2f7", chartData: [
          { label: "0h", value: 22 }, { label: "4h", value: 35 }, { label: "8h", value: 68 },
          { label: "12h", value: 45 }, { label: "16h", value: 72 }, { label: "20h", value: 51 },
          { label: "24h", value: 45 },
        ]},
        { type: "text", content: "Processes", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
        { type: "grid", columns: 2, spacing: 6, rowSpacing: 4, children: [
          { type: "text", content: "node: 12.3%", textStyle: "caption2", color: "#7aa2f7" },
          { type: "text", content: "chrome: 8.1%", textStyle: "caption2", color: "#bb9af7" },
          { type: "text", content: "rust-analyzer: 5.4%", textStyle: "caption2", color: "#7dcfff" },
          { type: "text", content: "Finder: 1.2%", textStyle: "caption2", color: "secondaryLabel" },
        ]},
        { type: "hstack", spacing: 4, children: [
          { type: "image", systemName: "clock", color: { light: "#6B7280", dark: "#7a83a0" }, size: 12 },
          { type: "text", content: "Uptime: 4d 12h 33m", textStyle: "caption2", color: "secondaryLabel" },
        ]},
      ],
    },
  },
};
