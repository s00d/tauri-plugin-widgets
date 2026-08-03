import { createServer, type Server, type ServerResponse } from "node:http";
import { existsSync, readFileSync, watch, type FSWatcher } from "node:fs";
import { join, resolve } from "node:path";
import { pluginRoot } from "./paths.js";

const WIDGET_HTML = join(pluginRoot(), "widget.html");

function loadConfig(path: string): unknown {
  const raw = readFileSync(path, "utf-8");
  const data = JSON.parse(raw) as unknown;
  // Strip $schema for runtime if present
  if (data && typeof data === "object" && "$schema" in (data as Record<string, unknown>)) {
    const { $schema: _schema, ...rest } = data as Record<string, unknown>;
    return rest;
  }
  return data;
}

function shimScript(size: string): string {
  return `<script>
(function(){
  var cfg = null;
  var listeners = {};
  function fetchConfig(){
    return fetch('/api/config').then(function(r){ if(!r.ok) throw new Error('config '+r.status); return r.json(); })
      .then(function(j){ cfg = j; return j; });
  }
  window.__TAURI_INTERNALS__ = {
    invoke: function(cmd){
      var c = String(cmd||'');
      if (c.indexOf('get_widget_config') !== -1) {
        return cfg ? Promise.resolve(cfg) : fetchConfig();
      }
      if (c.indexOf('widget_action') !== -1) {
        console.log('[preview] widget_action', arguments[1]);
        return Promise.resolve(null);
      }
      if (c.indexOf('plugin:event|listen') !== -1) return Promise.resolve();
      return Promise.resolve(null);
    },
    transformCallback: function(f){
      var id = 'cb_'+Math.random().toString(36).slice(2);
      listeners[id] = f;
      return id;
    },
    metadata: { currentWindow: { label: 'preview' } }
  };
  // Hot reload via SSE
  try {
    var es = new EventSource('/api/events');
    es.onmessage = function(){ location.reload(); };
  } catch (e) {}
  // Force size via query if missing
  if (!new URLSearchParams(location.search).get('size')) {
    var u = new URL(location.href);
    u.searchParams.set('size', ${JSON.stringify(size)});
    history.replaceState(null,'',u);
  }
})();
</script>`;
}

export interface StartPreviewServerOptions {
  configPath: string;
  size?: string;
  port?: number;
  watchFile?: boolean;
  open?: boolean;
}

export interface PreviewServerHandle {
  url: string;
  close: () => void;
}

/**
 * Serve widget.html with Tauri shim + live reload for a widget config JSON file.
 */
export function startPreviewServer({
  configPath,
  size = "medium",
  port = 4177,
  watchFile = true,
  open = false,
}: StartPreviewServerOptions): Promise<PreviewServerHandle> {
  const abs = resolve(configPath);
  if (!existsSync(abs)) {
    throw new Error(`config not found: ${abs}`);
  }
  if (!existsSync(WIDGET_HTML)) {
    throw new Error(`widget.html missing at ${WIDGET_HTML}`);
  }

  let htmlBase = readFileSync(WIDGET_HTML, "utf-8");
  const clients = new Set<ServerResponse>();

  const server: Server = createServer((req, res) => {
    const url = new URL(req.url || "/", `http://127.0.0.1`);
    if (url.pathname === "/api/config") {
      try {
        const body = JSON.stringify(loadConfig(abs));
        res.writeHead(200, { "content-type": "application/json", "cache-control": "no-store" });
        res.end(body);
      } catch (e) {
        res.writeHead(500, { "content-type": "text/plain" });
        res.end(String((e as Error)?.message || e));
      }
      return;
    }
    if (url.pathname === "/api/events") {
      res.writeHead(200, {
        "content-type": "text/event-stream",
        "cache-control": "no-cache",
        connection: "keep-alive",
      });
      res.write(":\n\n");
      clients.add(res);
      req.on("close", () => clients.delete(res));
      return;
    }
    if (url.pathname === "/" || url.pathname === "/index.html") {
      const injected = htmlBase.replace("<head>", `<head>${shimScript(size)}`);
      // Ensure size/group/widgetId query defaults in links — page uses URLSearchParams
      res.writeHead(200, { "content-type": "text/html; charset=utf-8", "cache-control": "no-store" });
      res.end(injected);
      return;
    }
    res.writeHead(404);
    res.end("not found");
  });

  const notify = () => {
    for (const res of clients) {
      try {
        res.write("data: reload\n\n");
      } catch {
        clients.delete(res);
      }
    }
  };

  let watcher: FSWatcher | null = null;
  if (watchFile) {
    try {
      watcher = watch(abs, { persistent: true }, () => {
        // Reload widget.html from disk too (in case plugin updates)
        try {
          htmlBase = readFileSync(WIDGET_HTML, "utf-8");
        } catch {
          /* keep */
        }
        notify();
      });
    } catch (e) {
      console.error("watch failed:", (e as Error).message);
    }
  }

  return new Promise((resolvePromise, reject) => {
    server.once("error", reject);
    server.listen(port, "127.0.0.1", () => {
      const url = `http://127.0.0.1:${port}/?size=${encodeURIComponent(size)}&group=preview&widgetId=preview`;
      console.log(`preview  ${url}`);
      console.log(`config   ${abs}${watchFile ? "  (watching)" : ""}`);
      if (open) {
        const opener =
          process.platform === "darwin" ? "open" : process.platform === "win32" ? "start" : "xdg-open";
        import("node:child_process").then(({ spawn }) => {
          spawn(opener, [url], { stdio: "ignore", detached: true }).unref();
        });
      }
      resolvePromise({
        url,
        close: () => {
          if (watcher) watcher.close();
          for (const res of clients) {
            try {
              res.end();
            } catch {
              /* ignore */
            }
          }
          server.close();
        },
      });
    });
  });
}

export { WIDGET_HTML, loadConfig };
