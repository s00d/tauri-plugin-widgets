import type { PresetDef } from "./types";

export const revenue: PresetDef = {
  icon: "\u{1F4CA}",
  name: "Revenue",
  onAction(action, _payload, log) {
    if (action === "export_report") log("Revenue report exported");
  },
  config: {
    small: {
      type: "vstack", padding: 12, spacing: 6, cornerRadius: 16,
      background: { light: "#F5F3FF", dark: "#1e1e2e" },
      children: [
        { type: "text", content: "Revenue", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
        { type: "text", content: "$12,450", fontSize: 28, fontWeight: "bold", color: "#a6e3a1" },
        { type: "chart", chartType: "bar", tint: { light: "#6366f1", dark: "#89b4fa" }, chartData: [
          { label: "Mon", value: 120 }, { label: "Tue", value: 180, color: "#a6e3a1" },
          { label: "Wed", value: 90 }, { label: "Thu", value: 210, color: "#a6e3a1" }, { label: "Fri", value: 160 },
        ]},
      ],
    },
    medium: {
      type: "hstack", padding: 14, spacing: 16, cornerRadius: 16,
      background: { light: "#F5F3FF", dark: "#1e1e2e" },
      children: [
        { type: "vstack", spacing: 4, alignment: "leading", flex: 1, children: [
          { type: "text", content: "Revenue", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
          { type: "text", content: "$12,450", fontSize: 32, fontWeight: "bold", color: "#a6e3a1" },
          { type: "text", content: "+18.3% vs last week", textStyle: "caption", color: "#a6e3a1" },
          { type: "spacer" },
          { type: "text", content: "Expenses: $4,210", textStyle: "caption2", color: "#f38ba8" },
        ]},
        { type: "chart", chartType: "bar", tint: { light: "#6366f1", dark: "#89b4fa" }, chartData: [
          { label: "Mon", value: 120 }, { label: "Tue", value: 180, color: "#a6e3a1" },
          { label: "Wed", value: 90 }, { label: "Thu", value: 210, color: "#a6e3a1" },
          { label: "Fri", value: 160 }, { label: "Sat", value: 95 }, { label: "Sun", value: 70 },
        ]},
      ],
    },
    large: {
      type: "vstack", padding: 16, spacing: 10, cornerRadius: 16,
      background: { light: "#F5F3FF", dark: "#1e1e2e" },
      children: [
        { type: "hstack", spacing: 8, children: [
          { type: "text", content: "Revenue Dashboard", textStyle: "headline", color: "label" },
          { type: "spacer" },
          { type: "button", label: "Export", action: "export_report",
            backgroundColor: { light: "#6366f1", dark: "#89b4fa" }, color: { light: "#ffffff", dark: "#1e1e2e" }, fontSize: 11, cornerRadius: 6 },
        ]},
        { type: "grid", columns: 3, spacing: 8, rowSpacing: 8, children: [
          { type: "vstack", spacing: 2, alignment: "center", padding: 8,
            background: { light: "#EDE9FE", dark: "#313244" }, cornerRadius: 8, children: [
            { type: "text", content: "Revenue", textStyle: "caption2", color: "secondaryLabel" },
            { type: "text", content: "$12,450", fontSize: 18, fontWeight: "bold", color: "#a6e3a1" },
          ]},
          { type: "vstack", spacing: 2, alignment: "center", padding: 8,
            background: { light: "#EDE9FE", dark: "#313244" }, cornerRadius: 8, children: [
            { type: "text", content: "Expenses", textStyle: "caption2", color: "secondaryLabel" },
            { type: "text", content: "$4,210", fontSize: 18, fontWeight: "bold", color: "#f38ba8" },
          ]},
          { type: "vstack", spacing: 2, alignment: "center", padding: 8,
            background: { light: "#EDE9FE", dark: "#313244" }, cornerRadius: 8, children: [
            { type: "text", content: "Profit", textStyle: "caption2", color: "secondaryLabel" },
            { type: "text", content: "$8,240", fontSize: 18, fontWeight: "bold", color: { light: "#6366f1", dark: "#89b4fa" } },
          ]},
        ]},
        { type: "divider", color: "separator" },
        { type: "text", content: "Weekly Revenue", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
        { type: "chart", chartType: "bar", tint: { light: "#6366f1", dark: "#89b4fa" }, chartData: [
          { label: "Mon", value: 120 }, { label: "Tue", value: 180, color: "#a6e3a1" },
          { label: "Wed", value: 90 }, { label: "Thu", value: 210, color: "#a6e3a1" },
          { label: "Fri", value: 160 }, { label: "Sat", value: 95 }, { label: "Sun", value: 70 },
        ]},
        { type: "hstack", spacing: 12, children: [
          { type: "vstack", spacing: 4, flex: 1, children: [
            { type: "text", content: "By Category", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
            { type: "chart", chartType: "pie", chartData: [
              { label: "SaaS", value: 45, color: "#6366f1" },
              { label: "Services", value: 30, color: "#a6e3a1" },
              { label: "Products", value: 25, color: "#f38ba8" },
            ]},
          ]},
          { type: "vstack", spacing: 4, flex: 1, children: [
            { type: "text", content: "Monthly Trend", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
            { type: "chart", chartType: "area", tint: "#a6e3a1", chartData: [
              { label: "W1", value: 9800 }, { label: "W2", value: 10500 },
              { label: "W3", value: 11200 }, { label: "W4", value: 12450 },
            ]},
          ]},
        ]},
      ],
    },
  },
};
