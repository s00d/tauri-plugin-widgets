import type { WidgetConfig, WidgetElement } from "tauri-plugin-widgets-api";
import type { PresetDef } from "./types";

const LABEL = "Summer Vacation";
const DAY = 86_400_000;

/** Next June 1 (local midnight); the wait is counted from November 1 before it. */
function span(now: Date): { target: Date; start: Date } {
  let year = now.getFullYear();
  if (now >= new Date(year, 5, 1)) year += 1;
  return { target: new Date(year, 5, 1), start: new Date(year - 1, 10, 1) };
}

/** Text for a block, or any element (the running timer). */
const content = (val: string | WidgetElement, fontSize: number): WidgetElement =>
  typeof val === "string" ? { type: "text", content: val, fontSize, fontWeight: "bold", color: "#fff" } : val;

const block = (val: string | WidgetElement, label: string): WidgetElement => ({
  type: "container", contentAlignment: "center", padding: 12,
  background: { light: "#4338ca", dark: "#3730a3" }, cornerRadius: 12, children: [
    { type: "vstack", spacing: 2, alignment: "center", children: [
      content(val, 36),
      { type: "text", content: label, textStyle: "caption", color: "#a5b4fc" },
    ]},
  ],
});

const blockSm = (val: string | WidgetElement, label: string): WidgetElement => ({
  type: "vstack", spacing: 0, alignment: "center", children: [
    content(val, 18),
    { type: "text", content: label, textStyle: "caption2", color: "#a5b4fc" },
  ],
});

/** Layouts as of `at`. Hours / minutes / seconds are a timer counting down to the moment
 *  the day count drops, so WidgetKit runs them itself; that moment is the next entry. */
function layouts(at: number): Pick<WidgetConfig, "small" | "medium" | "large"> {
  const { target, start } = span(new Date(at));
  const diff = target.getTime() - at;
  // Whole days left after today's part: at a day change the count drops and the timer
  // starts again from 24:00:00.
  const totalDays = Math.max(0, Math.ceil(diff / DAY) - 1);
  const months = Math.floor(totalDays / 30);
  const days = totalDays % 30;
  const dayEnd = new Date(target.getTime() - totalDays * DAY).toISOString();
  const clock = (fontSize: number, color = "#ffffff"): WidgetElement => ({
    type: "timer", targetDate: dayEnd, counting: "down", showHours: true, fontSize, fontWeight: "bold", color,
  });

  const progress = Math.min(1, Math.max(0, (at - start.getTime()) / (target.getTime() - start.getTime())));
  const pctStr = (progress * 100).toFixed(0);
  // Runs on the device between `startAt` and `endAt` (WidgetKit ProgressView(timerInterval)).
  const bar = (label?: string): WidgetElement => ({
    type: "progress", value: Math.round(progress * 1000) / 1000, startAt: start.getTime(), endAt: target.getTime(),
    tint: "#818cf8", color: "#a5b4fc", ...(label ? { label } : {}),
  });
  const dateShort = target.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
  const dateLong = target.toLocaleDateString("en-US", { month: "long", day: "numeric", year: "numeric" });

  const small: WidgetElement = {
    type: "vstack", padding: 14, spacing: 8, cornerRadius: 16,
    background: { gradientType: "linear", colors: ["#312e81", "#4c1d95"], direction: "topLeadingToBottomTrailing" },
    children: [
      { type: "image", systemName: "hourglass", color: "#c4b5fd", size: 24 },
      { type: "hstack", spacing: 6, children: [
        { type: "text", content: `${totalDays}d`, fontSize: 26, fontWeight: "bold", color: "#ffffff" },
        clock(14, "#c7d2fe"),
      ] },
      { type: "text", content: LABEL, textStyle: "caption", fontWeight: "medium", color: "#c7d2fe" },
      bar(),
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
        { type: "text", content: dateShort, textStyle: "footnote", color: "#a5b4fc" },
        bar(`${pctStr}% elapsed`),
        { type: "spacer" },
        { type: "hstack", spacing: 8, children: [
          blockSm(String(months), "months"),
          blockSm(String(days), "days"),
          blockSm(clock(18), "h:m:s"),
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
          { type: "text", content: dateLong, textStyle: "subheadline", color: "#a5b4fc" },
        ] },
      ] },
      { type: "divider", color: "#4338ca" },
      { type: "hstack", spacing: 16, children: [
        block(String(months), "months"),
        block(String(days), "days"),
        block(clock(24), "h:m:s"),
      ] },
      bar(`${pctStr}% of waiting period elapsed`),
      { type: "spacer" },
      { type: "text", content: `"Life is what happens when you're busy making other plans."`, textStyle: "footnote", color: "#c7d2fe", alignment: "center" },
    ],
  };

  return { small, medium, large };
}

/** Now plus the next 2 day changes as timeline entries; WidgetKit asks for a new timeline
 *  after the last one, so the count goes on with no app. */
function buildCountdownConfig(): WidgetConfig {
  const now = Date.now();
  const { target } = span(new Date(now));
  const totalDays = Math.max(0, Math.ceil((target.getTime() - now) / DAY) - 1);
  const firstChange = target.getTime() - totalDays * DAY;
  // Two day changes are enough: WidgetKit asks for a new timeline after the last one, and
  // every entry is rendered when the timeline loads.
  const entries = Array.from({ length: Math.min(2, totalDays) }, (_, i) => {
    const at = firstChange + i * DAY;
    return { at, ...layouts(at) };
  });
  return { ...layouts(now), entries };
}

export const countdown: PresetDef = {
  icon: "\u231B",
  name: "Countdown",
  config: buildCountdownConfig(),
  builder: buildCountdownConfig,
  // Timers, the bar and entries run on the device; the tick only moves the entry window on.
  intervalMs: 60_000,
};
