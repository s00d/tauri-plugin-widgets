import { useEffect, useState, useCallback, useRef } from "react";
import {
  setRegisterWidget,
  reloadAllTimelines,
  createWidgetWindow,
  closeWidgetWindow,
  startWidgetUpdater,
  setWidgetConfig,
  onWidgetAction,
  getWidgetTrace,
  type WidgetConfig,
  type WidgetTrace,
} from "tauri-plugin-widgets-api";
import { PRESETS } from "./presets";
import "./App.css";

const WIDGET_KIND = "ExampleWidget";
const APP_GROUP = "group.com.s00d.tauriwidgets.example";
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

  const addLog = useCallback((message: string, isError = false) => {
    const prefix = isError ? "[ERR]" : "[OK]";
    const time = new Date().toLocaleTimeString();
    setLogs((prev) => [`${time} ${prefix} ${message}`, ...prev].slice(0, 50));
  }, []);

  useEffect(() => {
    activePresetRef.current = activePreset;
  }, [activePreset]);

  useEffect(() => {
    (async () => {
      try {
        await setRegisterWidget([WIDGET_KIND]);
        setRegistered(true);
        addLog("Widget registered");
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
      preset?.onAction?.(action, payload, addLog);
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
        if (data.group && data.group !== APP_GROUP) return;
        void handleAction(data.action, data.payload);
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
      });

      // A newer click won the race — drop this updater.
      if (gen !== applyGenRef.current) {
        stop();
        return;
      }

      updaterStopRef.current = stop;
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
