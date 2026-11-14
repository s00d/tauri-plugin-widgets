import { useEffect, useState, useCallback, useRef } from "react";
import {
  setRegisterWidget,
  reloadAllTimelines,
  createWidgetWindow,
  closeWidgetWindow,
  startWidgetUpdater,
  setWidgetConfig,
  setWidgetState,
  onWidgetAction,
  getWidgetTrace,
  startActivity,
  updateActivity,
  endActivity,
  type WidgetConfig,
  type WidgetTrace,
} from "tauri-plugin-widgets-api";
import { PRESETS } from "./presets";
import { ISLAND_DEMOS, LAUNCHER_ID, LAUNCHER_LABEL, activityWidgetId, launcherConfig } from "./islandDemos";
import "./App.css";

const WIDGET_KIND = "ExampleWidget";
// The App Group comes from `plugins.widgets.appGroup` of the platform's config: iOS
// `group.…` (tauri.conf.json), macOS `TEAMID.…` (tauri.macos.conf.json: a Team-ID group is
// the app's own on macOS, so no "access data from other apps" checks).
const APP_GROUP: string | undefined = undefined;
const WIDGET_ID = "example";
const WIDGET_LABEL = "desktop-widget";
type WidgetSize = "small" | "medium" | "large";

const SIZE_DIMS: Record<WidgetSize, { width: number; height: number }> = {
  small: { width: 170, height: 170 },
  medium: { width: 360, height: 170 },
  large: { width: 360, height: 380 },
};

function App() {
  const [logs, setLogs] = useState<string[]>([]);
  const [registered, setRegistered] = useState(false);
  const [widgetOpen, setWidgetOpen] = useState(false);
  const [activePreset, setActivePreset] = useState<string | null>(null);
  const [tab, setTab] = useState<"presets" | "editor" | "controls">("presets");
  const [jsonText, setJsonText] = useState("");
  const [jsonError, setJsonError] = useState<string | null>(null);
  const [widgetSize, setWidgetSize] = useState<WidgetSize>("small");
  const [liveIntervalMs, setLiveIntervalMs] = useState(0);
  const [trace, setTrace] = useState<WidgetTrace | null>(null);
  const activePresetRef = useRef<string | null>(null);
  const updaterBuilderRef = useRef<(() => WidgetConfig | Promise<WidgetConfig>) | null>(null);
  const updaterStopRef = useRef<(() => void) | null>(null);
  const applyGenRef = useRef(0);
  const [runningDemos, setRunningDemos] = useState<string[]>([]);
  /** Off: picking a demo replaces the running one. On: demos stack (older ones become minimal). */
  const [stackDemos, setStackDemos] = useState(false);
  /** Per demo: stop function of its running ticker. */
  const demoStopRef = useRef(new Map<string, () => void>());

  const addLog = useCallback((message: string, isError = false) => {
    const prefix = isError ? "[ERR]" : "[OK]";
    const time = new Date().toLocaleTimeString();
    setLogs((prev) => [`${time} ${prefix} ${message}`, ...prev].slice(0, 50));
  }, []);

  useEffect(() => {
    activePresetRef.current = activePreset;
    // Remembered across restarts: the widget keeps showing the preset, and its taps need
    // the preset's handler (calculator "=") even after the app was relaunched.
    try {
      if (activePreset) localStorage.setItem("activePreset", activePreset);
    } catch {
      /* storage unavailable: start without a preset */
    }
  }, [activePreset]);

  useEffect(() => {
    (async () => {
      try {
        await setRegisterWidget([WIDGET_KIND]);
        setRegistered(true);
        addLog("Widget registered");
        let saved: string | null = null;
        try {
          saved = localStorage.getItem("activePreset");
        } catch {
          /* storage unavailable */
        }
        if (saved && PRESETS[saved]) await handleApplyPreset(saved);
      } catch (e) {
        addLog("Register failed: " + String(e), true);
      }
    })();
  }, [addLog]);

  // Single action path: live events + slow poll fallback (no dedup hack).
  useEffect(() => {
    let stopped = false;
    let unlisten: (() => void) | null = null;

    async function handleAction(action: string, payload?: string) {
      const name = activePresetRef.current;
      const preset = name ? PRESETS[name] : null;
      addLog(`Action: "${action}"${payload ? ` payload=${payload}` : ""}`);
      // Await: a preset may write widget state (`setWidgetState`) first. That write reloads the
      // widget itself: when only state changes (calculator "="), the rebuilt config below is
      // identical, so `setWidgetConfig` skips its write and reload and nothing else would.
      await preset?.onAction?.(action, payload, addLog, (patch) =>
        setWidgetState(WIDGET_ID, patch, { group: APP_GROUP }),
      );
      const builder = updaterBuilderRef.current ?? (preset ? (preset.builder ?? (() => preset.config)) : null);
      if (!builder) return;
      const next = await builder();
      await setWidgetConfig(next, APP_GROUP, WIDGET_ID);
      setJsonText(JSON.stringify(next, null, 2));
    }

    void (async () => {
      unlisten = await onWidgetAction((data) => {
        if (stopped) return;
        if (data.widgetId && data.widgetId !== WIDGET_ID) return;
        if (APP_GROUP && data.group && data.group !== APP_GROUP) return;
        void handleAction(data.action, data.payload ?? undefined);
      });
    })();

    return () => {
      stopped = true;
      unlisten?.();
    };
  }, [addLog]);

  function stopUpdater() {
    updaterStopRef.current?.();
    updaterStopRef.current = null;
    updaterBuilderRef.current = null;
    setLiveIntervalMs(0);
  }

  async function handleApplyPreset(name: string) {
    const gen = ++applyGenRef.current;
    stopUpdater();
    const preset = PRESETS[name];
    if (!preset) return;
    try {
      const builder = preset.builder ?? (() => preset.config);
      updaterBuilderRef.current = builder;
      // Live tick only when intervalMs is explicitly > 0.
      // A bare `builder` is for action rebuilds (tasks/payments), not auto-poll.
      const intervalMs =
        typeof preset.intervalMs === "number" && preset.intervalMs > 0
          ? preset.intervalMs
          : 0;

      const stop = await startWidgetUpdater(builder, APP_GROUP, WIDGET_ID, {
        intervalMs,
        immediate: true,
        // One-shot apply: reload WidgetKit once. Live ticks must not burn the budget.
        reload: intervalMs === 0,
      });

      // A newer click won the race — drop this updater.
      if (gen !== applyGenRef.current) {
        stop();
        return;
      }

      updaterStopRef.current = stop;
      // Live ticks skip reloads (WidgetKit allows few); one reload shows the new preset,
      // then device-driven parts (`startAt`/`endAt`, timers) run on their own.
      if (intervalMs > 0) await reloadAllTimelines().catch(() => {});
      setLiveIntervalMs(intervalMs);
      setActivePreset(name);
      const snap = await builder();
      if (gen !== applyGenRef.current) return;
      if (intervalMs > 0) {
        setJsonText(
          `// Auto-updating every ${intervalMs / 1000}s via startWidgetUpdater\n` +
            JSON.stringify(snap, null, 2),
        );
      } else {
        setJsonText(JSON.stringify(snap, null, 2));
      }
      setJsonError(null);
      addLog(`Applied "${preset.name}"${intervalMs > 0 ? ` (live ${intervalMs / 1000}s)` : ""}`);
    } catch (e) {
      if (gen !== applyGenRef.current) return;
      addLog(`Failed: ${String(e)}`, true);
    }
  }

  async function handleApplyJson() {
    const gen = ++applyGenRef.current;
    stopUpdater();
    try {
      // Strip // comment lines so live-preset snapshots still parse.
      const cleaned = jsonText
        .split("\n")
        .filter((line) => !/^\s*\/\//.test(line))
        .join("\n");
      const config: WidgetConfig = JSON.parse(cleaned);
      updaterBuilderRef.current = () => config;
      const stop = await startWidgetUpdater(() => config, APP_GROUP, WIDGET_ID, {
        intervalMs: 0,
        immediate: true,
        reload: true,
      });
      if (gen !== applyGenRef.current) {
        stop();
        return;
      }
      updaterStopRef.current = stop;
      setLiveIntervalMs(0);
      setActivePreset(null);
      setJsonError(null);
      addLog("Applied custom config");
    } catch (e) {
      if (gen !== applyGenRef.current) return;
      setJsonError(String(e));
      addLog(`JSON error: ${String(e)}`, true);
    }
  }

  function handleFormat() {
    try {
      const cleaned = jsonText
        .split("\n")
        .filter((line) => !/^\s*\/\//.test(line))
        .join("\n");
      const parsed = JSON.parse(cleaned);
      setJsonText(JSON.stringify(parsed, null, 2));
      setJsonError(null);
    } catch (e) {
      setJsonError(String(e));
    }
  }

  useEffect(() => {
    return () => {
      updaterStopRef.current?.();
      updaterStopRef.current = null;
    };
  }, []);

  function markDemo(id: string, running: boolean) {
    setRunningDemos((prev) => (running ? [...prev.filter((x) => x !== id), id] : prev.filter((x) => x !== id)));
  }

  /** Stop a demo's ticker and end its activity. */
  async function stopDemo(id: string) {
    demoStopRef.current.get(id)?.();
    demoStopRef.current.delete(id);
    markDemo(id, false);
    await endActivity(id, 0, { dismissalPolicy: "immediate" }).catch(() => {});
  }

  /**
   * Start one island demo: push frame(0), then a frame every `tickMs` (a tick is skipped
   * while the previous update is still in flight, like a real app would throttle).
   */
  async function startDemo(id: string) {
    const demo = ISLAND_DEMOS.find((d) => d.id === id);
    if (!demo) return;
    demoStopRef.current.get(id)?.();
    if (!stackDemos) {
      for (const other of Array.from(demoStopRef.current.keys())) if (other !== id) await stopDemo(other);
    }
    const t0 = performance.now();
    try {
      const handle = await startActivity(demo.frame(0));
      addLog(`${demo.name}: started (${handle.surface}); hover the island to expand it`);
    } catch (e) {
      addLog(`${demo.name}: start failed ${String(e)}`, true);
      return;
    }
    markDemo(id, true);
    if (!demo.tickMs) {
      demoStopRef.current.set(id, () => {});
      return;
    }
    let busy = false;
    let lateMs = 0;
    const timer = setInterval(() => {
      const t = performance.now() - t0;
      if (demo.durationMs && t >= demo.durationMs) {
        clearInterval(timer);
        demoStopRef.current.delete(id);
        markDemo(id, false);
        void endActivity(id, 2000, { content: demo.frame(demo.durationMs) }).then(() => addLog(`${demo.name}: ended`));
        return;
      }
      if (busy) {
        lateMs += demo.tickMs;
        return;
      }
      busy = true;
      const sent = performance.now();
      updateActivity(demo.frame(t))
        .then(() => {
          const rtt = performance.now() - sent;
          if (rtt > demo.tickMs * 2) addLog(`${demo.name}: slow update ${Math.round(rtt)} ms`, true);
        })
        .catch((e) => addLog(`${demo.name}: update failed ${String(e)}`, true))
        .finally(() => {
          busy = false;
        });
    }, demo.tickMs);
    demoStopRef.current.set(id, () => {
      clearInterval(timer);
      if (lateMs) addLog(`${demo.name}: ${lateMs} ms of ticks skipped (update slower than tick)`);
    });
  }

  /** Taps inside the island and in the launcher window. */
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    let stopped = false;
    void onWidgetAction(async (data) => {
      if (stopped) return;
      const from = data.widgetId ?? "";
      if (from === LAUNCHER_ID && data.action.startsWith("launch:")) {
        addLog(`Launcher: open ${data.action.slice(7)}`);
        await closeWidgetWindow(LAUNCHER_LABEL).catch(() => {});
        return;
      }
      const demo = ISLAND_DEMOS.find((d) => activityWidgetId(d.id) === from);
      if (!demo) return;
      addLog(`${demo.name}: tap "${data.action}"`);
      if (data.action === "cancel") void stopDemo(demo.id);
      if (data.action === "open-launcher") {
        await setWidgetConfig(launcherConfig(), APP_GROUP, LAUNCHER_ID);
        const w = 300;
        await createWidgetWindow({
          label: LAUNCHER_LABEL,
          width: w,
          height: 170,
          x: Math.round((window.screen.width - w) / 2),
          y: 70,
          skipTaskbar: true,
          group: APP_GROUP,
          widgetId: LAUNCHER_ID,
          size: "medium",
        }).catch((e) => addLog("Launcher window failed: " + String(e), true));
      }
    }).then((u) => {
      if (stopped) u();
      else unlisten = u;
    });
    return () => {
      stopped = true;
      unlisten?.();
      for (const stop of demoStopRef.current.values()) stop();
    };
  }, [addLog]);

  async function handleRefreshTrace() {
    try {
      const t = await getWidgetTrace(APP_GROUP, { since: Date.now() - 3_600_000 });
      setTrace(t);
      addLog(
        t.enabled
          ? `Trace: ${t.events.length} events, ${t.receipts.length} receipts`
          : "Trace disabled (set WIDGET_DEBUG=1 in release)",
      );
    } catch (e) {
      addLog("Trace failed: " + String(e), true);
    }
  }

  async function handleOpenWidget() {
    try {
      if (widgetOpen) {
        await closeWidgetWindow(WIDGET_LABEL).catch(() => {});
      }
      const dims = SIZE_DIMS[widgetSize];
      await createWidgetWindow({
        label: WIDGET_LABEL,
        width: dims.width,
        height: dims.height,
        x: 80,
        y: 80,
        skipTaskbar: true,
        group: APP_GROUP,
        widgetId: WIDGET_ID,
        size: widgetSize,
      });
      setWidgetOpen(true);
      addLog(`Widget window opened (${widgetSize}: ${dims.width}\u00D7${dims.height})`);
    } catch (e) {
      addLog("Open failed: " + String(e), true);
    }
  }

  async function handleCloseWidget() {
    try {
      await closeWidgetWindow(WIDGET_LABEL);
      setWidgetOpen(false);
      addLog("Widget window closed");
    } catch (e) {
      addLog("Close failed: " + String(e), true);
    }
  }

  return (
    <main className="container">
      <div className="card">
        <h1>Widget Builder</h1>
        <div className="status-bar">
          <span className={`badge ${registered ? "badge-ok" : "badge-err"}`}>
            {registered ? "Registered" : "Not registered"}
          </span>
          {activePreset && (
            <span className="badge badge-info">
              {PRESETS[activePreset]?.icon} {PRESETS[activePreset]?.name}
            </span>
          )}
        </div>

        <div className="tabs">
          <button className={`tab ${tab === "presets" ? "active" : ""}`} onClick={() => setTab("presets")}>
            Presets
          </button>
          <button className={`tab ${tab === "editor" ? "active" : ""}`} onClick={() => setTab("editor")}>
            Editor
          </button>
          <button className={`tab ${tab === "controls" ? "active" : ""}`} onClick={() => setTab("controls")}>
            Controls
          </button>
        </div>

        {tab === "presets" && (
          <div className="preset-grid">
            {Object.entries(PRESETS).map(([key, p]) => (
              <div
                key={key}
                className={`preset-card ${activePreset === key ? "active" : ""}`}
                onClick={() => handleApplyPreset(key)}
              >
                <span className="preset-icon">{p.icon}</span>
                {p.name}
                {activePreset === key && liveIntervalMs > 0 && (
                  <span style={{ fontSize: 10, color: "#4ade80", marginLeft: 4 }}> LIVE</span>
                )}
              </div>
            ))}
          </div>
        )}

        {tab === "editor" && (
          <>
            <textarea
              className="json-editor"
              value={jsonText}
              onChange={(e) => {
                setJsonText(e.target.value);
                setJsonError(null);
              }}
              placeholder="Paste or edit widget config JSON..."
              spellCheck={false}
            />
            {jsonError && <div className="json-error">{jsonError}</div>}
            <div className="btn-row">
              <button onClick={handleApplyJson}>Apply JSON</button>
              <button className="btn-secondary" onClick={handleFormat}>Format</button>
            </div>
          </>
        )}

        {tab === "controls" && (
          <div className="controls">
            <div className="btn-row">
              <label>
                Size{" "}
                <select value={widgetSize} onChange={(e) => setWidgetSize(e.target.value as WidgetSize)}>
                  <option value="small">small</option>
                  <option value="medium">medium</option>
                  <option value="large">large</option>
                </select>
              </label>
              <button onClick={handleOpenWidget}>{widgetOpen ? "Reopen" : "Open"} desktop widget</button>
              <button className="btn-secondary" onClick={handleCloseWidget} disabled={!widgetOpen}>
                Close
              </button>
              <button className="btn-secondary" onClick={() => reloadAllTimelines().then(() => addLog("Reloaded"))}>
                Reload timelines
              </button>
              <button className="btn-secondary" onClick={handleRefreshTrace}>
                Refresh trace
              </button>
            </div>
            <div className="btn-row">
              <label>
                <input type="checkbox" checked={stackDemos} onChange={(e) => setStackDemos(e.target.checked)} /> Run
                several at once
              </label>
              {ISLAND_DEMOS.map((d) => {
                const on = runningDemos.includes(d.id);
                return (
                  <button
                    key={d.id}
                    className={on ? "" : "btn-secondary"}
                    onClick={() => (on ? stopDemo(d.id) : startDemo(d.id))}
                  >
                    {d.icon} {d.name}
                    {on ? " ■" : ""}
                  </button>
                );
              })}
            </div>
            {trace && (
              <div className="trace-panel">
                <h3>Delivery trace {trace.enabled ? "" : "(off)"}</h3>
                <pre className="trace-pre">
                  {trace.events
                    .slice(-40)
                    .map((e) => {
                      const kind = String(e.kind ?? "?");
                      const extra =
                        kind === "reload"
                          ? ` performed=${String(e.performed)} ${JSON.stringify(e.reason)}`
                          : kind === "configSet"
                            ? ` id=${e.widgetId} changed=${String(e.changed)} bytes=${e.bytes}`
                            : kind === "render"
                              ? ` ${e.trigger}@${e.source} nonce=${e.nonce} lag=${e.lagMs}ms`
                              : kind === "write"
                                ? ` ${e.transport} ok=${String(e.ok)} ${e.durationMs}ms`
                                : "";
                      return `${new Date(Number(e.ts)).toLocaleTimeString()}  ${kind}${extra}`;
                    })
                    .join("\n") || "(empty)"}
                </pre>
              </div>
            )}
          </div>
        )}

        <div className="log-panel">
          {logs.map((l, i) => (
            <div key={i} className={l.includes("[ERR]") ? "log-err" : "log-ok"}>
              {l}
            </div>
          ))}
        </div>
      </div>
    </main>
  );
}

export default App;
