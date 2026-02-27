import type { PresetDef } from "./types";

export const fitness: PresetDef = {
  icon: "\u{1F3C3}",
  name: "Fitness",
  onAction(action, _payload, log) {
    if (action === "sync_fitness") log("Fitness data synced successfully");
  },
  config: {
    small: {
      type: "vstack", padding: 12, spacing: 8, cornerRadius: 16,
      background: { light: "#FFF5F5", dark: "#0d1117" },
      children: [
        { type: "text", content: "Activity", textStyle: "subheadline", fontWeight: "semibold", color: "label" },
        { type: "gauge", value: 0.72, min: 0, max: 1, currentValueLabel: "72%", label: "Steps", gaugeStyle: "circular", tint: "#ff6b6b" },
        { type: "hstack", spacing: 8, children: [
          { type: "progress", value: 0.45, barStyle: "linear", tint: "#4ecdc4", label: "Calories", color: "secondaryLabel" },
          { type: "progress", value: 0.88, barStyle: "linear", tint: "#ffe66d", label: "Water", color: "secondaryLabel" },
        ]},
      ],
    },
    medium: {
      type: "hstack", padding: 14, spacing: 16, cornerRadius: 16,
      background: { light: "#FFF5F5", dark: "#0d1117" },
      children: [
        { type: "vstack", spacing: 6, alignment: "center", children: [
          { type: "gauge", value: 0.72, min: 0, max: 1, currentValueLabel: "72%", label: "Steps", gaugeStyle: "circular", tint: "#ff6b6b" },
          { type: "text", content: "7,200 / 10,000", textStyle: "caption2", color: "secondaryLabel" },
        ]},
        { type: "divider", color: "separator" },
        { type: "vstack", spacing: 6, flex: 1, children: [
          { type: "progress", value: 0.45, tint: "#4ecdc4", label: "Calories: 450 / 1000", color: "secondaryLabel" },
          { type: "progress", value: 0.88, tint: "#ffe66d", label: "Water: 7 / 8 glasses", color: "secondaryLabel" },
          { type: "progress", value: 0.33, tint: "#ff6b6b", label: "Exercise: 20 / 60 min", color: "secondaryLabel" },
        ]},
      ],
    },
    large: {
      type: "vstack", padding: 16, spacing: 10, cornerRadius: 16,
      background: { light: "#FFF5F5", dark: "#0d1117" },
      children: [
        { type: "hstack", spacing: 8, children: [
          { type: "image", systemName: "figure.run", color: "#ff6b6b", size: 24 },
          { type: "text", content: "Today's Activity", textStyle: "title3", fontWeight: "bold", color: "label" },
          { type: "spacer" },
          { type: "button", label: "Sync", action: "sync_fitness", backgroundColor: "#ff6b6b", color: "#ffffff", fontSize: 12, cornerRadius: 6 },
        ]},
        { type: "divider", color: "separator" },
        { type: "hstack", spacing: 12, children: [
          { type: "gauge", value: 0.72, min: 0, max: 1, currentValueLabel: "72%", label: "Steps", gaugeStyle: "circular", tint: "#ff6b6b", flex: 1 },
          { type: "gauge", value: 0.45, min: 0, max: 1, currentValueLabel: "45%", label: "Calories", gaugeStyle: "circular", tint: "#4ecdc4", flex: 1 },
          { type: "gauge", value: 0.88, min: 0, max: 1, currentValueLabel: "88%", label: "Water", gaugeStyle: "circular", tint: "#ffe66d", flex: 1 },
        ]},
        { type: "divider", color: "separator" },
        { type: "vstack", spacing: 6, children: [
          { type: "progress", value: 0.33, tint: "#a78bfa", label: "Exercise: 20 / 60 min", color: "secondaryLabel" },
          { type: "progress", value: 0.6, tint: "#34d399", label: "Standing: 6 / 10 hours", color: "secondaryLabel" },
        ]},
        { type: "chart", chartType: "area", tint: "#ff6b6b", chartData: [
          { label: "Mon", value: 8500 }, { label: "Tue", value: 6200 }, { label: "Wed", value: 9800 },
          { label: "Thu", value: 7100 }, { label: "Fri", value: 10200 }, { label: "Sat", value: 4300 },
          { label: "Sun", value: 7200 },
        ]},
      ],
    },
  },
};
