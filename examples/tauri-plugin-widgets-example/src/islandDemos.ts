// Live-activity (island) demos for the playground. Each demo is a pure `frame(t)` of the
// elapsed milliseconds; the runner pushes a new frame every `tickMs` (no queueing: a tick
// is skipped while the previous update is still in flight), so the island animates at the
// rate a real app would push it.
import type { ActivityContent, WidgetConfig, WidgetElement } from "tauri-plugin-widgets-api";

export type IslandDemo = {
  name: string;
  icon: string;
  /** Activity id (unique per demo, so several can run at once). */
  id: string;
  tickMs: number;
  /** Stop after this many ms (`0` = until stopped). */
  durationMs: number;
  frame(t: number): ActivityContent;
};

/** `widget-action` from an island tap carries `widgetId = "activity.<id>"`. */
export const activityWidgetId = (id: string) => `activity.${id}`;

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));
const wave = (t: number, phase: number) =>
  50 + 28 * Math.sin(t / 900 + phase) + 14 * Math.sin(t / 380 + phase * 2) + 6 * Math.sin(t / 130);

const text = (content: string, fontSize: number, extra: Partial<WidgetElement> = {}): WidgetElement =>
  ({ type: "text", content, fontSize, ...extra }) as WidgetElement;

// ── Download: smooth progress with a varying speed ─────────────────────────────────────
const DL_MS = 9000;
function dlProgress(t: number) {
  // Integral of a wobbly speed, normalised so it reaches 1 at DL_MS.
  const x = clamp(t / DL_MS, 0, 1);
  return clamp(x + 0.04 * Math.sin(x * Math.PI * 6) * (1 - x), 0, 1);
}
const download: IslandDemo = {
  name: "Download",
  icon: "⬇️",
  id: "demo-download",
  tickMs: 60,
  durationMs: DL_MS + 1500,
  frame(t) {
    const p = dlProgress(t);
    const done = p >= 1;
    const speed = done ? 0 : Math.round(40 + 25 * Math.sin(t / 300) + 10 * Math.sin(t / 77));
    const left = Math.max(0, Math.ceil(((1 - p) * DL_MS) / 1000));
    const tint = done ? "#4ade80" : "#60a5fa";
    return {
      id: this.id,
      title: done ? "Downloaded" : "Downloading",
      body: "macOS-27.dmg",
      progress: p,
      tint,
      timestamp: Date.now(),
      compactLeading: { type: "gauge", value: p, gaugeStyle: "circular", tint, frame: { width: 18, height: 18 } },
      compactTrailing: text(`${Math.round(p * 100)}%`, 12, { fontWeight: "semibold", color: tint }),
      minimal: { type: "gauge", value: p, gaugeStyle: "circular", tint },
      islandLeading: text("macOS-27.dmg", 13, { fontWeight: "semibold" }),
      islandTrailing: text(done ? "Done" : `${speed} MB/s`, 12, { color: tint }),
      layout: {
        type: "vstack",
        spacing: 8,
        padding: { leading: 14, trailing: 14, bottom: 10 },
        children: [
          { type: "progress", value: p, tint, thickness: 6 },
          {
            type: "hstack",
            children: [
              text(`${(p * 4.2).toFixed(2)} / 4.20 GB`, 11, { color: "secondary" }),
              { type: "spacer" },
              text(done ? "complete" : `${left}s left`, 11, { color: "secondary" }),
            ],
          },
          {
            type: "hstack",
            spacing: 8,
            children: [
              { type: "button", label: "Show in Finder", action: "reveal", variant: "bordered", fontSize: 11 },
              { type: "spacer" },
              { type: "button", label: "Cancel", action: "cancel", variant: "plain", fontSize: 11, color: "#f87171" },
            ],
          },
        ],
      },
    };
  },
};

// ── Eyes: blink every ~2.6 s, pupils follow a wandering point ──────────────────────────
function eyeOpen(t: number) {
  const cycle = t % 2600;
  if (cycle > 2400) return 1 - Math.sin(((cycle - 2400) / 200) * Math.PI); // close → open in 200 ms
  return 1;
}
function eye(size: number, open: number, dx: number, dy: number): WidgetElement {
  const h = Math.max(1.5, size * open);
  return {
    type: "zstack",
    frame: { width: size, height: size },
    children: [
      { type: "shape", shapeType: "ellipse", fill: "#ffffff", frame: { width: size * 0.8, height: h } },
      ...(open > 0.35
        ? [
            {
              type: "shape",
              shapeType: "circle",
              fill: "#111827",
              frame: { width: size * 0.36, height: Math.min(size * 0.36, h) },
              offsetX: dx * size * 0.18,
              offsetY: dy * size * 0.12 * open,
            } as WidgetElement,
          ]
        : []),
    ],
  } as WidgetElement;
}
function eyes(t: number, size: number, gap: number): WidgetElement {
  const open = eyeOpen(t);
  const dx = Math.sin(t / 700);
  const dy = Math.sin(t / 1100);
  return { type: "hstack", spacing: gap, children: [eye(size, open, dx, dy), eye(size, open, dx, dy)] };
}
const blink: IslandDemo = {
  name: "Eyes",
  icon: "👀",
  id: "demo-eyes",
  tickMs: 50,
  durationMs: 0,
  frame(t) {
    return {
      id: this.id,
      title: "Watching",
      timestamp: Date.now(),
      tint: "#fbbf24",
      compactLeading: eyes(t, 14, 3),
      compactTrailing: text(eyeOpen(t) < 0.5 ? "blink" : "hi", 11, { color: "#fbbf24" }),
      minimal: eyes(t, 9, 1),
      islandCenter: text("I see you", 13, { fontWeight: "semibold" }),
      layout: {
        type: "hstack",
        padding: { bottom: 10 },
        children: [{ type: "spacer" }, eyes(t, 46, 18), { type: "spacer" }],
      },
    };
  },
};

// ── Monitor: scrolling charts ──────────────────────────────────────────────────────────
function series(t: number, n: number, stepMs: number, phase: number) {
  const now = Math.floor(t / stepMs);
  return Array.from({ length: n }, (_, i) => {
    const k = now - n + 1 + i;
    return { label: String(k), value: Math.round(clamp(wave(k * stepMs, phase), 2, 98)) };
  });
}
const monitor: IslandDemo = {
  name: "Monitor",
  icon: "📈",
  id: "demo-monitor",
  tickMs: 100,
  durationMs: 0,
  frame(t) {
    const cpu = Math.round(clamp(wave(t, 0), 2, 98));
    const net = Math.round(clamp(wave(t, 1.3), 2, 98));
    const hot = cpu > 75;
    const tint = hot ? "#f87171" : "#22d3ee";
    return {
      id: this.id,
      title: `CPU ${cpu}%`,
      body: `Net ${net} MB/s`,
      tint,
      timestamp: Date.now(),
      compactLeading: {
        type: "chart", chartType: "bar", chartData: series(t, 6, 300, 0), tint,
        showXAxis: false, showYAxis: false, yMin: 0, yMax: 100, frame: { width: 40, height: 18 },
      },
      compactTrailing: text(`${cpu}%`, 12, { fontWeight: "semibold", color: tint }),
      minimal: { type: "gauge", value: cpu / 100, gaugeStyle: "circular", tint },
      islandLeading: text(hot ? "CPU hot" : "System", 13, { fontWeight: "semibold", color: tint }),
      islandTrailing: text(`${cpu}%`, 14, { fontWeight: "bold", color: tint }),
      layout: {
        type: "hstack",
        spacing: 10,
        padding: { leading: 12, trailing: 12, bottom: 8 },
        children: [
          {
            type: "chart", chartType: "area", chartData: series(t, 30, 100, 0), tint,
            interpolation: "monotone", showXAxis: false, showYAxis: false, yMin: 0, yMax: 100,
            frame: { height: 48, maxWidth: "infinity" },
          },
          {
            type: "chart", chartType: "bar", chartData: series(t, 12, 200, 1.3), tint: "#a78bfa",
            showXAxis: false, showYAxis: false, yMin: 0, yMax: 100, frame: { width: 90, height: 48 },
          },
        ],
      },
    };
  },
};

// ── Music: bouncing equalizer + transport buttons ──────────────────────────────────────
const SONG_MS = 30000;
function eq(t: number, n: number) {
  return Array.from({ length: n }, (_, i) => ({
    label: String(i),
    value: Math.round(clamp(20 + 70 * Math.abs(Math.sin(t / (110 + i * 37) + i * 1.7)), 5, 100)),
  }));
}
const music: IslandDemo = {
  name: "Music",
  icon: "🎵",
  id: "demo-music",
  tickMs: 70,
  durationMs: 0,
  frame(t) {
    const pos = t % SONG_MS;
    const mm = (ms: number) => `${Math.floor(ms / 60000)}:${String(Math.floor(ms / 1000) % 60).padStart(2, "0")}`;
    const tint = "#f472b6";
    return {
      id: this.id,
      title: "Night Drive",
      body: "Synth Club",
      progress: pos / SONG_MS,
      tint,
      timestamp: Date.now(),
      compactLeading: { type: "image", systemName: "music.note", color: tint, size: 13 },
      compactTrailing: {
        type: "chart", chartType: "bar", chartData: eq(t, 5), tint,
        showXAxis: false, showYAxis: false, yMin: 0, yMax: 100, frame: { width: 26, height: 14 },
      },
      minimal: { type: "image", systemName: "music.note", color: tint, size: 12 },
      islandLeading: {
        type: "vstack", alignment: "leading", spacing: 0,
        children: [text("Night Drive", 13, { fontWeight: "semibold" }), text("Synth Club", 11, { color: "secondary" })],
      },
      islandTrailing: {
        type: "chart", chartType: "bar", chartData: eq(t, 7), tint,
        showXAxis: false, showYAxis: false, yMin: 0, yMax: 100, frame: { width: 46, height: 22 },
      },
      layout: {
        type: "vstack",
        spacing: 6,
        padding: { leading: 14, trailing: 14, bottom: 8 },
        children: [
          {
            type: "hstack", spacing: 8,
            children: [
              text(mm(pos), 10, { color: "secondary" }),
              { type: "progress", value: pos / SONG_MS, tint, thickness: 4, flex: 1 },
              text(mm(SONG_MS), 10, { color: "secondary" }),
            ],
          },
          {
            type: "hstack", spacing: 24,
            children: [
              { type: "spacer" },
              { type: "button", label: "", systemName: "backward.fill", action: "prev", variant: "plain" },
              { type: "button", label: "", systemName: "pause.fill", action: "pause", variant: "plain" },
              { type: "button", label: "", systemName: "forward.fill", action: "next", variant: "plain" },
              { type: "spacer" },
            ],
          },
        ],
      },
    };
  },
};

// ── Timer: one push, the native timer counts by itself ─────────────────────────────────
const timer: IslandDemo = {
  name: "Timer",
  icon: "⏱️",
  id: "demo-timer",
  tickMs: 0,
  durationMs: 0,
  frame() {
    return {
      id: this.id,
      title: "Tea",
      body: "Green tea, 3 min",
      icon: "timer",
      tint: "#34d399",
      endsAt: Date.now() + 3 * 60_000,
      timestamp: Date.now(),
    };
  },
};

// ── Launcher: tap the island to open a window of app icons under it ────────────────────
export const LAUNCHER_ID = "launcher";
export const LAUNCHER_LABEL = "island-launcher";
const launcherDemo: IslandDemo = {
  name: "Launcher",
  icon: "🚀",
  id: "demo-launcher",
  tickMs: 0,
  durationMs: 0,
  frame() {
    return {
      id: this.id,
      title: "Apps",
      tint: "#818cf8",
      timestamp: Date.now(),
      compactLeading: text("🚀", 12),
      compactTrailing: text("Apps", 11, { color: "#818cf8", fontWeight: "semibold" }),
      minimal: text("🚀", 11),
      islandCenter: text("Quick launch", 13, { fontWeight: "semibold" }),
      layout: {
        type: "hstack",
        padding: { bottom: 10 },
        children: [
          { type: "spacer" },
          { type: "button", label: "Open apps", systemName: "square.grid.2x2", action: "open-launcher", variant: "prominent" },
          { type: "spacer" },
        ],
      },
    };
  },
};

export const ISLAND_DEMOS: IslandDemo[] = [download, blink, monitor, music, timer, launcherDemo];

const APPS: Array<[string, string, string]> = [
  ["🧭", "Safari", "#0ea5e9"],
  ["✉️", "Mail", "#3b82f6"],
  ["🎵", "Music", "#f43f5e"],
  ["📷", "Photos", "#f59e0b"],
  ["🗺️", "Maps", "#22c55e"],
  ["📝", "Notes", "#eab308"],
  ["⚙️", "Settings", "#64748b"],
  ["💬", "Messages", "#10b981"],
];

/** Icon grid shown in a small window under the island; a tap sends `launch:<app>`. */
export function launcherConfig(): WidgetConfig {
  const tile = ([emoji, name, color]: [string, string, string]): WidgetElement => ({
    type: "link",
    action: `launch:${name}`,
    children: [
      {
        type: "vstack",
        spacing: 4,
        children: [
          {
            type: "zstack",
            frame: { width: 44, height: 44 },
            background: color,
            cornerRadius: 11,
            children: [text(emoji, 22)],
          },
          text(name, 10, { color: "secondary", lineLimit: 1 }),
        ],
      },
    ],
  });
  return {
    medium: {
      type: "grid",
      columns: 4,
      spacing: 10,
      rowSpacing: 10,
      padding: 12,
      background: "#0b0b0f",
      cornerRadius: 18,
      frame: { maxWidth: "infinity", maxHeight: "infinity" },
      children: APPS.map(tile),
    },
  };
}
