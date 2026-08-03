import { defineCommand } from "citty";
import { existsSync, readdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { pluginRoot } from "../lib/paths.js";
import { detectTauriIdentifier, readTauriConf } from "../lib/tauri-conf.js";
import {
  codesignIdentities,
  codesignIsAdHoc,
  identitiesHaveTeam,
  pickPreferredIdentity,
  recommendTransport,
} from "../lib/signing.js";
import {
  extractAppGroups,
  findAppGroupMentions,
  findEntitlementsFiles,
} from "../lib/entitlements.js";
import { findBuiltApps } from "../lib/fsx.js";
import { loadTraceFile } from "../lib/trace.js";
import { checkReceiptSchemaSkew } from "../lib/receipt.js";
import { validatePluginWidgets } from "../lib/validate.js";
import { createReport, type Report } from "../lib/report.js";

export async function runDoctor(cwd: string): Promise<Report> {
  const report = createReport();
  const root = pluginRoot();

  report.section("transport");
  const conf = readTauriConf(cwd);
  if (!conf) {
    report.bad("tauri.conf.json not found under src-tauri/ or cwd");
  } else {
    const widgets = conf.data?.plugins?.widgets || {};
    const transport = widgets.transport || "auto";
    const appGroup = widgets.appGroup || "";
    report.ok(`${conf.path}`);
    report.ok(`plugins.widgets.transport = ${transport}`);
    if (!appGroup) report.bad("plugins.widgets.appGroup is empty");
    else report.ok(`appGroup = ${appGroup}`);

    if (transport === "widgetContainer") {
      report.warn("widgetContainer is macOS ad-hoc friendly; invalid on iOS (use appGroup)");
    }
    if (transport === "appGroup" && !appGroup) {
      report.bad("transport=appGroup requires plugins.widgets.appGroup");
    }
    if (transport === "auto") {
      report.warn("transport=auto is for local dev — pin appGroup or widgetContainer before release");
    }
  }

  report.section("entitlements"); // no leading blank line — matches original doctor output
  const appGroupConf = conf?.data?.plugins?.widgets?.appGroup || "";
  const transportKind = conf?.data?.plugins?.widgets?.transport || "auto";
  const identifier = detectTauriIdentifier(conf) || "";
  const ents = findEntitlementsFiles(cwd);
  if (!ents.length) report.warn("no .entitlements files found under src-tauri");
  const allGroups = new Set<string>();
  for (const f of ents) {
    const gs = extractAppGroups(f);
    gs.forEach((g) => allGroups.add(g));
    const rel = f.startsWith(cwd) ? f.slice(cwd.length + 1) : f;
    if (appGroupConf && gs.length && !gs.includes(appGroupConf)) {
      report.bad(`${rel}: App Groups ${gs.join(", ")} ≠ conf ${appGroupConf}`);
    } else if (appGroupConf && gs.includes(appGroupConf)) {
      report.ok(`${rel} contains ${appGroupConf}`);
    } else if (gs.length) {
      report.ok(`${rel}: ${gs.join(", ")}`);
    }
  }
  if (appGroupConf && allGroups.size && !allGroups.has(appGroupConf)) {
    report.bad(`no entitlements file lists ${appGroupConf}`);
  }

  report.section("\napp group (3 places)");
  if (!appGroupConf) {
    report.warn("skip — plugins.widgets.appGroup empty");
  } else {
    const places = {
      conf: Boolean(conf?.data?.plugins?.widgets?.appGroup),
      entitlements: allGroups.has(appGroupConf),
      project: false,
    };
    const { hits } = findAppGroupMentions(cwd, appGroupConf);
    const projectHits = hits.filter(
      (h) =>
        h.endsWith(".plist") ||
        h.endsWith(".pbxproj") ||
        h.includes("gen/apple") ||
        h.includes("macos-widget") ||
        h.includes("ios-widget"),
    );
    places.project = projectHits.length > 0;
    if (places.conf) report.ok(`tauri.conf plugins.widgets.appGroup = ${appGroupConf}`);
    else report.bad("tauri.conf missing appGroup");
    if (places.entitlements) report.ok("entitlements list the same group");
    else report.warn("entitlements do not list the conf appGroup yet");
    if (places.project) report.ok(`project files mention group (${projectHits.slice(0, 3).join(", ")})`);
    else report.warn("no Info.plist / pbxproj / widget target mention of appGroup (ok before init-macos/ios)");
  }

  report.section("\nsigning");
  if (process.platform === "darwin") {
    const identities = codesignIdentities();
    const hasTeam = identitiesHaveTeam(identities);
    const recommended = recommendTransport(identities);
    if (!identities.length) {
      report.warn("no codesign identities — only ad-hoc (-) available");
    } else {
      for (const i of identities.slice(0, 5)) {
        report.ok(`${i.kind}: ${i.name}${i.team ? ` (team ${i.team})` : ""}`);
      }
      if (identities.length > 5) report.ok(`… +${identities.length - 5} more (npx tauri-widgets signing)`);
    }
    if (transportKind === "appGroup" || transportKind === "userDefaults") {
      if (!hasTeam) {
        report.bad(
          `transport=${transportKind} but no Team ID certificate — use widgetContainer or install a Development/Developer ID identity`,
        );
      } else {
        report.ok(`Team ID available → ${transportKind} is plausible`);
      }
    } else if (transportKind === "widgetContainer") {
      if (hasTeam) {
        report.warn(`transport=widgetContainer while Team ID exists — switch to appGroup for release / MAS`);
      } else {
        report.ok(`widgetContainer matches ad-hoc / no-team setup`);
      }
    } else if (transportKind === "auto") {
      report.warn(`transport=auto — pin to ${recommended} (from identities)`);
    }
    const preferred = pickPreferredIdentity(identities);
    if (preferred) {
      report.ok(`suggested WIDGET_SIGN_IDENTITY="${preferred.name}"`);
    }
  } else {
    report.warn("identity discovery is macOS-only — on Windows: npx tauri-widgets signing --dev-cert");
  }

  const apps = findBuiltApps(cwd);
  if (!apps.length) {
    report.warn("no .app under target/*/bundle/macos — skip codesign inspect (build first)");
  } else {
    for (const app of apps.slice(0, 3)) {
      const adhoc = codesignIsAdHoc(app);
      const rel = app.startsWith(cwd) ? app.slice(cwd.length + 1) : app;
      if (adhoc === null) {
        report.warn(`codesign could not inspect ${rel}`);
        continue;
      }
      if (adhoc) {
        report.ok(`${rel}: ad-hoc signature`);
        if (transportKind === "appGroup" || transportKind === "userDefaults") {
          report.bad(
            `transport=${transportKind} needs a Team ID App Group — use widgetContainer for ad-hoc, or sign with a team`,
          );
        }
      } else {
        report.ok(`${rel}: team / identity signature`);
      }
    }
  }

  report.section("\nbundle");
  const appexCandidates = [
    join(cwd, "src-tauri", "target", "release", "bundle", "macos"),
    join(cwd, "artifacts", "macos"),
    join(root, "artifacts", "macos"),
  ];
  let foundAppex = false;
  for (const candidateRoot of appexCandidates) {
    if (!existsSync(candidateRoot)) continue;
    const walk = (dir: string, depth = 0) => {
      if (depth > 5 || foundAppex) return;
      let entries: import("node:fs").Dirent[] = [];
      try {
        entries = readdirSync(dir, { withFileTypes: true });
      } catch {
        return;
      }
      for (const e of entries) {
        const p = join(dir, e.name);
        if (e.isDirectory() && e.name.endsWith(".appex")) {
          foundAppex = true;
          report.ok(`.appex found: ${p}`);
          return;
        }
        if (e.isDirectory()) walk(p, depth + 1);
      }
    };
    walk(candidateRoot);
  }
  if (!foundAppex) report.warn("no .appex found in release bundle / artifacts (ok if not built yet)");

  report.section("\nruntime (trace)");
  const trace = loadTraceFile(cwd, identifier);
  if (!trace) {
    report.warn(
      "widget_trace.json not found — run `pnpm tauri dev` once (debug journals by default), or set WIDGET_DEBUG=1 for release",
    );
  } else {
    report.ok(`trace ${trace.path} (${trace.events.length} events)`);
    const dayAgo = Date.now() - 86_400_000;
    const recent = trace.events.filter((e) => Number(e.ts) >= dayAgo);
    const reloads = recent.filter((e) => e.kind === "reload");
    const throttled = reloads.filter((e) => {
      const reason = e.reason;
      return typeof reason === "object" && reason != null && (reason.outcome === "throttled" || reason.Throttled);
    });
    if (throttled.length) {
      report.bad(
        `${throttled.length}/${reloads.length} reloads throttled in last 24h — TAURI_WIDGET_MIN_RELOAD_SECS or skipReload`,
      );
    } else if (reloads.length) {
      report.ok(`${reloads.length} reload events in last 24h`);
    }
    if (reloads.length >= 60) {
      report.warn(`${reloads.length} reloads/24h — near WidgetKit daily budget`);
    }
    const renders = recent.filter((e) => e.kind === "render");
    if (renders.length) {
      const last = renders[renders.length - 1];
      const ageSec = Math.round((Date.now() - Number(last.ts)) / 1000);
      report.ok(`last render ${ageSec}s ago, nonce ${last.nonce}, trigger=${last.trigger}`);
    } else {
      report.warn("no render events in trace (native/desktop widget may not have painted)");
    }
  }

  report.section("\nschema");
  checkReceiptSchemaSkew(root, report);

  report.section("\nplugin-config");
  const widgetsBlock = conf?.data?.plugins?.widgets;
  const pluginFindings = validatePluginWidgets(widgetsBlock);
  if (!pluginFindings.length) report.ok("plugins.widgets matches plugin-config shape");
  for (const f of pluginFindings) {
    if (f.level === "error") report.bad(f.msg);
    else report.warn(f.msg);
  }

  return report;
}

export const doctor = defineCommand({
  meta: {
    name: "doctor",
    description: "Static + optional runtime checks for widget transport / signing / bundle",
  },
  args: {
    cwd: {
      type: "positional",
      description: "App project root (default: cwd)",
      required: false,
    },
  },
  async run({ args }) {
    const cwd = resolve(args.cwd || process.cwd());
    const report = await runDoctor(cwd);
    report.print();
    process.exitCode = report.failures ? 1 : 0;
  },
});
