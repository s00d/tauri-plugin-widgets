import type { PresetDef } from "./types";
import type { WidgetConfig, WidgetElement } from "tauri-plugin-widgets-api";

type TaskItem = { id: string; text: string; checked: boolean };

const baseTasks: TaskItem[] = [
  { id: "buy_groceries", text: "Buy groceries", checked: true },
  { id: "call_dentist", text: "Call dentist", checked: false },
  { id: "send_report", text: "Send report", checked: true },
  { id: "review_pr_42", text: "Review PR #42", checked: false },
  { id: "book_flights", text: "Book flight tickets", checked: false },
];

/** The app owns the list (texts, adding); the checks live in widget state. */
let tasksState: TaskItem[] = baseTasks.map((t) => ({ ...t }));

/** State keys: `tk_<id>` (checked) and `tk_n` (how many are checked). A tap flips the
 *  task and moves the counter in-process (WidgetKit AppIntent / Android receiver /
 *  desktop window), so checking off needs no app round-trip. */
const key = (t: TaskItem) => `tk_${t.id}`;

function defaultState(): Record<string, unknown> {
  const st: Record<string, unknown> = { tk_n: tasksState.filter((t) => t.checked).length };
  for (const t of tasksState) st[key(t)] = t.checked;
  return st;
}

/** One checkbox per task: two variants, only the one matching the state is drawn. */
function taskRows(limit?: number): WidgetElement[] {
  const rows = limit ? tasksState.slice(0, limit) : tasksState;
  return rows.flatMap((t): WidgetElement[] => [
    { type: "toggle", variant: "checkbox", label: t.text, isOn: true, tint: "#4CAF50",
      set: { [key(t)]: false, tk_n: "$dec" }, when: { path: `state.${key(t)}`, eq: true } },
    { type: "toggle", variant: "checkbox", label: t.text, isOn: false, tint: "#4CAF50",
      set: { [key(t)]: true, tk_n: "$inc" }, when: { not: { path: `state.${key(t)}`, eq: true } } },
  ]);
}

function buildConfig(): WidgetConfig {
  const total = Math.max(tasksState.length, 1);
  const done = "{{state.tk_n}}";
  const bar = { type: "progress", value: 0, total, bind: { value: "state.tk_n" }, tint: "#4CAF50" } as const;
  return {
    state: defaultState(),
    small: {
      type: "vstack", padding: 12, spacing: 6, cornerRadius: 12,
      background: { light: "#ffffff", dark: "#1C1C1E" },
      children: [
        { type: "text", content: "Today's Tasks", textStyle: "headline", color: "label" },
        { type: "divider", color: "separator" },
        { type: "vstack", spacing: 4, alignment: "leading", children: taskRows(2) },
        { type: "spacer" },
        { type: "button", label: "Open App", action: "open_tasks",
          backgroundColor: { light: "#2196F3", dark: "#3B82F6" }, color: "#ffffff", cornerRadius: 8 },
      ],
    },
    medium: {
      type: "hstack", padding: 14, spacing: 12, cornerRadius: 12,
      background: { light: "#ffffff", dark: "#1C1C1E" },
      children: [
        { type: "vstack", spacing: 6, alignment: "leading", flex: 1, children: [
          { type: "text", content: "Today", textStyle: "headline", color: "label" },
          { type: "vstack", spacing: 3, alignment: "leading", children: taskRows(3) },
        ]},
        { type: "divider", color: "separator" },
        // Fixed-feel sidebar: big count + unlabeled bar + button (no nested card).
        { type: "vstack", spacing: 8, alignment: "center", frame: { width: 108 }, children: [
          { type: "text", content: `${done} / ${total}`, fontSize: 26, fontWeight: "bold", color: "#4CAF50",
            alignment: "center" },
          { type: "text", content: "completed", textStyle: "caption2", color: "secondaryLabel",
            alignment: "center" },
          { ...bar },
          { type: "button", label: "Add Task", action: "add_task",
            backgroundColor: "#4CAF50", color: "#ffffff", fontSize: 11, cornerRadius: 6 },
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
        { ...bar, label: `${done} of ${total} completed`, color: "secondaryLabel" },
        { type: "divider", color: "separator" },
        { type: "vstack", spacing: 4, alignment: "leading", children: taskRows() },
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
