import type { WidgetConfig, WidgetElement } from "tauri-plugin-widgets-api";
import type { PresetDef } from "./types";

interface Quote {
  text: string;
  textLong: string;
  author: string;
  source: string;
}

const QUOTES: Quote[] = [
  {
    text: "The only way to do great work is to love what you do.",
    textLong: "The only way to do great work is to love what you do. If you haven't found it yet, keep looking. Don't settle.",
    author: "Steve Jobs",
    source: "Stanford Commencement, 2005",
  },
  {
    text: "In the middle of difficulty lies opportunity.",
    textLong: "In the middle of every difficulty lies opportunity. The important thing is not to stop questioning.",
    author: "Albert Einstein",
    source: "Letter to a friend, 1952",
  },
  {
    text: "Simplicity is the ultimate sophistication.",
    textLong: "Simplicity is the ultimate sophistication. It takes a lot of hard work to make something simple.",
    author: "Leonardo da Vinci",
    source: "Notebooks, c. 1500",
  },
  {
    text: "The best time to plant a tree was 20 years ago. The second best time is now.",
    textLong: "The best time to plant a tree was 20 years ago. The second best time is now. Start where you are, use what you have, do what you can.",
    author: "Chinese Proverb",
    source: "Traditional",
  },
  {
    text: "Code is like humor. When you have to explain it, it's bad.",
    textLong: "Code is like humor. When you have to explain it, it's bad. Write code that speaks for itself.",
    author: "Cory House",
    source: "Programming Wisdom",
  },
];

let quoteIdx = 0;

function buildQuoteConfig(): WidgetConfig {
  const q = QUOTES[quoteIdx];
  const counter = `${quoteIdx + 1} / ${QUOTES.length}`;

  const small: WidgetElement = {
    type: "vstack", padding: 16, spacing: 10, cornerRadius: 16,
    background: { light: "#fef3c7", dark: "#422006" },
    children: [
      { type: "image", systemName: "quote.opening", color: "#d97706", size: 20 },
      { type: "text", content: q.text, textStyle: "subheadline", fontWeight: "medium",
        color: { light: "#78350f", dark: "#fde68a" }, alignment: "leading", lineLimit: 4 },
      { type: "spacer" },
      { type: "text", content: `\u2014 ${q.author}`, textStyle: "caption", fontWeight: "semibold",
        color: { light: "#92400e", dark: "#fbbf24" }, alignment: "trailing" },
    ],
  };

  const medium: WidgetElement = {
    type: "hstack", padding: 16, spacing: 16, cornerRadius: 16,
    background: { light: "#fef3c7", dark: "#422006" },
    children: [
      { type: "vstack", spacing: 8, alignment: "leading", flex: 1, children: [
        { type: "image", systemName: "quote.opening", color: "#d97706", size: 28 },
        { type: "text", content: q.text, textStyle: "body", fontWeight: "medium",
          color: { light: "#78350f", dark: "#fde68a" }, alignment: "leading" },
        { type: "text", content: `\u2014 ${q.author}`, textStyle: "footnote", fontWeight: "semibold",
          color: { light: "#92400e", dark: "#fbbf24" }, alignment: "trailing" },
      ] },
      { type: "vstack", spacing: 6, alignment: "center", children: [
        { type: "spacer" },
        { type: "text", content: counter, textStyle: "caption2", color: { light: "#b45309", dark: "#d97706" } },
        { type: "button", label: "Next", action: "next_quote",
          backgroundColor: "#d97706", color: "#ffffff", fontSize: 12, cornerRadius: 8 },
      ] },
    ],
  };

  const large: WidgetElement = {
    type: "vstack", padding: 20, spacing: 14, cornerRadius: 16,
    background: { light: "#fef3c7", dark: "#422006" },
    children: [
      { type: "image", systemName: "quote.opening", color: "#d97706", size: 36 },
      { type: "text", content: q.textLong, textStyle: "title3", fontWeight: "medium",
        color: { light: "#78350f", dark: "#fde68a" }, alignment: "leading" },
      { type: "spacer" },
      { type: "divider", color: { light: "#e5c07b", dark: "#854d0e" } },
      { type: "hstack", spacing: 12, children: [
        { type: "text", content: `\u2014 ${q.author}`, textStyle: "subheadline", fontWeight: "bold",
          color: { light: "#92400e", dark: "#fbbf24" } },
        { type: "text", content: q.source, textStyle: "caption", color: { light: "#b45309", dark: "#d97706" } },
        { type: "spacer" },
        { type: "text", content: counter, textStyle: "caption2", color: { light: "#b45309", dark: "#d97706" } },
        { type: "button", label: "Next Quote", action: "next_quote",
          backgroundColor: "#d97706", color: "#ffffff", fontSize: 12, cornerRadius: 8 },
      ] },
    ],
  };

  return { small, medium, large };
}

export const quote: PresetDef = {
  icon: "\u{1F4AC}",
  name: "Quote",
  config: buildQuoteConfig(),
  builder: buildQuoteConfig,
  intervalMs: 30000,
  onAction(action, _payload, log) {
    if (action === "next_quote") {
      quoteIdx = (quoteIdx + 1) % QUOTES.length;
      log(`Quote ${quoteIdx + 1}/${QUOTES.length}: ${QUOTES[quoteIdx].author}`);
    }
  },
};
