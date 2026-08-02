import type { PresetDef } from "./types";
import type { WidgetConfig, WidgetElement } from "tauri-plugin-widgets-api";

/** Snapshot-like item (mirrors Subly upcoming.json fields). */
export type UpcomingItem = {
  id: string;
  name: string;
  /** YYYY-MM-DD */
  nextPayment: string;
  price: string | number;
  currencyCode?: string;
  /** Optional base64 PNG (without data: prefix). */
  iconData?: string;
};

const ACCENT = "#3878FA";
const ACCENT_DEEP = "#1F59E0";
const BG = { light: "#F2F2F7", dark: "#1C1C1E" } as const;

const DEMO_ITEMS: UpcomingItem[] = [
  { id: "1", name: "Streaming", nextPayment: "2099-06-01", price: "12.50", currencyCode: "USD" },
  { id: "2", name: "Cloud Pro", nextPayment: offsetDate(0), price: "9.99", currencyCode: "USD" },
  { id: "3", name: "Music Plus", nextPayment: offsetDate(1), price: "4.99", currencyCode: "USD" },
  { id: "4", name: "News Daily", nextPayment: "2099-06-05", price: "2.99", currencyCode: "USD" },
  { id: "5", name: "Gym Club", nextPayment: "2099-06-08", price: "29.00", currencyCode: "USD" },
  { id: "6", name: "VPN Secure", nextPayment: "2099-06-12", price: "6.49", currencyCode: "USD" },
  { id: "7", name: "Storage XL", nextPayment: "2099-06-15", price: "1.99", currencyCode: "USD" },
  { id: "8", name: "Design Kit", nextPayment: "2099-06-18", price: "14.00", currencyCode: "USD" },
  { id: "9", name: "Team Chat", nextPayment: "2099-06-20", price: "7.50", currencyCode: "USD" },
  { id: "10", name: "Analytics", nextPayment: "2099-06-22", price: "19.00", currencyCode: "USD" },
];

function offsetDate(daysFromToday: number): string {
  const d = new Date();
  d.setHours(12, 0, 0, 0);
  d.setDate(d.getDate() + daysFromToday);
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

function clipName(raw: string, max = 200): string {
  const s = raw.replace(/[\n\t]+/g, " ").trim();
  if (s.length <= max) return s;
  return s.slice(0, max) + "…";
}

function letterGlyph(name: string): string {
  const ch = clipName(name).charAt(0);
  return ch ? ch.toUpperCase() : "?";
}

function formatCurrency(amount: string | number, currencyCode?: string): string {
  const n = typeof amount === "number" ? amount : Number.parseFloat(String(amount));
  const code = (currencyCode || "USD").trim().toUpperCase() || "USD";
  if (!Number.isFinite(n)) return String(amount);
  try {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency: code.length === 3 ? code : "USD",
      minimumFractionDigits: Math.abs(n % 1) < 0.001 ? 0 : 2,
      maximumFractionDigits: 2,
    }).format(n);
  } catch {
    return `$${n.toFixed(2)}`;
  }
}

function formatRelativeDate(isoDate: string, now = new Date()): string {
  const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(isoDate.trim());
  if (!m) return isoDate;
  const target = new Date(Number(m[1]), Number(m[2]) - 1, Number(m[3]), 12, 0, 0, 0);
  const today = new Date(now);
  today.setHours(12, 0, 0, 0);
  const diffDays = Math.round((target.getTime() - today.getTime()) / 86_400_000);
  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Tomorrow";
  return target.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}

function avatar(letter: string, size: number, iconData?: string): WidgetElement {
  if (iconData) {
    return {
      type: "image",
      data: iconData,
      size,
      contentMode: "fill",
      cornerRadius: Math.round(size * 0.22),
      clipShape: "rectangle",
    };
  }
  return {
    type: "zstack",
    alignment: "center",
    children: [
      {
        type: "shape",
        shapeType: "rectangle",
        fill: ACCENT_DEEP,
        size,
        cornerRadius: Math.round(size * 0.22),
      },
      {
        type: "text",
        content: letter,
        fontSize: Math.max(8, Math.round(size * 0.42)),
        fontWeight: "bold",
        fontDesign: "rounded",
        color: "#FFFFFF",
      },
    ],
  };
}

function compactRow(item: UpcomingItem, dense: boolean): WidgetElement {
  const size = dense ? 20 : 24;
  const nameSize = dense ? 10 : 11;
  const dateSize = dense ? 8 : 9;
  const priceSize = dense ? 10 : 11;
  return {
    type: "hstack",
    spacing: 6,
    alignment: "center",
    children: [
      avatar(letterGlyph(item.name), size, item.iconData),
      {
        type: "vstack",
        spacing: 0,
        alignment: "leading",
        flex: 1,
        children: [
          {
            type: "text",
            content: clipName(item.name),
            fontSize: nameSize,
            fontWeight: "semibold",
            fontDesign: "rounded",
            color: "label",
            lineLimit: 1,
          },
          {
            type: "text",
            content: formatRelativeDate(item.nextPayment),
            fontSize: dateSize,
            color: "secondaryLabel",
            lineLimit: 1,
          },
        ],
      },
      {
        type: "text",
        content: formatCurrency(item.price, item.currencyCode),
        fontSize: priceSize,
        fontWeight: "bold",
        fontDesign: "rounded",
        color: ACCENT,
        lineLimit: 1,
      },
    ],
  };
}

function emptyBody(size: "small" | "medium" | "large"): WidgetElement {
  const icon = size === "small" ? 48 : size === "medium" ? 40 : 52;
  const pad = size === "large" ? 14 : size === "medium" ? 12 : 14;
  const children: WidgetElement[] = [];
  if (size !== "small") {
    children.push({
      type: "label",
      text: "Upcoming",
      systemName: "calendar.badge.clock",
      fontSize: 11,
      fontWeight: "bold",
      color: "label",
      iconColor: ACCENT,
      spacing: 4,
    });
  }
  children.push(
    { type: "spacer" },
    {
      type: "zstack",
      alignment: "center",
      children: [
        { type: "shape", shapeType: "circle", fill: "#3878FA", size: icon },
        { type: "image", systemName: "checkmark.circle.fill", size: Math.round(icon * 0.58), color: ACCENT },
      ],
    },
    {
      type: "text",
      content: "No upcoming payments",
      fontSize: size === "large" ? 14 : size === "medium" ? 12 : 13,
      fontWeight: "semibold",
      fontDesign: "rounded",
      color: "secondaryLabel",
      alignment: "center",
      lineLimit: 2,
    },
    { type: "spacer" },
  );
  return {
    type: "vstack",
    padding: pad,
    spacing: size === "large" ? 10 : 8,
    cornerRadius: 16,
    background: BG,
    children,
  };
}

function smallHero(item: UpcomingItem): WidgetElement {
  return {
    type: "vstack",
    padding: 14,
    spacing: 10,
    cornerRadius: 16,
    background: BG,
    children: [
      { type: "divider", color: ACCENT, thickness: 4 },
      {
        type: "hstack",
        spacing: 10,
        alignment: "center",
        children: [
          avatar(letterGlyph(item.name), 44, item.iconData),
          {
            type: "vstack",
            spacing: 2,
            alignment: "leading",
            flex: 1,
            children: [
              {
                type: "text",
                content: clipName(item.name),
                fontSize: 15,
                fontWeight: "bold",
                fontDesign: "rounded",
                color: "label",
                lineLimit: 1,
              },
              {
                type: "text",
                content: formatRelativeDate(item.nextPayment),
                fontSize: 11,
                fontWeight: "semibold",
                color: "secondaryLabel",
                lineLimit: 1,
              },
            ],
          },
        ],
      },
      { type: "spacer" },
      {
        type: "text",
        content: formatCurrency(item.price, item.currencyCode),
        fontSize: 22,
        fontWeight: "bold",
        fontDesign: "rounded",
        color: ACCENT,
        alignment: "leading",
        lineLimit: 1,
      },
    ],
  };
}

function listFamily(items: UpcomingItem[], size: "medium" | "large"): WidgetElement {
  const dense = size === "medium";
  const cap = size === "medium" ? 4 : 6;
  const visible = items.slice(0, cap);
  const rest = Math.max(0, items.length - visible.length);
  const children: WidgetElement[] = [
    {
      type: "label",
      text: "Upcoming",
      systemName: "calendar.badge.clock",
      fontSize: 11,
      fontWeight: "bold",
      color: "label",
      iconColor: ACCENT,
      spacing: 4,
    },
    {
      type: "grid",
      columns: 2,
      spacing: dense ? 8 : 10,
      rowSpacing: dense ? 6 : 8,
      children: visible.map((it) => compactRow(it, dense)),
    },
  ];
  if (rest > 0) {
    children.push(
      { type: "spacer", minLength: dense ? 2 : 4 },
      {
        type: "text",
        content: `+${rest} more`,
        fontSize: 9,
        fontWeight: "bold",
        color: "secondaryLabel",
        lineLimit: 1,
      },
    );
  }
  return {
    type: "vstack",
    padding: dense ? 10 : 12,
    spacing: 4,
    cornerRadius: 16,
    background: BG,
    children,
  };
}

/** Build Subly-like Upcoming payments WidgetConfig from snapshot items. */
export function buildUpcomingPayments(items: UpcomingItem[]): WidgetConfig {
  if (items.length === 0) {
    return {
      version: 1,
      small: emptyBody("small"),
      medium: emptyBody("medium"),
      large: emptyBody("large"),
    };
  }
  return {
    version: 1,
    small: smallHero(items[0]),
    medium: listFamily(items, "medium"),
    large: listFamily(items, "large"),
  };
}

export const upcomingPayments: PresetDef = {
  icon: "💳",
  name: "Upcoming Payments",
  config: buildUpcomingPayments(DEMO_ITEMS),
  builder: () => buildUpcomingPayments(DEMO_ITEMS),
};
