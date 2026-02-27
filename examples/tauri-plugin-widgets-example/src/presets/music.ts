import type { WidgetConfig, WidgetElement } from "tauri-plugin-widgets-api";
import type { PresetDef } from "./types";

interface Track {
  title: string;
  artist: string;
  album: string;
  durationSec: number;
  durationStr: string;
}

const TRACKS: Track[] = [
  { title: "Bohemian Rhapsody", artist: "Queen", album: "A Night at the Opera (1975)", durationSec: 355, durationStr: "5:55" },
  { title: "Stairway to Heaven", artist: "Led Zeppelin", album: "Led Zeppelin IV (1971)", durationSec: 482, durationStr: "8:02" },
  { title: "Hotel California", artist: "Eagles", album: "Hotel California (1977)", durationSec: 390, durationStr: "6:30" },
  { title: "Comfortably Numb", artist: "Pink Floyd", album: "The Wall (1979)", durationSec: 382, durationStr: "6:22" },
];

let playing = true;
let trackIdx = 0;
let positionSec = 222;

function fmt(sec: number): string {
  const m = Math.floor(sec / 60);
  const s = sec % 60;
  return `${m}:${String(s).padStart(2, "0")}`;
}

function advance() {
  if (!playing) return;
  positionSec++;
  if (positionSec >= TRACKS[trackIdx].durationSec) {
    trackIdx = (trackIdx + 1) % TRACKS.length;
    positionSec = 0;
  }
}

function buildMusicConfig(): WidgetConfig {
  advance();
  const track = TRACKS[trackIdx];
  const progress = positionSec / track.durationSec;
  const posStr = fmt(positionSec);
  const playBtn = playing ? "\u23F8" : "\u25B6";
  const playBtnLabel = playing ? `${playBtn} Pause` : `${playBtn} Play`;
  const upcoming = [1, 2, 3].map((i) => TRACKS[(trackIdx + i) % TRACKS.length]);

  const small: WidgetElement = {
    type: "vstack", padding: 14, spacing: 8, cornerRadius: 16,
    background: { light: "#EEF2FF", dark: "#16213e" },
    children: [
      { type: "hstack", spacing: 10, children: [
        { type: "image", systemName: "music.note", color: "#ffffff", size: 20,
          clipShape: "circle",
          background: { gradientType: "linear", colors: ["#e94560", "#c62e46"], direction: "topToBottom" },
          frame: { width: 40, height: 40 } },
        { type: "vstack", spacing: 2, alignment: "leading", flex: 1, children: [
          { type: "text", content: track.title, textStyle: "subheadline", fontWeight: "semibold", color: "label", lineLimit: 1 },
          { type: "text", content: track.artist, textStyle: "caption", color: "secondaryLabel" },
        ] },
      ] },
      { type: "progress", value: progress, tint: "#e94560", label: `${posStr} / ${track.durationStr}`, color: "secondaryLabel" },
    ],
  };

  const medium: WidgetElement = {
    type: "hstack", padding: 14, spacing: 14, cornerRadius: 16,
    background: { light: "#EEF2FF", dark: "#16213e" },
    children: [
      { type: "image", systemName: "music.note.list", color: "#ffffff", size: 28,
        clipShape: "circle",
        background: { gradientType: "linear", colors: ["#e94560", "#c62e46"], direction: "topToBottom" },
        frame: { width: 56, height: 56 } },
      { type: "vstack", spacing: 6, alignment: "leading", flex: 1, children: [
        { type: "text", content: track.title, textStyle: "headline", color: "label", lineLimit: 1 },
        { type: "text", content: `${track.artist} \u2022 ${track.album}`, textStyle: "caption", color: "secondaryLabel" },
        { type: "progress", value: progress, tint: "#e94560", label: `${posStr} / ${track.durationStr}`, color: "secondaryLabel" },
        { type: "hstack", spacing: 20, children: [
          { type: "button", label: "\u23EA", action: "music_prev",
            backgroundColor: { light: "#E0E7FF", dark: "#1a2744" }, color: { light: "#3730a3", dark: "#e2e8f0" }, fontSize: 14, textAlignment: "center" },
          { type: "button", label: playBtn, action: "music_toggle",
            backgroundColor: "#e94560", color: "#ffffff", fontSize: 14, textAlignment: "center" },
          { type: "button", label: "\u23E9", action: "music_next",
            backgroundColor: { light: "#E0E7FF", dark: "#1a2744" }, color: { light: "#3730a3", dark: "#e2e8f0" }, fontSize: 14, textAlignment: "center" },
        ] },
      ] },
    ],
  };

  const large: WidgetElement = {
    type: "vstack", padding: 16, spacing: 10, cornerRadius: 16,
    background: { light: "#EEF2FF", dark: "#16213e" },
    children: [
      { type: "hstack", spacing: 12, children: [
        { type: "image", systemName: "music.note.list", color: "#ffffff", size: 32,
          clipShape: "circle",
          background: { gradientType: "linear", colors: ["#e94560", "#c62e46"], direction: "topToBottom" },
          frame: { width: 64, height: 64 } },
        { type: "vstack", spacing: 4, alignment: "leading", flex: 1, children: [
          { type: "text", content: track.title, textStyle: "title2", fontWeight: "bold", color: "label" },
          { type: "text", content: track.artist, textStyle: "subheadline", color: "secondaryLabel" },
          { type: "text", content: track.album, textStyle: "caption", color: "secondaryLabel" },
        ] },
      ] },
      { type: "progress", value: progress, tint: "#e94560", label: `${posStr} / ${track.durationStr}`, color: "secondaryLabel" },
      { type: "hstack", spacing: 16, children: [
        { type: "button", label: "\u23EA Prev", action: "music_prev",
          backgroundColor: { light: "#E0E7FF", dark: "#1a2744" }, color: { light: "#3730a3", dark: "#e2e8f0" }, fontSize: 13, cornerRadius: 8, textAlignment: "center", flex: 1 },
        { type: "button", label: playBtnLabel, action: "music_toggle",
          backgroundColor: "#e94560", color: "#ffffff", fontSize: 13, cornerRadius: 8, textAlignment: "center", flex: 1 },
        { type: "button", label: "Next \u23E9", action: "music_next",
          backgroundColor: { light: "#E0E7FF", dark: "#1a2744" }, color: { light: "#3730a3", dark: "#e2e8f0" }, fontSize: 13, cornerRadius: 8, textAlignment: "center", flex: 1 },
      ] },
      { type: "divider", color: "separator" },
      { type: "text", content: "Up Next", textStyle: "footnote", fontWeight: "semibold", color: "secondaryLabel" },
      ...upcoming.map((t): WidgetElement => ({
        type: "hstack", spacing: 8, children: [
          { type: "image", systemName: "music.note", color: "#4fc3f7", size: 16 },
          { type: "text", content: `${t.title} \u2022 ${t.artist}`, textStyle: "footnote", color: "label", lineLimit: 1, alignment: "leading", flex: 1 },
        ],
      })),
    ],
  };

  return { small, medium, large };
}

export const music: PresetDef = {
  icon: "\u{1F3B5}",
  name: "Music",
  config: buildMusicConfig(),
  builder: buildMusicConfig,
  intervalMs: 1000,
  onAction(action, _payload, log) {
    switch (action) {
      case "music_toggle":
        playing = !playing;
        log(playing ? "\u25B6 Playing" : "\u23F8 Paused");
        break;
      case "music_prev":
        trackIdx = (trackIdx - 1 + TRACKS.length) % TRACKS.length;
        positionSec = 0;
        log(`\u23EA ${TRACKS[trackIdx].title}`);
        break;
      case "music_next":
        trackIdx = (trackIdx + 1) % TRACKS.length;
        positionSec = 0;
        log(`\u23E9 ${TRACKS[trackIdx].title}`);
        break;
    }
  },
};
