import type { PresetDef } from "./types";

export const crypto: PresetDef = {
  icon: "\u20BF",
  name: "Crypto",
  onAction(action, _payload, log) {
    if (action === "refresh_crypto") log("Crypto prices refreshed");
  },
  config: {
    small: {
      type: "vstack", padding: 12, spacing: 6, cornerRadius: 16,
      background: { light: "#F5F3FF", dark: "#0f0f23" },
      children: [
        { type: "hstack", spacing: 6, children: [
          { type: "image", systemName: "bitcoinsign.circle.fill", color: "#f7931a", size: 20 },
          { type: "text", content: "Bitcoin", textStyle: "subheadline", fontWeight: "semibold", color: "label" },
        ]},
        { type: "text", content: "$67,432", fontSize: 28, fontWeight: "bold", color: "label" },
        { type: "text", content: "+2.34%", textStyle: "subheadline", fontWeight: "medium", color: "#00ff88" },
      ],
    },
    medium: {
      type: "hstack", padding: 14, spacing: 16, cornerRadius: 16,
      background: { light: "#F5F3FF", dark: "#0f0f23" },
      children: [
        { type: "vstack", spacing: 4, alignment: "leading", flex: 1, children: [
          { type: "hstack", spacing: 6, children: [
            { type: "image", systemName: "bitcoinsign.circle.fill", color: "#f7931a", size: 24 },
            { type: "text", content: "Bitcoin", textStyle: "headline", color: "label" },
          ]},
          { type: "text", content: "$67,432", fontSize: 32, fontWeight: "bold", color: "label" },
          { type: "text", content: "+2.34% today", textStyle: "footnote", color: "#00ff88" },
          { type: "spacer" },
          { type: "button", label: "Refresh", action: "refresh_crypto",
            backgroundColor: "#f7931a", color: "#000", fontSize: 11, cornerRadius: 6 },
        ]},
        { type: "chart", chartType: "line", tint: "#00ff88", chartData: [
          { label: "1", value: 64000 }, { label: "2", value: 65200 }, { label: "3", value: 63800 },
          { label: "4", value: 66100 }, { label: "5", value: 65900 }, { label: "6", value: 67432 },
        ]},
      ],
    },
    large: {
      type: "vstack", padding: 16, spacing: 10, cornerRadius: 16,
      background: { light: "#F5F3FF", dark: "#0f0f23" },
      children: [
        { type: "hstack", spacing: 8, children: [
          { type: "image", systemName: "bitcoinsign.circle.fill", color: "#f7931a", size: 28 },
          { type: "text", content: "Crypto Portfolio", textStyle: "title3", fontWeight: "bold", color: "label" },
          { type: "spacer" },
          { type: "button", label: "Refresh", action: "refresh_crypto",
            backgroundColor: "#f7931a", color: "#000", fontSize: 11, cornerRadius: 6 },
        ]},
        { type: "divider", color: "separator" },
        { type: "hstack", spacing: 8, children: [
          { type: "vstack", spacing: 2, alignment: "leading", padding: 8, flex: 1,
            background: { light: "#EDE9FE", dark: "#1a1a35" }, cornerRadius: 8, children: [
            { type: "text", content: "BTC", textStyle: "caption", fontWeight: "bold", color: "#f7931a" },
            { type: "text", content: "$67,432", fontSize: 16, fontWeight: "bold", color: "label" },
            { type: "text", content: "+2.34%", textStyle: "caption2", color: "#00ff88" },
          ]},
          { type: "vstack", spacing: 2, alignment: "leading", padding: 8, flex: 1,
            background: { light: "#EDE9FE", dark: "#1a1a35" }, cornerRadius: 8, children: [
            { type: "text", content: "ETH", textStyle: "caption", fontWeight: "bold", color: "#627eea" },
            { type: "text", content: "$3,821", fontSize: 16, fontWeight: "bold", color: "label" },
            { type: "text", content: "+1.12%", textStyle: "caption2", color: "#00ff88" },
          ]},
          { type: "vstack", spacing: 2, alignment: "leading", padding: 8, flex: 1,
            background: { light: "#EDE9FE", dark: "#1a1a35" }, cornerRadius: 8, children: [
            { type: "text", content: "SOL", textStyle: "caption", fontWeight: "bold", color: "#00ffa3" },
            { type: "text", content: "$142", fontSize: 16, fontWeight: "bold", color: "label" },
            { type: "text", content: "-0.87%", textStyle: "caption2", color: "#ff6b6b" },
          ]},
        ]},
        { type: "text", content: "BTC 7-day", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
        { type: "chart", chartType: "line", tint: "#00ff88", chartData: [
          { label: "Mon", value: 64000 }, { label: "Tue", value: 65200 }, { label: "Wed", value: 63800 },
          { label: "Thu", value: 66100 }, { label: "Fri", value: 65900 }, { label: "Sat", value: 66800 },
          { label: "Sun", value: 67432 },
        ]},
        { type: "text", content: "Volume 24h", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
        { type: "chart", chartType: "bar", tint: "#f7931a", chartData: [
          { label: "Mon", value: 28 }, { label: "Tue", value: 35 }, { label: "Wed", value: 22 },
          { label: "Thu", value: 41 }, { label: "Fri", value: 38 }, { label: "Sat", value: 19 },
          { label: "Sun", value: 31 },
        ]},
      ],
    },
  },
};
