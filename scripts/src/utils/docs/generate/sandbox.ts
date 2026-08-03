// @ts-nocheck
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative } from "node:path";
import {
  ALL_PLATFORMS,
  PUBLIC,
  ROOT,
  writeOrCheck,
} from "./shared.js";
export function copyShots(report, check) {
  const dest = join(PUBLIC, "shots");
  if (existsSync(dest)) rmSync(dest, { recursive: true, force: true });
  mkdirSync(dest, { recursive: true });
  for (const platform of ALL_PLATFORMS) {
    const src = join(ROOT, "tests/golden", platform);
    if (!existsSync(src)) continue;
    for (const file of readdirSync(src)) {
      if (!file.endsWith(".png")) continue;
      const to = join(dest, platform, file);
      mkdirSync(dirname(to), { recursive: true });
      copyFileSync(join(src, file), to);
      report.written.push(relative(ROOT, to));
    }
  }
  void check;
}

export function copySchemas(report, check) {
  const srcDir = join(ROOT, "schemas");
  const destDir = join(PUBLIC, "schemas");
  mkdirSync(destDir, { recursive: true });
  for (const name of ["widget-config.v1.json", "plugin-config.v1.json", "capabilities.json"]) {
    const from = join(srcDir, name);
    const to = join(destDir, name);
    if (!existsSync(from)) {
      report.stale.push(`missing schema ${name}`);
      continue;
    }
    const body = readFileSync(from, "utf8");
    if (check) {
      if (!existsSync(to) || readFileSync(to, "utf8") !== body) {
        report.stale.push(`docs/public/schemas/${name}`);
      }
    } else {
      writeFileSync(to, body, "utf8");
      report.written.push(`docs/public/schemas/${name}`);
    }
  }
}

export function copySandbox(report, check) {
  const widget = readFileSync(join(ROOT, "widget.html"), "utf8");
  const shim = `<script>
(function(){
  var pendingConfig = null;
  var callbacks = {};
  var nextId = 1;
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: 'docs-sandbox' } },
    transformCallback: function(cb) {
      var id = nextId++;
      callbacks[id] = cb;
      return id;
    },
    invoke: function(cmd, args) {
      if (cmd === 'plugin:widgets|get_widget_config') {
        return Promise.resolve(pendingConfig || { small: { type: 'text', content: 'Waiting for config…', color: '#fff' } });
      }
      if (cmd === 'plugin:widgets|widget_action') {
        console.log('[sandbox] action', args);
        return Promise.resolve();
      }
      if (cmd === 'plugin:widgets|close_widget_window') return Promise.resolve();
      if (cmd === 'plugin:event|listen') return Promise.resolve();
      if (cmd.indexOf('report_receipt') !== -1 || cmd.indexOf('report-receipt') !== -1) return Promise.resolve();
      return Promise.resolve(null);
    }
  };
  window.addEventListener('message', function(ev) {
    var data = ev.data;
    if (!data || data.type !== 'render') return;
    pendingConfig = data.config;
    if (typeof window.__WIDGET_SANDBOX_RENDER__ === 'function') {
      window.__WIDGET_SANDBOX_RENDER__(pendingConfig);
    } else {
      window.__WIDGET_SANDBOX_PENDING__ = pendingConfig;
    }
  });
})();
</script>`;

  // Match both pre-rollup (`(function(){`) and rollup IIFE (`(function () {`).
  const scriptOpen = /<script>\n\(function\s*\(\)\s*\{/;
  if (!scriptOpen.test(widget)) {
    throw new Error("widget.html: expected IIFE script open for sandbox shim");
  }
  let html = widget.replace(scriptOpen, `${shim}\n<script>\n(function () {`);

  const renderSig =
    /function render\(cfg,\s*source(?:,\s*nonceHint)?\)\s*\{/;
  if (!renderSig.test(html)) {
    throw new Error("widget.html: expected render(cfg, source[, nonceHint]) for sandbox hook");
  }
  html = html.replace(
    renderSig,
    "window.__WIDGET_SANDBOX_RENDER__=function(c){render(c,'sandbox')};\n" +
      "if(window.__WIDGET_SANDBOX_PENDING__){var __p=window.__WIDGET_SANDBOX_PENDING__;window.__WIDGET_SANDBOX_PENDING__=null;setTimeout(function(){render(__p,'sandbox')},0);}\n" +
      "function render(cfg, source, nonceHint) {",
  );

  writeOrCheck(report, join(PUBLIC, "widget-sandbox.html"), html, check);
}
