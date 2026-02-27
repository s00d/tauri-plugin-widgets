import type { PresetDef } from "./types";

export const battery: PresetDef = {
  icon: "\u{1F50B}",
  name: "Battery",
  onAction(action, _payload, log) {
    if (action === "refresh_battery") log("Battery status refreshed");
  },
  config: {
    small: {
      type: "vstack", padding: 14, spacing: 8, cornerRadius: 16,
      background: { light: "#F0F0FF", dark: "#0c0c1d" },
      shadow: { color: "#6366f1", radius: 12, x: 0, y: 0 },
      children: [
        { type: "label", text: "Battery", systemName: "battery.100", iconColor: "#22c55e",
          fontSize: 12, fontWeight: "semibold", color: "secondaryLabel" },
        { type: "gauge", value: 0.85, min: 0, max: 1, currentValueLabel: "85%", label: "", gaugeStyle: "circular", tint: "#22c55e" },
        { type: "hstack", spacing: 12, children: [
          { type: "label", text: "85%", systemName: "iphone", iconColor: "#a78bfa", fontSize: 11, color: "secondaryLabel" },
          { type: "label", text: "62%", systemName: "applewatch", iconColor: "#34d399", fontSize: 11, color: "secondaryLabel" },
        ]},
      ],
    },
    medium: {
      type: "hstack", padding: 14, spacing: 14, cornerRadius: 16,
      background: { light: "#F0F0FF", dark: "#0c0c1d" },
      children: [
        { type: "gauge", value: 0.85, min: 0, max: 1, currentValueLabel: "85%", label: "iPhone", gaugeStyle: "circular", tint: "#22c55e" },
        { type: "gauge", value: 0.62, min: 0, max: 1, currentValueLabel: "62%", label: "Watch", gaugeStyle: "circular", tint: "#34d399" },
        { type: "gauge", value: 0.41, min: 0, max: 1, currentValueLabel: "41%", label: "AirPods", gaugeStyle: "circular", tint: "#fbbf24" },
        { type: "divider", color: "separator" },
        { type: "vstack", spacing: 4, alignment: "leading", flex: 1, children: [
          { type: "text", content: "Est. Remaining", textStyle: "caption", color: "secondaryLabel" },
          { type: "text", content: "iPhone: 6h 30m", textStyle: "footnote", color: "#a78bfa" },
          { type: "text", content: "Watch: 8h 15m", textStyle: "footnote", color: "#34d399" },
          { type: "text", content: "AirPods: 2h 45m", textStyle: "footnote", color: "#fbbf24" },
        ]},
      ],
    },
    large: {
      type: "vstack", padding: 16, spacing: 10, cornerRadius: 16,
      background: { light: "#F0F0FF", dark: "#0c0c1d" },
      children: [
        { type: "hstack", spacing: 8, children: [
          { type: "image", systemName: "battery.100", color: "#22c55e", size: 20 },
          { type: "text", content: "Battery Status", textStyle: "headline", color: "label" },
          { type: "spacer" },
          { type: "button", label: "Refresh", action: "refresh_battery",
            backgroundColor: "#6366f1", color: "#ffffff", fontSize: 11, cornerRadius: 6 },
        ]},
        { type: "divider", color: "separator" },
        { type: "hstack", spacing: 12, children: [
          { type: "gauge", value: 0.85, min: 0, max: 1, currentValueLabel: "85%", label: "iPhone", gaugeStyle: "circular", tint: "#22c55e", flex: 1 },
          { type: "gauge", value: 0.62, min: 0, max: 1, currentValueLabel: "62%", label: "Watch", gaugeStyle: "circular", tint: "#34d399", flex: 1 },
          { type: "gauge", value: 0.41, min: 0, max: 1, currentValueLabel: "41%", label: "AirPods", gaugeStyle: "circular", tint: "#fbbf24", flex: 1 },
          { type: "gauge", value: 0.93, min: 0, max: 1, currentValueLabel: "93%", label: "iPad", gaugeStyle: "circular", tint: "#a78bfa", flex: 1 },
        ]},
        { type: "divider", color: "separator" },
        { type: "grid", columns: 2, spacing: 8, rowSpacing: 6, children: [
          { type: "hstack", spacing: 6, children: [
            { type: "image", systemName: "iphone", color: "#a78bfa", size: 14 },
            { type: "text", content: "iPhone 15 Pro", textStyle: "footnote", color: "#a78bfa" },
          ]},
          { type: "text", content: "6h 30m remaining", textStyle: "footnote", color: "secondaryLabel" },
          { type: "hstack", spacing: 6, children: [
            { type: "image", systemName: "applewatch", color: "#34d399", size: 14 },
            { type: "text", content: "Apple Watch", textStyle: "footnote", color: "#34d399" },
          ]},
          { type: "text", content: "8h 15m remaining", textStyle: "footnote", color: "secondaryLabel" },
          { type: "hstack", spacing: 6, children: [
            { type: "image", systemName: "airpodspro", color: "#fbbf24", size: 14 },
            { type: "text", content: "AirPods Pro", textStyle: "footnote", color: "#fbbf24" },
          ]},
          { type: "text", content: "2h 45m remaining", textStyle: "footnote", color: "secondaryLabel" },
          { type: "hstack", spacing: 6, children: [
            { type: "image", systemName: "ipad", color: "#a78bfa", size: 14 },
            { type: "text", content: "iPad Air", textStyle: "footnote", color: "#a78bfa" },
          ]},
          { type: "text", content: "10h 20m remaining", textStyle: "footnote", color: "secondaryLabel" },
        ]},
      ],
    },
  },
};
