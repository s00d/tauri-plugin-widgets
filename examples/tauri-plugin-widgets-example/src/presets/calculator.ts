import type { ColorValue, WidgetConfig, WidgetElement } from "tauri-plugin-widgets-api";
import type { PresetDef } from "./types";

// ── State ──────────────────────────────────────────────
// Typing runs in the widget without the app: the entry is widget state and each key's
// `set` appends to it (`"{{state.c_d}}7"` is resolved when the key is drawn). Only keys
// that compute (`=`, a chained operator, %, ±, ⌫) go to the app, which writes the result
// back with `setWidgetState`.
//   c_d  entry being typed ("" = none)     c_dot  entry has a "."
//   c_r  value shown when nothing is typed (result / first operand)
//   c_a  first operand                      c_op   operator symbol (+ − × ÷)
let history: string[] = [];

const CLEAR = { c_d: null, c_dot: null, c_a: null, c_op: null, c_r: null };

function calc(a: number, b: number, op: string): number {
  switch (op) {
    case "+": return a + b;
    case "\u2212": return a - b;
    case "\u00D7": return a * b;
    case "\u00F7": return b !== 0 ? a / b : NaN;
    default: return b;
  }
}

function formatResult(n: number): string {
  if (!isFinite(n)) return "Error";
  const s = parseFloat(n.toFixed(10)).toString();
  return s.length > 12 ? n.toExponential(5) : s;
}

const num = (s: string | undefined) => (s ? parseFloat(s) : 0);

/** `calc:<key>|<state fields>` from the keys below; returns the state patch to write. */
function handleKey(key: string, args: string[]): Record<string, unknown> | null {
  switch (key) {
    case "=": {
      const [a, op, b] = args;
      const r = formatResult(calc(num(a), num(b), op));
      history = [`${formatResult(num(a))} ${op} ${formatResult(num(b))} = ${r}`, ...history].slice(0, 5);
      return { ...CLEAR, c_r: r };
    }
    case "op": {
      // Chained `a op b op2`: compute, keep the result as the next first operand.
      const [next, a, op, b] = args;
      const r = formatResult(calc(num(a), num(b), op));
      return { c_d: null, c_dot: null, c_a: r, c_r: r, c_op: next };
    }
    case "BS": {
      const d = (args[0] ?? "").slice(0, -1);
      return { c_d: d === "-" ? null : d || null, c_dot: d.includes(".") || null };
    }
    case "%":
      return { c_d: null, c_dot: null, c_r: formatResult(num(args[0]) / 100) };
    case "+-": {
      const [d, r] = args;
      const neg = (v: string) => (v.startsWith("-") ? v.slice(1) : "-" + v);
      return d ? { c_d: neg(d) } : r && r !== "0" ? { c_r: neg(r) } : null;
    }
    case "AC":
      history = [];
      return null;
  }
  return null;
}

// ── UI helpers ─────────────────────────────────────────
const BG: ColorValue = { light: "#eef2ff", dark: "#1c1c1e" };
const BG_NUM: ColorValue = { light: "#ffffff", dark: "#3a3a3c" };
const BG_OP: ColorValue = { light: "#f59e0b", dark: "#f09a36" };
const BG_FN: ColorValue = { light: "#d1d5db", dark: "#636366" };
const FG: ColorValue = { light: "#0f172a", dark: "#ffffff" };
const FG_SUB: ColorValue = { light: "#475569", dark: "#8e8e93" };
const FG_OP: ColorValue = { light: "#b45309", dark: "#f09a36" };
const DIVIDER: ColorValue = { light: "#cbd5e1", dark: "#3a3a3c" };

type Pad = number | { top?: number; bottom?: number; leading?: number; trailing?: number };
type Key = Partial<Pick<Extract<WidgetElement, { type: "button" }>, "set" | "action" | "when">>;

/** A key that does nothing right now still needs a tap target: in WidgetKit a button with no
 *  `set` / `action` is not interactive, so the tap falls through and opens the app. Removing
 *  a key that never exists is a no-op state write. */
const INERT: Key = { set: { c_noop: null } };

function btn(label: string, bg: ColorValue, fg: ColorValue, fs: number, pad: Pad, key: Key = {}): WidgetElement {
  return { type: "button", label, backgroundColor: bg, color: fg, fontSize: fs, cornerRadius: 6, padding: pad, ...key };
}

const EMPTY = { path: "state.c_d", eq: "" };
const TYPED = { not: EMPTY };
/** An operator is waiting for its second operand, and it is typed. */
const PENDING = { all: [{ path: "state.c_op", exists: true }, TYPED] };

/** Keys with variants: exactly one variant of each key passes its `when`, so the grid
 *  always has 20 cells. */
function digit(k: string, fs: number, pad: Pad): WidgetElement[] {
  const append = btn(k, BG_NUM, FG, fs, pad, { set: { c_d: `{{state.c_d}}${k}`, c_r: null } });
  if (k !== "0") return [append];
  // A leading 0 types nothing (it only clears a shown result).
  return [btn(k, BG_NUM, FG, fs, pad, { set: { c_r: null }, when: EMPTY }), { ...append, when: TYPED } as WidgetElement];
}

function dot(fs: number, pad: Pad): WidgetElement[] {
  const noDot = { not: { path: "state.c_dot" } };
  return [
    btn(".", BG_NUM, FG, fs, pad, { ...INERT, when: { path: "state.c_dot" } }),
    btn(".", BG_NUM, FG, fs, pad, { set: { c_d: "0.", c_dot: true, c_r: null }, when: { all: [noDot, EMPTY] } }),
    btn(".", BG_NUM, FG, fs, pad, { set: { c_d: "{{state.c_d}}.", c_dot: true, c_r: null }, when: { all: [noDot, TYPED] } }),
  ];
}

function operator(sym: string, fs: number, pad: Pad): WidgetElement[] {
  // Picked and nothing typed yet: drawn inverted.
  const active = { all: [{ path: "state.c_op", eq: sym }, EMPTY] };
  const pick = { set: { c_op: sym, c_a: "{{state.c_d}}{{state.c_r}}", c_r: "{{state.c_d}}{{state.c_r}}", c_d: null, c_dot: null } };
  return [
    btn(sym, BG_OP, FG, fs, pad, { action: `calc:op|${sym}|{{state.c_a}}|{{state.c_op}}|{{state.c_d}}`, when: PENDING }),
    btn(sym, FG, BG_OP, fs, pad, { ...pick, when: active }),
    btn(sym, BG_OP, FG, fs, pad, { ...pick, when: { not: { any: [PENDING, active] } } }),
  ];
}

function keypad(fsNum: number, fsSym: number, pad: Pad = 2): WidgetElement[] {
  const d = (k: string) => digit(k, fsNum, pad);
  return [
    btn("C", BG_FN, FG, fsSym, pad, { set: CLEAR }),
    btn("\u00B1", BG_FN, FG, fsSym, pad, { action: "calc:+-|{{state.c_d}}|{{state.c_r}}" }),
    btn("\u232B", BG_FN, FG, fsSym, pad, { ...INERT, when: EMPTY }),
    btn("\u232B", BG_FN, FG, fsSym, pad, { action: "calc:BS|{{state.c_d}}", when: TYPED }),
    ...operator("\u00F7", fsSym, pad),
    ...d("7"), ...d("8"), ...d("9"), ...operator("\u00D7", fsSym, pad),
    ...d("4"), ...d("5"), ...d("6"), ...operator("\u2212", fsSym, pad),
    ...d("1"), ...d("2"), ...d("3"), ...operator("+", fsSym, pad),
    btn("%", BG_FN, FG, fsSym, pad, { action: "calc:%|{{state.c_d}}{{state.c_r}}" }),
    ...d("0"), ...dot(fsNum, pad),
    btn("=", BG_OP, FG, fsSym, pad, { action: "calc:=|{{state.c_a}}|{{state.c_op}}|{{state.c_d}}", when: PENDING }),
    btn("=", BG_OP, FG, fsSym, pad, { ...INERT, when: { not: PENDING } }),
  ];
}

/** Entry, else the shown value, else 0 (`lineLimit` cuts long ones). */
function display(fontSize: number): WidgetElement[] {
  const shown = { any: [TYPED, { path: "state.c_r", exists: true }] };
  const base = { type: "text", fontSize, fontWeight: "bold", color: FG, alignment: "trailing", lineLimit: 1 } as const;
  return [
    { ...base, content: "{{state.c_d}}{{state.c_r}}", when: shown },
    { ...base, content: "0", when: { not: shown } },
  ];
}

const opHint = (fontSize: number): WidgetElement =>
  ({ type: "text", content: " {{state.c_op}}", fontSize, color: FG_OP, alignment: "leading", lineLimit: 1 });

// ── Builder ────────────────────────────────────────────
function buildCalculatorConfig(): WidgetConfig {
  // systemSmall ≈ systemMedium height on macOS (~158) — keep vertical chrome tiny so 5 keypad rows fit.
  const compactPad = { top: 1, bottom: 1, leading: 2, trailing: 2 };

  const small: WidgetElement = {
    type: "vstack", padding: 6, spacing: 2, cornerRadius: 14, background: BG,
    children: [
      { type: "hstack", spacing: 4, alignment: "center", children: [opHint(10), { type: "spacer" }, ...display(20)] },
      { type: "divider", color: DIVIDER },
      { type: "grid", columns: 4, spacing: 2, rowSpacing: 2, children: keypad(11, 10, compactPad) },
    ],
  };

  const medium: WidgetElement = {
    type: "vstack", padding: 8, spacing: 3, cornerRadius: 14, background: BG,
    children: [
      { type: "hstack", spacing: 6, alignment: "center", children: [opHint(11), { type: "spacer" }, ...display(24)] },
      { type: "divider", color: DIVIDER },
      { type: "grid", columns: 4, spacing: 3, rowSpacing: 2, children: keypad(13, 12, compactPad) },
    ],
  };

  const large: WidgetElement = {
    type: "vstack", padding: 14, spacing: 8, cornerRadius: 16, background: BG,
    children: [
      { type: "hstack", spacing: 8, children: [
        { type: "text", content: "\u{1F9EE}", fontSize: 14 },
        { type: "text", content: "Calculator", fontSize: 14, fontWeight: "semibold", color: FG_SUB },
        { type: "spacer" },
        btn("AC", BG_FN, FG, 11, 4, { set: CLEAR, action: "calc:AC" }),
      ] },
      { ...opHint(16), alignment: "trailing" } as WidgetElement,
      ...display(42),
      { type: "divider", color: DIVIDER },
      { type: "grid", columns: 4, spacing: 5, rowSpacing: 5, children: keypad(18, 16, { top: 4, bottom: 4, leading: 6, trailing: 6 }) },
      { type: "divider", color: DIVIDER },
      { type: "text", content: "History", fontSize: 11, fontWeight: "semibold", color: FG_SUB },
      ...(history.length > 0
        ? history.slice(0, 3).map((h): WidgetElement => ({ type: "text", content: h, fontSize: 12, color: FG_SUB, alignment: "trailing", lineLimit: 1 }))
        : [{ type: "text", content: "No calculations yet", fontSize: 12, color: FG_SUB, alignment: "center" } as WidgetElement]),
    ],
  };

  return { state: { c_d: "" }, small, medium, large };
}

export const calculator: PresetDef = {
  icon: "\u{1F9EE}",
  name: "Calculator",
  config: buildCalculatorConfig(),
  // No polling: typing is in-process; computing keys rebuild through `onAction`.
  builder: buildCalculatorConfig,
  async onAction(action, _payload, log, setState) {
    if (!action.startsWith("calc:")) return;
    const [key, ...args] = action.slice(5).split("|");
    const patch = handleKey(key, args);
    if (patch) await setState(patch);
    log(`[${key}] ${args.join(" ")} \u2192 ${JSON.stringify(patch)}`);
  },
};
