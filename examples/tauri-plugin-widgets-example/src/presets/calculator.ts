import type { ColorValue, WidgetConfig, WidgetElement } from "tauri-plugin-widgets-api";
import type { PresetDef } from "./types";

// ── State ──────────────────────────────────────────────
let display = "0";
let firstOperand: number | null = null;
let operator: string | null = null;
let waitingForOperand = false;
let history: string[] = [];

function reset() {
  display = "0";
  firstOperand = null;
  operator = null;
  waitingForOperand = false;
}

function inputDigit(digit: string) {
  if (display.replace(/[^0-9]/g, "").length >= 12 && !waitingForOperand) return;
  if (waitingForOperand) {
    display = digit;
    waitingForOperand = false;
  } else {
    display = display === "0" ? digit : display + digit;
  }
}

function inputDot() {
  if (waitingForOperand) {
    display = "0.";
    waitingForOperand = false;
    return;
  }
  if (!display.includes(".")) display += ".";
}

function calc(a: number, b: number, op: string): number {
  switch (op) {
    case "+": return a + b;
    case "-": return a - b;
    case "*": return a * b;
    case "/": return b !== 0 ? a / b : NaN;
    default: return b;
  }
}

const OP_SYMBOL: Record<string, string> = { "+": "+", "-": "−", "*": "×", "/": "÷" };

function handleOperator(nextOp: string) {
  const current = parseFloat(display);
  if (operator && !waitingForOperand) {
    const result = calc(firstOperand!, current, operator);
    display = formatResult(result);
    firstOperand = result;
  } else {
    firstOperand = current;
  }
  operator = nextOp;
  waitingForOperand = true;
}

function handleEquals() {
  if (operator == null || firstOperand == null) return;
  const current = parseFloat(display);
  const result = calc(firstOperand, current, operator);
  const expr = `${formatResult(firstOperand)} ${OP_SYMBOL[operator] ?? operator} ${formatResult(current)} = ${formatResult(result)}`;
  history = [expr, ...history].slice(0, 5);
  display = formatResult(result);
  firstOperand = null;
  operator = null;
  waitingForOperand = true;
}

function formatResult(n: number): string {
  if (!isFinite(n)) return "Error";
  const s = parseFloat(n.toFixed(10)).toString();
  return s.length > 12 ? n.toExponential(5) : s;
}

function handleKey(key: string) {
  if (key >= "0" && key <= "9") return inputDigit(key);
  switch (key) {
    case ".": return inputDot();
    case "+": case "-": case "*": case "/": return handleOperator(key);
    case "=": return handleEquals();
    case "C": return reset();
    case "AC": history = []; return reset();
    case "BS":
      if (!waitingForOperand && display.length > 1) display = display.slice(0, -1);
      else { display = "0"; waitingForOperand = false; }
      return;
    case "%":
      display = formatResult(parseFloat(display) / 100);
      return;
    case "+-":
      if (display !== "0") display = display.startsWith("-") ? display.slice(1) : "-" + display;
      return;
  }
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

function btn(
  label: string,
  key: string,
  bg: ColorValue,
  fg: ColorValue,
  fs: number,
  pad: number | { top?: number; bottom?: number; leading?: number; trailing?: number } = 2,
): WidgetElement {
  const isActive = ["+", "-", "*", "/"].includes(key) && operator === key && waitingForOperand;
  return {
    type: "button", label,
    action: `calc:${key}`,
    backgroundColor: isActive ? fg : bg,
    color: isActive ? bg : fg,
    fontSize: fs, cornerRadius: 6,
    padding: pad,
  };
}

function keypad(
  fsNum: number,
  fsSym: number,
  pad: number | { top?: number; bottom?: number; leading?: number; trailing?: number } = 2,
): WidgetElement[] {
  return [
    btn("C", "C", BG_FN, FG, fsSym, pad), btn("±", "+-", BG_FN, FG, fsSym, pad), btn("⌫", "BS", BG_FN, FG, fsSym, pad), btn("÷", "/", BG_OP, FG, fsSym, pad),
    btn("7", "7", BG_NUM, FG, fsNum, pad), btn("8", "8", BG_NUM, FG, fsNum, pad), btn("9", "9", BG_NUM, FG, fsNum, pad), btn("×", "*", BG_OP, FG, fsSym, pad),
    btn("4", "4", BG_NUM, FG, fsNum, pad), btn("5", "5", BG_NUM, FG, fsNum, pad), btn("6", "6", BG_NUM, FG, fsNum, pad), btn("−", "-", BG_OP, FG, fsSym, pad),
    btn("1", "1", BG_NUM, FG, fsNum, pad), btn("2", "2", BG_NUM, FG, fsNum, pad), btn("3", "3", BG_NUM, FG, fsNum, pad), btn("+", "+", BG_OP, FG, fsSym, pad),
    btn("%", "%", BG_FN, FG, fsSym, pad), btn("0", "0", BG_NUM, FG, fsNum, pad), btn(".", ".", BG_NUM, FG, fsNum, pad), btn("=", "=", BG_OP, FG, fsSym, pad),
  ];
}

// ── Builder ────────────────────────────────────────────
function buildCalculatorConfig(): WidgetConfig {
  const dsp = display.length > 10 ? display.slice(0, 10) + "\u2026" : display;
  const opHint = operator && waitingForOperand ? ` ${OP_SYMBOL[operator] ?? operator}` : "";

  // systemSmall ≈ systemMedium height on macOS (~158) — keep vertical chrome tiny so 5 keypad rows fit.
  const compactPad = { top: 1, bottom: 1, leading: 2, trailing: 2 };

  const small: WidgetElement = {
    type: "vstack", padding: 6, spacing: 2, cornerRadius: 14, background: BG,
    children: [
      { type: "hstack", spacing: 4, alignment: "center", children: [
        { type: "text", content: opHint.trim() || " ", fontSize: 10, color: FG_OP, alignment: "leading", lineLimit: 1 },
        { type: "spacer" },
        { type: "text", content: dsp, fontSize: 20, fontWeight: "bold", color: FG, alignment: "trailing", lineLimit: 1 },
      ] },
      { type: "divider", color: DIVIDER },
      { type: "grid", columns: 4, spacing: 2, rowSpacing: 2, children: [
        ...keypad(11, 10, compactPad),
      ] },
    ],
  };

  const medium: WidgetElement = {
    type: "vstack", padding: 8, spacing: 3, cornerRadius: 14, background: BG,
    children: [
      { type: "hstack", spacing: 6, alignment: "center", children: [
        { type: "text", content: opHint.trim() || " ", fontSize: 11, color: FG_OP, alignment: "leading", lineLimit: 1 },
        { type: "spacer" },
        { type: "text", content: display.length > 12 ? display.slice(0, 12) + "\u2026" : display, fontSize: 24, fontWeight: "bold", color: FG, alignment: "trailing", lineLimit: 1 },
      ] },
      { type: "divider", color: DIVIDER },
      { type: "grid", columns: 4, spacing: 3, rowSpacing: 2, children: [
        ...keypad(13, 12, compactPad),
      ] },
    ],
  };

  const large: WidgetElement = {
    type: "vstack", padding: 14, spacing: 8, cornerRadius: 16, background: BG,
    children: [
      { type: "hstack", spacing: 8, children: [
        { type: "text", content: "\u{1F9EE}", fontSize: 14 },
        { type: "text", content: "Calculator", fontSize: 14, fontWeight: "semibold", color: FG_SUB },
        { type: "spacer" },
        btn("AC", "AC", BG_FN, FG, 11, 4),
      ] },
      { type: "text", content: opHint || " ", fontSize: 16, color: FG_OP, alignment: "trailing" },
      { type: "text", content: display.length > 14 ? display.slice(0, 14) + "\u2026" : display, fontSize: 42, fontWeight: "bold", color: FG, alignment: "trailing", lineLimit: 1 },
      { type: "divider", color: DIVIDER },
      { type: "grid", columns: 4, spacing: 5, rowSpacing: 5, children: [
        ...keypad(18, 16, { top: 4, bottom: 4, leading: 6, trailing: 6 }),
      ] },
      { type: "divider", color: DIVIDER },
      { type: "text", content: "History", fontSize: 11, fontWeight: "semibold", color: FG_SUB },
      ...(history.length > 0
        ? history.slice(0, 3).map((h): WidgetElement => ({ type: "text", content: h, fontSize: 12, color: FG_SUB, alignment: "trailing", lineLimit: 1 }))
        : [{ type: "text", content: "No calculations yet", fontSize: 12, color: FG_SUB, alignment: "center" } as WidgetElement]),
    ],
  };

  return { small, medium, large };
}

export const calculator: PresetDef = {
  icon: "\u{1F9EE}",
  name: "Calculator",
  config: buildCalculatorConfig(),
  builder: buildCalculatorConfig,
  intervalMs: 300,
  onAction(action, _payload, log) {
    if (!action.startsWith("calc:")) return;
    const key = action.slice(5);
    handleKey(key);
    log(`[${key}] \u2192 ${display}`);
  },
};
