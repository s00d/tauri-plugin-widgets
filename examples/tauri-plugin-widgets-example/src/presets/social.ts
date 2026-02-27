import type { PresetDef } from "./types";

export const social: PresetDef = {
  icon: "\u{1F465}",
  name: "Social",
  onAction(action, _payload, log) {
    switch (action) {
      case "share_stats": log("Stats shared!"); break;
      case "view_likes": log("Viewing likes..."); break;
      case "view_followers": log("Viewing followers..."); break;
    }
  },
  config: {
    small: {
      type: "vstack", padding: 12, spacing: 8, cornerRadius: 16,
      background: { light: "#F9FAFB", dark: "#18181b" },
      children: [
        { type: "label", text: "Social Stats", systemName: "person.2.fill", iconColor: "#a855f7",
          fontSize: 13, fontWeight: "semibold", color: "secondaryLabel" },
        { type: "grid", columns: 2, spacing: 8, rowSpacing: 8, children: [
          { type: "link", action: "view_likes", padding: 8,
            background: { light: "#F3F4F6", dark: "#27272a" }, cornerRadius: 8, children: [
            { type: "vstack", spacing: 2, alignment: "leading", children: [
              { type: "hstack", children: [
                { type: "spacer" },
                { type: "image", systemName: "heart.fill", color: "#ef4444", size: 12 },
              ]},
              { type: "text", content: "12.4K", fontSize: 16, fontWeight: "bold", color: "label" },
              { type: "text", content: "Likes", textStyle: "caption2", color: "secondaryLabel" },
            ]},
          ]},
          { type: "link", action: "view_followers", padding: 8,
            background: { light: "#F3F4F6", dark: "#27272a" }, cornerRadius: 8, children: [
            { type: "vstack", spacing: 2, alignment: "center", children: [
              { type: "shape", shapeType: "circle", fill: "#3b82f6", size: 8 },
              { type: "text", content: "8,291", fontSize: 16, fontWeight: "bold", color: "label" },
              { type: "text", content: "Followers", textStyle: "caption2", color: "secondaryLabel" },
            ]},
          ]},
        ]},
      ],
    },
    medium: {
      type: "hstack", padding: 14, spacing: 10, cornerRadius: 16,
      background: { light: "#F9FAFB", dark: "#18181b" },
      children: [
        { type: "grid", columns: 2, spacing: 8, rowSpacing: 8, children: [
          { type: "vstack", spacing: 2, alignment: "leading", padding: 8,
            background: { light: "#F3F4F6", dark: "#27272a" }, cornerRadius: 8, children: [
            { type: "hstack", children: [
              { type: "spacer" },
              { type: "image", systemName: "heart.fill", color: "#ef4444", size: 16 },
            ]},
            { type: "text", content: "12.4K", fontSize: 14, fontWeight: "bold", color: "label" },
            { type: "text", content: "Likes", textStyle: "caption2", color: "secondaryLabel" },
          ]},
          { type: "vstack", spacing: 2, alignment: "center", padding: 8,
            background: { light: "#F3F4F6", dark: "#27272a" }, cornerRadius: 8, children: [
            { type: "image", systemName: "person.2.fill", color: "#3b82f6", size: 16 },
            { type: "text", content: "8,291", fontSize: 14, fontWeight: "bold", color: "label" },
            { type: "text", content: "Followers", textStyle: "caption2", color: "secondaryLabel" },
          ]},
          { type: "vstack", spacing: 2, alignment: "center", padding: 8,
            background: { light: "#F3F4F6", dark: "#27272a" }, cornerRadius: 8, children: [
            { type: "image", systemName: "eye.fill", color: "#a855f7", size: 16 },
            { type: "text", content: "45.2K", fontSize: 14, fontWeight: "bold", color: "label" },
            { type: "text", content: "Views", textStyle: "caption2", color: "secondaryLabel" },
          ]},
          { type: "vstack", spacing: 2, alignment: "center", padding: 8,
            background: { light: "#F3F4F6", dark: "#27272a" }, cornerRadius: 8, children: [
            { type: "image", systemName: "bubble.left.fill", color: "#22c55e", size: 16 },
            { type: "text", content: "342", fontSize: 14, fontWeight: "bold", color: "label" },
            { type: "text", content: "Comments", textStyle: "caption2", color: "secondaryLabel" },
          ]},
        ]},
        { type: "chart", chartType: "line", tint: "#a855f7", chartData: [
          { label: "Mon", value: 1200 }, { label: "Tue", value: 1800 }, { label: "Wed", value: 1500 },
          { label: "Thu", value: 2100 }, { label: "Fri", value: 1900 }, { label: "Sat", value: 2400 },
          { label: "Sun", value: 2200 },
        ]},
      ],
    },
    large: {
      type: "vstack", padding: 16, spacing: 10, cornerRadius: 16,
      background: { light: "#F9FAFB", dark: "#18181b" },
      children: [
        { type: "hstack", spacing: 8, children: [
          { type: "text", content: "Social Dashboard", textStyle: "headline", color: "label" },
          { type: "spacer" },
          { type: "button", label: "Share", action: "share_stats",
            backgroundColor: "#a855f7", color: "#ffffff", fontSize: 11, cornerRadius: 6 },
        ]},
        { type: "grid", columns: 4, spacing: 8, rowSpacing: 8, children: [
          { type: "vstack", spacing: 2, alignment: "leading", padding: 8,
            background: { light: "#F3F4F6", dark: "#27272a" }, cornerRadius: 8, children: [
            { type: "hstack", children: [
              { type: "spacer" },
              { type: "image", systemName: "heart.fill", color: "#ef4444", size: 18 },
            ]},
            { type: "text", content: "12.4K", fontSize: 16, fontWeight: "bold", color: "label" },
            { type: "text", content: "Likes", textStyle: "caption2", color: "secondaryLabel" },
          ]},
          { type: "vstack", spacing: 2, alignment: "center", padding: 8,
            background: { light: "#F3F4F6", dark: "#27272a" }, cornerRadius: 8, children: [
            { type: "image", systemName: "person.2.fill", color: "#3b82f6", size: 18 },
            { type: "text", content: "8,291", fontSize: 16, fontWeight: "bold", color: "label" },
            { type: "text", content: "Followers", textStyle: "caption2", color: "secondaryLabel" },
          ]},
          { type: "vstack", spacing: 2, alignment: "center", padding: 8,
            background: { light: "#F3F4F6", dark: "#27272a" }, cornerRadius: 8, children: [
            { type: "image", systemName: "eye.fill", color: "#a855f7", size: 18 },
            { type: "text", content: "45.2K", fontSize: 16, fontWeight: "bold", color: "label" },
            { type: "text", content: "Views", textStyle: "caption2", color: "secondaryLabel" },
          ]},
          { type: "vstack", spacing: 2, alignment: "center", padding: 8,
            background: { light: "#F3F4F6", dark: "#27272a" }, cornerRadius: 8, children: [
            { type: "image", systemName: "bubble.left.fill", color: "#22c55e", size: 18 },
            { type: "text", content: "342", fontSize: 16, fontWeight: "bold", color: "label" },
            { type: "text", content: "Comments", textStyle: "caption2", color: "secondaryLabel" },
          ]},
        ]},
        { type: "divider", color: "separator" },
        { type: "text", content: "Engagement (7 days)", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
        { type: "chart", chartType: "line", tint: "#a855f7", chartData: [
          { label: "Mon", value: 1200 }, { label: "Tue", value: 1800 }, { label: "Wed", value: 1500 },
          { label: "Thu", value: 2100 }, { label: "Fri", value: 1900 }, { label: "Sat", value: 2400 },
          { label: "Sun", value: 2200 },
        ]},
        { type: "text", content: "Top Posts", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
        { type: "hstack", spacing: 8, children: [
          { type: "text", content: "1.", fontSize: 12, fontWeight: "bold", color: "#fbbf24" },
          { type: "text", content: "New product launch \u2014 4.2K likes", textStyle: "footnote", color: "label", flex: 1, alignment: "leading" },
        ]},
        { type: "hstack", spacing: 8, children: [
          { type: "text", content: "2.", fontSize: 12, fontWeight: "bold", color: "#94a3b8" },
          { type: "text", content: "Behind the scenes \u2014 2.8K likes", textStyle: "footnote", color: "label", flex: 1, alignment: "leading" },
        ]},
        { type: "hstack", spacing: 8, children: [
          { type: "text", content: "3.", fontSize: 12, fontWeight: "bold", color: "#cd7c2e" },
          { type: "text", content: "Team photo \u2014 1.9K likes", textStyle: "footnote", color: "label", flex: 1, alignment: "leading" },
        ]},
      ],
    },
  },
};
