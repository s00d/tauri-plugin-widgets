import type { WidgetConfig, WidgetElement } from "tauri-plugin-widgets-api";
import type { PresetDef } from "./types";

const TARGET = "2026-04-06T00:00:00Z";
const START = "2025-11-01T00:00:00Z";
const LABEL = "Summer Vacation";

const block = (val: string, label: string): WidgetElement => ({
  type: "container", contentAlignment: "center", padding: 12,
  background: { light: "#4338ca", dark: "#3730a3" }, cornerRadius: 12, children: [
    { type: "vstack", spacing: 2, alignment: "center", children: [
      { type: "text", content: val, fontSize: 36, fontWeight: "bold", color: "#fff" },
      { type: "text", content: label, textStyle: "caption", color: "#a5b4fc" },
    ]},
  ],
});

const blockSm = (val: string, label: string): WidgetElement => ({
  type: "vstack", spacing: 0, alignment: "center", children: [
    { type: "text", content: val, fontSize: 18, fontWeight: "bold", color: "#fff" },
    { type: "text", content: label, textStyle: "caption2", color: "#a5b4fc" },
  ],
});

function buildCountdownConfig(): WidgetConfig {
  const target = new Date(TARGET);
  const now = new Date();
  const diff = target.getTime() - now.getTime();
  const totalDays = Math.max(0, Math.floor(diff / 86400000));
  const months = Math.floor(totalDays / 30);
  const days = totalDays % 30;
  const hours = Math.max(0, Math.floor((diff % 86400000) / 3600000));
  const minutes = Math.max(0, Math.floor((diff % 3600000) / 60000));

  const totalSpan = target.getTime() - new Date(START).getTime();
  const elapsed = now.getTime() - new Date(START).getTime();
  const progress = Math.min(1, Math.max(0, elapsed / totalSpan));
  const pctStr = (progress * 100).toFixed(0);

  const small: WidgetElement = {
    type: "vstack", padding: 14, spacing: 8, cornerRadius: 16,
    background: { gradientType: "linear", colors: ["#312e81", "#4c1d95"], direction: "topLeadingToBottomTrailing" },
    children: [
      { type: "image", systemName: "hourglass", color: "#c4b5fd", size: 24 },
      { type: "text", content: `${totalDays}d ${hours}h ${minutes}m`, fontSize: 28, fontWeight: "bold", color: "#ffffff" },
      { type: "text", content: LABEL, textStyle: "caption", fontWeight: "medium", color: "#c7d2fe" },
      { type: "progress", value: progress, tint: "#818cf8", color: "#a5b4fc" },
    ],
  };

  const medium: WidgetElement = {
    type: "hstack", padding: 16, spacing: 16, cornerRadius: 16, background: "#312e81",
    children: [
      { type: "vstack", spacing: 4, alignment: "center", children: [
        { type: "image", systemName: "hourglass", color: "#c4b5fd", size: 28 },
        { type: "text", content: String(totalDays), fontSize: 48, fontWeight: "bold", color: "#ffffff" },
        { type: "text", content: "days", textStyle: "subheadline", color: "#a5b4fc" },
      ] },
      { type: "divider", color: "#4338ca" },
      { type: "vstack", spacing: 6, alignment: "leading", flex: 1, children: [
        { type: "text", content: LABEL, textStyle: "headline", color: "#e0e7ff" },
        { type: "text", content: "Apr 6, 2026", textStyle: "footnote", color: "#a5b4fc" },
        { type: "progress", value: progress, tint: "#818cf8", label: `${pctStr}% elapsed`, color: "#a5b4fc" },
        { type: "spacer" },
        { type: "hstack", spacing: 8, children: [
          blockSm(String(months), "months"),
          blockSm(String(days), "days"),
          blockSm(String(hours), "hours"),
        ] },
      ] },
    ],
  };

  const large: WidgetElement = {
    type: "vstack", padding: 20, spacing: 12, cornerRadius: 16, background: "#312e81",
    children: [
      { type: "hstack", spacing: 12, children: [
        { type: "image", systemName: "hourglass", color: "#c4b5fd", size: 32 },
        { type: "vstack", spacing: 2, alignment: "leading", flex: 1, children: [
          { type: "text", content: LABEL, textStyle: "title2", fontWeight: "bold", color: "#e0e7ff" },
          { type: "text", content: "April 6, 2026", textStyle: "subheadline", color: "#a5b4fc" },
        ] },
      ] },
      { type: "divider", color: "#4338ca" },
      { type: "hstack", spacing: 16, children: [
        block(String(months), "months"),
        block(String(days), "days"),
        block(String(hours), "hours"),
        block(String(minutes), "min"),
      ] },
      { type: "progress", value: progress, tint: "#818cf8", label: `${pctStr}% of waiting period elapsed`, color: "#a5b4fc" },
      { type: "spacer" },
      { type: "text", content: `"Life is what happens when you're busy making other plans."`, textStyle: "footnote", color: "#c7d2fe", alignment: "center" },
    ],
  };

  return { small, medium, large };
}

export const countdown: PresetDef = {
  icon: "\u231B",
  name: "Countdown",
  config: buildCountdownConfig(),
  builder: buildCountdownConfig,
  intervalMs: 1000,
};
