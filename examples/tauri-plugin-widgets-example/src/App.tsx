import { useEffect, useState, useCallback, useRef } from "react";
import {
  setRegisterWidget,
  reloadAllTimelines,
  createWidgetWindow,
  closeWidgetWindow,
  startWidgetUpdater,
  setWidgetConfig,
  pollPendingWidgetActions,
  type WidgetConfig,
} from "tauri-plugin-widgets-api";
import { PRESETS } from "./presets";
import "./App.css";

const WIDGET_KIND = "ExampleWidget";
const APP_GROUP = "group.com.s00d.tauri-plugin-widgets-example";
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
  const [updaterStop, setUpdaterStop] = useState<(() => void) | null>(null);
  const [widgetSize, setWidgetSize] = useState<WidgetSize>("small");
  const recentActionRef = useRef<Map<string, number>>(new Map());

  const addLog = useCallback((message: string, isError = false) => {
    const prefix = isError ? "[ERR]" : "[OK]";
    const time = new Date().toLocaleTimeString();
    setLogs((prev) => [`${time} ${prefix} ${message}`, ...prev].slice(0, 50));
  }, []);

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

  function stopUpdater() {
    if (updaterStop) {
      updaterStop();
      setUpdaterStop(null);
    }
  }

  async function handleApplyPreset(name: string) {
    stopUpdater();
    const preset = PRESETS[name];
    if (!preset) return;
    try {
      const builder = preset.builder ?? (() => preset.config);
      const intervalMs = preset.builder ? (preset.intervalMs ?? 1000) : 0;

      const stop = await startWidgetUpdater(builder, APP_GROUP, {
        intervalMs,
        immediate: true,
        onAction: (action, payload) => {
          const actionKey = `${action}::${payload ?? ""}`;
          recentActionRef.current.set(actionKey, Date.now());
          addLog(`Action: "${action}"${payload ? ` payload=${payload}` : ""}`);
          preset.onAction?.(action, payload, addLog);
          void (async () => {
            const next = await builder();
            await setWidgetConfig(next, APP_GROUP);
            if (intervalMs === 0) {
              setJsonText(JSON.stringify(next, null, 2));
            }
          })();
        },
      });

      setUpdaterStop(() => stop);
      setActivePreset(name);
      const snap = await builder();
      if (intervalMs > 0) {
        setJsonText(`// Auto-updating every ${intervalMs / 1000}s via startWidgetUpdater\n` + JSON.stringify(snap, null, 2));
      } else {
        setJsonText(JSON.stringify(snap, null, 2));
      }
      setJsonError(null);
      addLog(`Applied "${preset.name}"${intervalMs > 0 ? ` (live ${intervalMs / 1000}s)` : ""}`);
    } catch (e) {
      addLog(`Failed: ${String(e)}`, true);
    }
  }

  useEffect(() => {
    if (!activePreset) return;
    const timer = setInterval(() => {
      void (async () => {
        try {
          const actions = await pollPendingWidgetActions(APP_GROUP);
          if (!actions.length) return;
          const preset = PRESETS[activePreset];
          if (!preset) return;
          const builder = preset.builder ?? (() => preset.config);
          for (const item of actions) {
            const actionKey = `${item.action}::${item.payload ?? ""}`;
            const seenAt = recentActionRef.current.get(actionKey) ?? 0;
            if (Date.now() - seenAt < 1200) continue;
            recentActionRef.current.set(actionKey, Date.now());
            addLog(`Action(poll): "${item.action}"${item.payload ? ` payload=${item.payload}` : ""}`);
            preset.onAction?.(item.action, item.payload, addLog);
            const next = await builder();
            await setWidgetConfig(next, APP_GROUP);
          }
        } catch (e) {
          console.debug("[example-app] pollPendingWidgetActions skipped/failed", e);
        }
      })();
    }, 500);
    return () => clearInterval(timer);
  }, [activePreset, addLog]);

  async function handleApplyJson() {
    stopUpdater();
    try {
      const config: WidgetConfig = JSON.parse(jsonText);
      const stop = await startWidgetUpdater(() => config, APP_GROUP, {
        intervalMs: 0,
        immediate: true,
      });
      setUpdaterStop(() => stop);
      setActivePreset(null);
      setJsonError(null);
      addLog("Applied custom config");
    } catch (e) {
      setJsonError(String(e));
      addLog(`JSON error: ${String(e)}`, true);
    }
  }

  function handleFormat() {
    try {
      const parsed = JSON.parse(jsonText);
      setJsonText(JSON.stringify(parsed, null, 2));
      setJsonError(null);
    } catch (e) {
      setJsonError(String(e));
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
                {activePreset === key && updaterStop && (
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
            {jsonError && <div style={{ color: "#dc2626", fontSize: 11, marginBottom: 6 }}>{jsonError}</div>}
            <div className="button-group">
              <button className="btn btn-primary" onClick={handleApplyJson}>Apply Config</button>
              <button className="btn" onClick={handleFormat}>Format</button>
              <button className="btn" onClick={() => { navigator.clipboard.writeText(jsonText); addLog("Copied to clipboard"); }}>
                Copy
              </button>
            </div>
          </>
        )}

        {tab === "controls" && (
          <>
            <h3>Reload</h3>
            <div className="button-group">
              <button
                className="btn"
                onClick={() => reloadAllTimelines().then(() => addLog("Reloaded all")).catch((e) => addLog(String(e), true))}
              >
                Reload All
              </button>
            </div>
            <h3>Desktop Widget Window</h3>
            <div style={{ marginBottom: 8 }}>
              <label style={{ fontSize: 13, marginRight: 8 }}>Size:</label>
              {(["small", "medium", "large"] as WidgetSize[]).map((s) => (
                <button
                  key={s}
                  className={`btn btn-small ${widgetSize === s ? "active" : ""}`}
                  style={{
                    marginRight: 4,
                    background: widgetSize === s ? "#6366f1" : undefined,
                    color: widgetSize === s ? "#fff" : undefined,
                  }}
                  onClick={() => setWidgetSize(s)}
                >
                  {s} ({SIZE_DIMS[s].width}\u00D7{SIZE_DIMS[s].height})
                </button>
              ))}
            </div>
            <div className="button-group">
              {!widgetOpen ? (
                <button className="btn btn-primary" onClick={handleOpenWidget}>Open Widget Window</button>
              ) : (
                <>
                  <button className="btn btn-primary" onClick={handleOpenWidget}>Reopen ({widgetSize})</button>
                  <button className="btn btn-danger" onClick={handleCloseWidget}>Close</button>
                </>
              )}
            </div>
          </>
        )}
      </div>

      <div className="card log-card">
        <div className="log-header">
          <h3 style={{ margin: 0 }}>Log</h3>
          <button className="btn btn-small" onClick={() => setLogs([])}>Clear</button>
        </div>
        <div className="log-list">
          {logs.length === 0 && <p className="log-empty">No logs yet</p>}
          {logs.map((log, i) => (
            <div key={i} className={`log-entry ${log.includes("[ERR]") ? "log-err" : "log-ok"}`}>
              <button
                type="button"
                style={{
                  all: "unset",
                  display: "block",
                  width: "100%",
                  cursor: "pointer",
                }}
                title="Click to copy"
                onClick={() => {
                  void navigator.clipboard.writeText(log);
                  addLog("Log entry copied");
                }}
              >
                {log}
              </button>
            </div>
          ))}
        </div>
      </div>
    </main>
  );
}

export default App;
