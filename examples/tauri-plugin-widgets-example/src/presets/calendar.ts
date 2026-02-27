import type { WidgetConfig, WidgetElement } from "tauri-plugin-widgets-api";
import type { PresetDef } from "./types";

const WEEKDAY_SHORT = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] as const;

function mondayFirstWeekdayIndex(jsDay: number): number {
  // JS Date: Sunday=0..Saturday=6, we need Monday=0..Sunday=6
  return (jsDay + 6) % 7;
}

function buildMonthGrid(baseDate: Date): Array<Array<number | null>> {
  const year = baseDate.getFullYear();
  const month = baseDate.getMonth();
  const first = new Date(year, month, 1);
  const daysInMonth = new Date(year, month + 1, 0).getDate();
  const startCol = mondayFirstWeekdayIndex(first.getDay());

  const cells: Array<number | null> = [];
  for (let i = 0; i < startCol; i++) cells.push(null);
  for (let day = 1; day <= daysInMonth; day++) cells.push(day);
  while (cells.length < 42) cells.push(null); // always 6 rows

  const rows: Array<Array<number | null>> = [];
  for (let i = 0; i < 42; i += 7) rows.push(cells.slice(i, i + 7));
  return rows;
}

function dayCell(day: number | null, today: number): WidgetElement {
  const isToday = day === today;
  return {
    type: "container",
    flex: 1,
    contentAlignment: "center",
    padding: 4,
    cornerRadius: 8,
    background: isToday ? { light: "#e11d48", dark: "#be123c" } : { light: "#f8fafc", dark: "#1e293b" },
    children: [
      {
        type: "text",
        content: day == null ? "" : String(day),
        fontSize: 12,
        fontWeight: isToday ? "bold" : "regular",
        color: isToday ? { light: "#ffffff", dark: "#ffffff" } : { light: "#0f172a", dark: "#e2e8f0" },
        alignment: "center",
      },
    ],
  };
}

function weekRow(days: Array<number | null>, today: number): WidgetElement {
  return {
    type: "hstack",
    spacing: 4,
    children: days.map((d) => dayCell(d, today)),
  };
}

function buildCalendarConfig(): WidgetConfig {
  const now = new Date();
  const day = now.getDate();
  const monthLong = now.toLocaleDateString("en-US", { month: "long" });
  const yearMonth = now.toLocaleDateString("en-US", { month: "long", year: "numeric" });
  const monthGrid = buildMonthGrid(now);

  const weekdayHeader: WidgetElement = {
    type: "hstack",
    spacing: 4,
    children: WEEKDAY_SHORT.map((name) => ({
      type: "container",
      flex: 1,
      children: [
        {
          type: "text",
          content: name,
          fontSize: 10,
          fontWeight: "semibold",
          color: { light: "#64748b", dark: "#94a3b8" },
          alignment: "center",
        },
      ],
    })),
  };

  const weekRows = monthGrid.map((row) => weekRow(row, day));

  const smallCaption: WidgetElement = {
    type: "text",
    content: `Today: ${day} ${monthLong}`,
    fontSize: 12,
    color: { light: "#64748b", dark: "#94a3b8" },
    alignment: "center",
  };

  const small: WidgetElement = {
    type: "vstack", padding: 12, spacing: 8, cornerRadius: 12,
    background: { light: "#ffffff", dark: "#0f172a" },
    children: [
      { type: "text", content: yearMonth, fontSize: 16, fontWeight: "bold", color: { light: "#0f172a", dark: "#f8fafc" }, alignment: "center" },
      smallCaption,
      weekdayHeader,
      ...weekRows.slice(0, 4),
    ],
  };

  const medium: WidgetElement = {
    type: "vstack", padding: 12, spacing: 6, cornerRadius: 12,
    background: { light: "#ffffff", dark: "#0f172a" },
    children: [
      { type: "text", content: yearMonth, fontSize: 18, fontWeight: "bold", color: { light: "#0f172a", dark: "#f8fafc" }, alignment: "center" },
      weekdayHeader,
      ...weekRows,
    ],
  };

  const large: WidgetElement = {
    type: "vstack", padding: 14, spacing: 6, cornerRadius: 12,
    background: { light: "#ffffff", dark: "#0f172a" },
    children: [
      { type: "text", content: yearMonth, fontSize: 20, fontWeight: "bold", color: { light: "#0f172a", dark: "#f8fafc" }, alignment: "center" },
      weekdayHeader,
      ...weekRows,
      { type: "button", label: "Add Event", action: "add_event",
        backgroundColor: { light: "#e11d48", dark: "#be123c" }, color: { light: "#ffffff", dark: "#ffffff" }, fontSize: 12, cornerRadius: 6, textAlignment: "center" },
    ],
  };

  return { small, medium, large };
}

export const calendar: PresetDef = {
  icon: "\u{1F4C5}",
  name: "Calendar",
  config: buildCalendarConfig(),
  builder: buildCalendarConfig,
  intervalMs: 60000,
  onAction(action, _payload, log) {
    if (action === "add_event") {
      log("New event dialog opened");
    }
  },
};
