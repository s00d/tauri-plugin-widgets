import type { PresetDef } from "./types";
import type { WidgetConfig } from "tauri-plugin-widgets-api";

type TaskItem = { id: string; text: string; checked: boolean };

const baseTasks: TaskItem[] = [
  { id: "buy_groceries", text: "Buy groceries", checked: true },
  { id: "call_dentist", text: "Call dentist", checked: false },
  { id: "send_report", text: "Send report", checked: true },
  { id: "review_pr_42", text: "Review PR #42", checked: false },
  { id: "book_flights", text: "Book flight tickets", checked: false },
];

let tasksState: TaskItem[] = baseTasks.map((t) => ({ ...t }));

function completedCount(): number {
  return tasksState.filter((t) => t.checked).length;
}

function listItems(limit?: number) {
  const rows = limit ? tasksState.slice(0, limit) : tasksState;
  return rows.map((t) => ({
    text: t.text,
    checked: t.checked,
    action: `toggle_task:${t.id}`,
  }));
}

function buildConfig(): WidgetConfig {
  const done = completedCount();
  const total = Math.max(tasksState.length, 1);
  return {
    small: {
      type: "vstack", padding: 12, spacing: 6, cornerRadius: 12,
      background: { light: "#ffffff", dark: "#1C1C1E" },
      children: [
        { type: "text", content: "Today's Tasks", textStyle: "headline", color: "label" },
        { type: "divider", color: "separator" },
        { type: "list", spacing: 4, items: listItems(2) },
        { type: "spacer" },
        { type: "button", label: "Open App", action: "open_tasks",
          backgroundColor: { light: "#2196F3", dark: "#3B82F6" }, color: "#ffffff", cornerRadius: 8 },
      ],
    },
    medium: {
      type: "hstack", padding: 14, spacing: 16, cornerRadius: 12,
      background: { light: "#ffffff", dark: "#1C1C1E" },
      children: [
        { type: "vstack", spacing: 6, alignment: "leading", flex: 1, children: [
          { type: "text", content: "Today", textStyle: "headline", color: "label" },
          { type: "list", spacing: 3, fontSize: 12, items: listItems(3) },
        ]},
        { type: "divider", color: "separator" },
        { type: "vstack", spacing: 6, alignment: "center", children: [
          { type: "vstack", spacing: 2, alignment: "center", padding: 8,
            background: { light: "#F0FDF4", dark: "#14532D" }, cornerRadius: 10,
            children: [
              { type: "text", content: `${done} / ${total}`, fontSize: 28, fontWeight: "bold", color: "#4CAF50" },
              { type: "text", content: "completed", textStyle: "caption", color: "secondaryLabel" },
            ],
          },
          { type: "progress", value: done / total, tint: "#4CAF50", label: `${done} of ${total} completed`, color: "secondaryLabel" },
          { type: "button", label: "Add Task", action: "add_task",
            backgroundColor: "#4CAF50", color: "#ffffff", fontSize: 12, cornerRadius: 6 },
        ]},
      ],
    },
    large: {
      type: "vstack", padding: 16, spacing: 8, cornerRadius: 12,
      background: { light: "#ffffff", dark: "#1C1C1E" },
      children: [
        { type: "hstack", spacing: 8, children: [
          { type: "text", content: "Today's Tasks", textStyle: "title3", fontWeight: "bold", color: "label" },
          { type: "spacer" },
          { type: "text", content: `${done} / ${total}`, textStyle: "footnote", fontWeight: "semibold", color: "#4CAF50",
            padding: { top: 4, bottom: 4, leading: 10, trailing: 10 },
            background: { light: "#F0FDF4", dark: "#14532D" }, cornerRadius: 12, clipShape: "capsule" },
        ]},
        { type: "progress", value: done / total, tint: "#4CAF50", label: `${done} of ${total} completed`, color: "secondaryLabel" },
        { type: "divider", color: "separator" },
        { type: "list", spacing: 4, items: listItems() },
        { type: "spacer" },
        { type: "hstack", spacing: 8, children: [
          { type: "button", label: "Add Task", action: "add_task",
            backgroundColor: "#4CAF50", color: "#ffffff", fontSize: 13, cornerRadius: 8 },
          { type: "button", label: "Open App", action: "open_tasks",
            backgroundColor: { light: "#2196F3", dark: "#3B82F6" }, color: "#ffffff", fontSize: 13, cornerRadius: 8 },
        ]},
      ],
    },
  };
}

export const tasks: PresetDef = {
  icon: "\u2705",
  name: "Tasks",
  config: buildConfig(),
  builder: () => buildConfig(),
  onAction(action, _payload, log) {
    if (action.startsWith("toggle_task:")) {
      const id = action.split(":", 2)[1];
      tasksState = tasksState.map((t) => (t.id === id ? { ...t, checked: !t.checked } : t));
      log(`Task toggled: ${id}`);
      return;
    }
    switch (action) {
      case "open_tasks": log("Opening tasks app..."); break;
      case "add_task": {
        const id = `task_${Date.now()}`;
        tasksState = [
          ...tasksState,
          { id, text: `New task #${tasksState.length + 1}`, checked: false },
        ];
        log("New task added");
        break;
      }
    }
  },
};
