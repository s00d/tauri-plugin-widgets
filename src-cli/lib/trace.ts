import { existsSync, readFileSync, readdirSync, watch } from "node:fs";
import { join } from "node:path";

export interface TraceEvent {
  ts: number;
  kind?: string;
  type?: string;
  role?: string;
  widgetId?: string;
  nonce?: number | string;
  trigger?: string;
  source?: string;
  changed?: unknown;
  ok?: boolean;
  reason?:
    | string
    | {
        outcome?: string;
        Throttled?: boolean;
        remainingSecs?: number;
        remaining_secs?: number;
        remaining?: number;
        performed?: unknown;
      };
  [key: string]: unknown;
}

export interface LoadedTrace {
  path: string;
  events: TraceEvent[];
}

export function findFilesNamed(dir: string, name: string, depth = 0, out: string[] = []): string[] {
  if (depth > 5 || !existsSync(dir)) return out;
  let entries: import("node:fs").Dirent[] = [];
  try {
    entries = readdirSync(dir, { withFileTypes: true });
  } catch {
    return out;
  }
  for (const e of entries) {
    const p = join(dir, e.name);
    if (e.isFile() && e.name === name) out.push(p);
    else if (e.isDirectory() && !e.name.startsWith(".")) findFilesNamed(p, name, depth + 1, out);
  }
  return out;
}

/** Resolve widget_trace.json candidates (aligned with Rust app_data_dir/widgets/). */
export function resolveTracePaths(cwd: string, identifier: string | null): string[] {
  const candidates: string[] = [];
  const home = process.env.HOME || "";
  const local = process.env.LOCALAPPDATA || "";
  const xdg = process.env.XDG_DATA_HOME || (home ? join(home, ".local", "share") : "");

  if (identifier) {
    if (home) {
      candidates.push(join(home, "Library", "Application Support", identifier, "widgets", "widget_trace.json"));
    }
    if (local) {
      candidates.push(join(local, identifier, "widgets", "widget_trace.json"));
    }
    if (xdg) {
      candidates.push(join(xdg, identifier, "widgets", "widget_trace.json"));
    }
  }
  // Legacy / mistaken paths still checked by doctor historically
  candidates.push(
    join(cwd, "src-tauri", "target", "debug", "widgets", "widget_trace.json"),
    join(cwd, "src-tauri", "target", "release", "widgets", "widget_trace.json"),
  );
  if (home) {
    candidates.push(...findFilesNamed(join(home, "Library", "Application Support"), "widget_trace.json").slice(0, 8));
  }
  return candidates;
}

export function loadTraceFile(cwd: string, identifier: string | null): LoadedTrace | null {
  for (const c of resolveTracePaths(cwd, identifier)) {
    if (!c || !existsSync(c)) continue;
    try {
      const events = JSON.parse(readFileSync(c, "utf-8")) as unknown;
      if (Array.isArray(events)) return { path: c, events: events as TraceEvent[] };
    } catch {
      /* ignore */
    }
  }
  return null;
}

function fmtTs(ts: number | string): string {
  const d = new Date(Number(ts));
  if (Number.isNaN(d.getTime())) return String(ts);
  return d.toISOString().slice(11, 19);
}

export function formatTraceEvent(e: TraceEvent): string {
  const t = fmtTs(e.ts);
  const kind = e.kind || e.type || "?";
  const role = e.role || (kind === "render" ? "wgt" : "host");
  const bits: string[] = [];
  if (e.widgetId) bits.push(e.widgetId);
  if (e.nonce != null) bits.push(`nonce=${e.nonce}`);
  if (e.trigger) bits.push(`trigger=${e.trigger}`);
  if (e.source) bits.push(`source=${e.source}`);
  if (e.changed != null) bits.push(`changed=${e.changed}`);
  if (e.reason) {
    const r = e.reason;
    if (typeof r === "object") {
      if (r.outcome) bits.push(`outcome=${r.outcome}`);
      if (r.Throttled || r.outcome === "throttled") {
        const rem = r.remainingSecs ?? r.remaining_secs ?? r.remaining;
        bits.push(rem != null ? `throttled remaining=${rem}s` : "throttled");
      }
      if (r.performed != null) bits.push(`performed=${r.performed}`);
    } else bits.push(String(r));
  }
  if (e.ok === false) bits.push("✗");
  if (e.ok === true && kind === "render") bits.push("✓");
  return `${t}  ${String(role).padEnd(4)} ${String(kind).padEnd(10)} ${bits.join(" ")}`.trimEnd();
}

export interface RunTraceFollowOptions {
  cwd: string;
  identifier: string | null;
  follow?: boolean;
  lines?: number;
}

/** Print last N events; optionally follow file for new events. */
export async function runTraceFollow({
  cwd,
  identifier,
  follow = false,
  lines = 40,
}: RunTraceFollowOptions): Promise<void> {
  const loaded = loadTraceFile(cwd, identifier);
  if (!loaded) {
    console.error(
      "widget_trace.json not found — run the app once (debug) or set WIDGET_DEBUG=1.\n" +
        `Looked under Application Support / app data for identifier=${identifier || "(none)"}`,
    );
    process.exitCode = 1;
    return;
  }
  console.error(`# ${loaded.path}`);
  const slice = loaded.events.slice(-lines);
  for (const e of slice) console.log(formatTraceEvent(e));

  if (!follow) return;

  const eventKey = (e: TraceEvent) =>
    `${e.ts}:${e.kind ?? e.type ?? ""}:${JSON.stringify(e)}`;
  let lastSeenKey = slice.length ? eventKey(slice[slice.length - 1]) : "";
  let lastLen = loaded.events.length;
  let raw = readFileSync(loaded.path, "utf-8");
  const tick = () => {
    try {
      const next = readFileSync(loaded.path, "utf-8");
      if (next === raw) return;
      raw = next;
      const events = JSON.parse(next) as unknown;
      if (!Array.isArray(events)) return;
      const typed = events as TraceEvent[];

      if (typed.length < lastLen) {
        for (const e of typed.slice(-lines)) console.log(formatTraceEvent(e));
        lastSeenKey = typed.length ? eventKey(typed[typed.length - 1]) : "";
        lastLen = typed.length;
        return;
      }

      let startIdx = 0;
      if (lastSeenKey) {
        const keys = typed.map((e) => eventKey(e));
        const idx = keys.lastIndexOf(lastSeenKey);
        if (idx >= 0) {
          startIdx = idx + 1;
        } else {
          for (const e of typed.slice(-lines)) console.log(formatTraceEvent(e));
          lastSeenKey = typed.length ? eventKey(typed[typed.length - 1]) : "";
          lastLen = typed.length;
          return;
        }
      } else {
        startIdx = lastLen;
      }
      if (startIdx < typed.length) {
        for (const e of typed.slice(startIdx)) console.log(formatTraceEvent(e));
        lastSeenKey = eventKey(typed[typed.length - 1]);
      }
      lastLen = typed.length;
    } catch {
      /* ignore partial writes */
    }
  };
  watch(loaded.path, { persistent: true }, tick);
  // Also poll — some FS don't emit reliably for atomic replaces
  setInterval(tick, 1000);
  console.error("# following (Ctrl+C to stop)");
  await new Promise(() => {});
}
