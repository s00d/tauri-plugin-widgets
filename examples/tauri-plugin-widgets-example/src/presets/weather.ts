import type { PresetDef } from "./types";

export const weather: PresetDef = {
  icon: "\u{1F324}",
  name: "Weather",
  onAction(action, _payload, log) {
    if (action === "refresh_weather") log("Weather data refreshed");
  },
  config: {
    small: {
      type: "vstack", padding: 14, spacing: 6, cornerRadius: 16,
      background: { light: "#E8F4FD", dark: "#1a1a2e" },
      children: [
        { type: "hstack", spacing: 8, children: [
          { type: "image", systemName: "cloud.sun.fill", color: "#ffcc00", size: 28 },
          { type: "text", content: "72\u00B0", textStyle: "largeTitle", fontWeight: "bold", color: { light: "#1a1a2e", dark: "#ffffff" } },
        ]},
        { type: "text", content: "Partly Cloudy", textStyle: "footnote", color: "secondaryLabel" },
        { type: "text", content: "San Francisco", textStyle: "caption", color: "secondaryLabel" },
      ],
    },
    medium: {
      type: "hstack", padding: 16, spacing: 14, cornerRadius: 16,
      background: { light: "#E8F4FD", dark: "#1a1a2e" },
      children: [
        { type: "vstack", spacing: 4, alignment: "center", flex: 1, children: [
          { type: "image", systemName: "cloud.sun.fill", color: "#ffcc00", size: 40 },
          { type: "text", content: "72\u00B0", fontSize: 36, fontWeight: "bold", color: { light: "#1a1a2e", dark: "#ffffff" } },
          { type: "text", content: "Partly Cloudy", textStyle: "footnote", color: "secondaryLabel" },
        ]},
        { type: "divider", color: "separator" },
        { type: "hstack", spacing: 10, alignment: "bottom", flex: 1, children: [
          { type: "vstack", alignment: "center", spacing: 2, children: [
            { type: "text", content: "Mon", textStyle: "caption2", color: "secondaryLabel" },
            { type: "image", systemName: "sun.max.fill", color: "#ffcc00", size: 16 },
            { type: "text", content: "75\u00B0", textStyle: "footnote", color: "label" },
          ]},
          { type: "vstack", alignment: "center", spacing: 2, children: [
            { type: "text", content: "Tue", textStyle: "caption2", color: "secondaryLabel" },
            { type: "image", systemName: "cloud.rain.fill", color: "#4fc3f7", size: 16 },
            { type: "text", content: "68\u00B0", textStyle: "footnote", color: "label" },
          ]},
          { type: "vstack", alignment: "center", spacing: 2, children: [
            { type: "text", content: "Wed", textStyle: "caption2", color: "secondaryLabel" },
            { type: "image", systemName: "cloud.fill", color: "#90a4ae", size: 16 },
            { type: "text", content: "65\u00B0", textStyle: "footnote", color: "label" },
          ]},
        ]},
      ],
    },
    large: {
      type: "vstack", padding: 16, spacing: 8, cornerRadius: 16,
      background: { light: "#E8F4FD", dark: "#1a1a2e" },
      children: [
        { type: "hstack", spacing: 12, children: [
          { type: "image", systemName: "cloud.sun.fill", color: "#ffcc00", size: 48 },
          { type: "vstack", spacing: 2, alignment: "leading", flex: 1, children: [
            { type: "text", content: "72\u00B0", fontSize: 42, fontWeight: "bold", color: { light: "#1a1a2e", dark: "#ffffff" } },
            { type: "text", content: "Partly Cloudy", textStyle: "subheadline", color: "secondaryLabel" },
            { type: "text", content: "San Francisco, CA", textStyle: "caption", color: "secondaryLabel" },
          ]},
        ]},
        { type: "divider", color: "separator" },
        { type: "hstack", spacing: 8, children: [
          { type: "vstack", alignment: "center", spacing: 2, flex: 1, children: [
            { type: "text", content: "Mon", textStyle: "caption2", color: "secondaryLabel" },
            { type: "image", systemName: "sun.max.fill", color: "#ffcc00", size: 18 },
            { type: "text", content: "75\u00B0/58\u00B0", textStyle: "caption", color: "label", lineLimit: 1 },
          ]},
          { type: "vstack", alignment: "center", spacing: 2, flex: 1, children: [
            { type: "text", content: "Tue", textStyle: "caption2", color: "secondaryLabel" },
            { type: "image", systemName: "cloud.rain.fill", color: "#4fc3f7", size: 18 },
            { type: "text", content: "68\u00B0/55\u00B0", textStyle: "caption", color: "label", lineLimit: 1 },
          ]},
          { type: "vstack", alignment: "center", spacing: 2, flex: 1, children: [
            { type: "text", content: "Wed", textStyle: "caption2", color: "secondaryLabel" },
            { type: "image", systemName: "cloud.fill", color: "#90a4ae", size: 18 },
            { type: "text", content: "65\u00B0/52\u00B0", textStyle: "caption", color: "label", lineLimit: 1 },
          ]},
          { type: "vstack", alignment: "center", spacing: 2, flex: 1, children: [
            { type: "text", content: "Thu", textStyle: "caption2", color: "secondaryLabel" },
            { type: "image", systemName: "sun.max.fill", color: "#ffcc00", size: 18 },
            { type: "text", content: "74\u00B0/60\u00B0", textStyle: "caption", color: "label", lineLimit: 1 },
          ]},
          { type: "vstack", alignment: "center", spacing: 2, flex: 1, children: [
            { type: "text", content: "Fri", textStyle: "caption2", color: "secondaryLabel" },
            { type: "image", systemName: "cloud.sun.fill", color: "#ffcc00", size: 18 },
            { type: "text", content: "70\u00B0/56\u00B0", textStyle: "caption", color: "label", lineLimit: 1 },
          ]},
        ]},
        { type: "divider", color: "separator" },
        { type: "hstack", spacing: 12, children: [
          { type: "vstack", spacing: 2, children: [
            { type: "text", content: "Humidity", textStyle: "caption", color: "secondaryLabel" },
            { type: "text", content: "62%", fontSize: 16, fontWeight: "semibold", color: "#4fc3f7" },
          ]},
          { type: "vstack", spacing: 2, children: [
            { type: "text", content: "Wind", textStyle: "caption", color: "secondaryLabel" },
            { type: "text", content: "12 mph", fontSize: 16, fontWeight: "semibold", color: "#a5b4fc" },
          ]},
          { type: "vstack", spacing: 2, children: [
            { type: "text", content: "UV Index", textStyle: "caption", color: "secondaryLabel" },
            { type: "text", content: "6", fontSize: 16, fontWeight: "semibold", color: "#fbbf24" },
          ]},
          { type: "spacer" },
          { type: "button", label: "Refresh", action: "refresh_weather", backgroundColor: { light: "#2196F3", dark: "#334155" }, color: "#ffffff", fontSize: 12, cornerRadius: 6 },
        ]},
      ],
    },
  },
};
