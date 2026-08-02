import type { WidgetElement } from "tauri-plugin-widgets-api";
import type { PresetDef } from "./types";

/** Initials badge — container centers text inside the circle (text+clip alone sat top-leading). */
function initialsAvatar(size: number, fontSize: number): WidgetElement {
  return {
    type: "container",
    contentAlignment: "center",
    frame: { width: size, height: size },
    cornerRadius: size / 2,
    clipShape: "circle",
    background: {
      gradientType: "linear",
      colors: ["#667eea", "#764ba2"],
      direction: "topLeadingToBottomTrailing",
    },
    children: [
      {
        type: "text",
        content: "AK",
        fontSize,
        fontWeight: "bold",
        color: "#ffffff",
        alignment: "center",
      },
    ],
  };
}

export const userProfile: PresetDef = {
  icon: "\u{1F464}",
  name: "User Profile",
  onAction(action, _payload, log) {
    switch (action) {
      case "edit_profile": log("Opening profile editor..."); break;
      case "view_stats": log("Viewing detailed stats..."); break;
    }
  },
  config: {
    small: {
      type: "vstack", padding: 14, spacing: 10, cornerRadius: 16,
      background: { light: "#F8F9FA", dark: "#1C1C1E" },
      children: [
        { type: "hstack", spacing: 10, children: [
          initialsAvatar(44, 16),
          { type: "vstack", spacing: 2, alignment: "leading", flex: 1, children: [
            { type: "text", content: "Alex Kim", textStyle: "headline", color: { light: "#0f172a", dark: "#f1f5f9" } },
            { type: "text", content: "Developer", textStyle: "caption", color: { light: "#64748b", dark: "#94a3b8" } },
          ]},
        ]},
        { type: "hstack", spacing: 16, children: [
          { type: "vstack", spacing: 0, alignment: "center", flex: 1, children: [
            { type: "text", content: "142", fontSize: 18, fontWeight: "bold", color: { light: "#0f172a", dark: "#f1f5f9" } },
            { type: "text", content: "Posts", textStyle: "caption2", color: { light: "#64748b", dark: "#94a3b8" }, lineLimit: 1 },
          ]},
          { type: "vstack", spacing: 0, alignment: "center", flex: 1, children: [
            { type: "text", content: "8.2K", fontSize: 18, fontWeight: "bold", color: { light: "#0f172a", dark: "#f1f5f9" } },
            { type: "text", content: "Followers", textStyle: "caption2", color: { light: "#64748b", dark: "#94a3b8" }, lineLimit: 1 },
          ]},
          { type: "vstack", spacing: 0, alignment: "center", flex: 1, children: [
            { type: "text", content: "523", fontSize: 18, fontWeight: "bold", color: { light: "#0f172a", dark: "#f1f5f9" } },
            { type: "text", content: "Following", textStyle: "caption2", color: { light: "#64748b", dark: "#94a3b8" }, lineLimit: 1 },
          ]},
        ]},
      ],
    },

    medium: {
      type: "hstack", padding: 16, spacing: 16, cornerRadius: 16,
      background: { light: "#F8F9FA", dark: "#1C1C1E" },
      children: [
        { type: "vstack", spacing: 8, alignment: "center", children: [
          initialsAvatar(60, 22),
          { type: "text", content: "Alex Kim", textStyle: "headline", color: { light: "#0f172a", dark: "#f1f5f9" } },
          { type: "text", content: "@alexkim", textStyle: "caption", color: { light: "#64748b", dark: "#94a3b8" } },
        ]},
        { type: "divider", color: { light: "#e5e7eb", dark: "#3f3f46" } },
        { type: "vstack", spacing: 8, alignment: "leading", flex: 1, children: [
          { type: "hstack", spacing: 12, children: [
            { type: "vstack", spacing: 0, alignment: "center", flex: 1, children: [
              { type: "text", content: "142", fontSize: 20, fontWeight: "bold", color: { light: "#0f172a", dark: "#f1f5f9" } },
              { type: "text", content: "Posts", textStyle: "caption2", color: { light: "#64748b", dark: "#94a3b8" }, lineLimit: 1 },
            ]},
            { type: "vstack", spacing: 0, alignment: "center", flex: 1, children: [
              { type: "text", content: "8.2K", fontSize: 20, fontWeight: "bold", color: { light: "#0f172a", dark: "#f1f5f9" } },
              { type: "text", content: "Followers", textStyle: "caption2", color: { light: "#64748b", dark: "#94a3b8" }, lineLimit: 1 },
            ]},
            { type: "vstack", spacing: 0, alignment: "center", flex: 1, children: [
              { type: "text", content: "523", fontSize: 20, fontWeight: "bold", color: { light: "#0f172a", dark: "#f1f5f9" } },
              { type: "text", content: "Following", textStyle: "caption2", color: { light: "#64748b", dark: "#94a3b8" }, lineLimit: 1 },
            ]},
          ]},
          { type: "text", content: "Full-stack developer \u00B7 Open source contributor", textStyle: "caption", color: { light: "#64748b", dark: "#94a3b8" }, lineLimit: 1,
            padding: { top: 6, bottom: 6, leading: 12, trailing: 12 },
            background: { light: "#EEEEF0", dark: "#2C2C2E" }, cornerRadius: 8, clipShape: "capsule" },
          { type: "button", label: "View Stats", action: "view_stats",
            backgroundColor: { light: "#667eea", dark: "#764ba2" }, color: "#ffffff", fontSize: 12, cornerRadius: 8 },
        ]},
      ],
    },

    large: {
      type: "vstack", padding: 16, spacing: 12, cornerRadius: 16,
      background: { light: "#F8F9FA", dark: "#1C1C1E" },
      children: [
        { type: "hstack", spacing: 14, children: [
          initialsAvatar(70, 26),
          { type: "vstack", spacing: 4, alignment: "leading", flex: 1, children: [
            { type: "text", content: "Alex Kim", textStyle: "title2", fontWeight: "bold", color: { light: "#0f172a", dark: "#f1f5f9" } },
            { type: "text", content: "@alexkim \u00B7 Developer", textStyle: "subheadline", color: { light: "#64748b", dark: "#94a3b8" } },
            { type: "text", content: "Full-stack developer. Open source contributor. Coffee enthusiast.", textStyle: "footnote", color: { light: "#64748b", dark: "#94a3b8" }, lineLimit: 2 },
          ]},
          { type: "button", label: "Edit", action: "edit_profile",
            backgroundColor: { light: "#667eea", dark: "#764ba2" }, color: "#ffffff", fontSize: 12, cornerRadius: 8 },
        ]},
        { type: "divider", color: { light: "#e5e7eb", dark: "#3f3f46" } },
        { type: "hstack", spacing: 8, children: [
          { type: "vstack", spacing: 2, alignment: "center", padding: 12, flex: 1,
            background: { light: "#EEF2FF", dark: "#27272A" }, cornerRadius: 12, children: [
            { type: "text", content: "142", fontSize: 22, fontWeight: "bold", color: { light: "#667eea", dark: "#818CF8" } },
            { type: "text", content: "Posts", textStyle: "caption", color: { light: "#64748b", dark: "#94a3b8" } },
          ]},
          { type: "vstack", spacing: 2, alignment: "center", padding: 12, flex: 1,
            background: { light: "#F0FDF4", dark: "#27272A" }, cornerRadius: 12, children: [
            { type: "text", content: "8.2K", fontSize: 22, fontWeight: "bold", color: { light: "#22C55E", dark: "#4ADE80" } },
            { type: "text", content: "Followers", textStyle: "caption", color: { light: "#64748b", dark: "#94a3b8" } },
          ]},
          { type: "vstack", spacing: 2, alignment: "center", padding: 12, flex: 1,
            background: { light: "#FFF7ED", dark: "#27272A" }, cornerRadius: 12, children: [
            { type: "text", content: "523", fontSize: 22, fontWeight: "bold", color: { light: "#F59E0B", dark: "#FBBF24" } },
            { type: "text", content: "Following", textStyle: "caption", color: { light: "#64748b", dark: "#94a3b8" } },
          ]},
        ]},
        { type: "divider", color: { light: "#e5e7eb", dark: "#3f3f46" } },
        { type: "text", content: "Activity (7 days)", textStyle: "footnote", fontWeight: "semibold", color: { light: "#64748b", dark: "#94a3b8" } },
        { type: "chart", chartType: "area", tint: { light: "#667eea", dark: "#818CF8" }, chartData: [
          { label: "Mon", value: 5 }, { label: "Tue", value: 12 }, { label: "Wed", value: 8 },
          { label: "Thu", value: 15 }, { label: "Fri", value: 22 }, { label: "Sat", value: 7 },
          { label: "Sun", value: 11 },
        ]},
        { type: "hstack", spacing: 8, children: [
          { type: "label", text: "3 new followers today", systemName: "person.badge.plus", iconColor: { light: "#22C55E", dark: "#4ADE80" }, fontSize: 12, color: { light: "#64748b", dark: "#94a3b8" } },
          { type: "spacer" },
          { type: "label", text: "Streak: 14 days", systemName: "flame.fill", iconColor: "#F59E0B", fontSize: 12, color: { light: "#64748b", dark: "#94a3b8" } },
        ]},
      ],
    },
  },
};
