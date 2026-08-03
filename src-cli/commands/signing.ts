import { defineCommand } from "citty";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import {
  detectTauriIdentifier,
  ensurePluginsWidgets,
  readTauriConf,
} from "../lib/tauri-conf.js";
import {
  codesignIdentities,
  identitiesHaveTeam,
  pickPreferredIdentity,
  recommendTransport,
  windowsDevCertScript,
} from "../lib/signing.js";
import { detectWindowsPublisher } from "../lib/windows.js";
import { syncMacosEntitlementsFromConf } from "../lib/entitlements.js";

export const signing = defineCommand({
  meta: {
    name: "signing",
    description:
      "Discover codesign identities, recommend transport, write plugins.widgets / entitlements / Windows DevCert.ps1",
  },
  args: {
    cwd: {
      type: "positional",
      description: "App project root (default: cwd)",
      required: false,
    },
    apply: {
      type: "boolean",
      description: "Write plugins.widgets + regenerate macOS entitlements from identities",
      default: false,
    },
    "write-entitlements": {
      type: "boolean",
      description: "Regenerate src-tauri/macos-widget/*.entitlements from plugins.widgets.appGroup",
      default: false,
    },
    "dev-cert": {
      type: "boolean",
      description: "Print + write Windows DevCert.ps1 (Subject = Publisher)",
      default: false,
    },
  },
  run({ args }) {
    const cwd = resolve(args.cwd || process.cwd());

    if (args["dev-cert"]) {
      const { publisher, path: pubPath } = detectWindowsPublisher(cwd);
      const script = windowsDevCertScript(publisher);
      process.stdout.write(script);
      const destCandidates = [
        join(cwd, "src-tauri", "windows-widget", "DevCert.ps1"),
        join(cwd, "windows-widget", "DevCert.ps1"),
      ];
      let wrote: string | null = null;
      for (const dest of destCandidates) {
        const dir = dirname(dest);
        if (!existsSync(dir)) continue;
        writeFileSync(dest, script, "utf-8");
        wrote = dest;
        break;
      }
      if (!wrote) {
        // Scaffold folder if missing so the file is not only printed
        const dest = destCandidates[0];
        mkdirSync(dirname(dest), { recursive: true });
        writeFileSync(dest, script, "utf-8");
        wrote = dest;
      }
      console.error(
        `\n# Wrote ${wrote.replace(cwd + "/", "")}` +
          (pubPath
            ? `  (Publisher from ${pubPath.replace(cwd + "/", "")})`
            : "  (default Publisher CN=TauriWidgetsDev — set Identity Publisher in your manifest)"),
      );
      console.error(`# Run elevated: powershell -ExecutionPolicy Bypass -File ${wrote.replace(cwd + "/", "")}`);
      return;
    }

    const conf = readTauriConf(cwd);
    const widgets = conf?.data?.plugins?.widgets || {};
    const appGroup =
      widgets.appGroup || (detectTauriIdentifier(conf) ? `group.${detectTauriIdentifier(conf)}` : null);
    const identities = codesignIdentities();
    const hasTeam = identitiesHaveTeam(identities);
    const transport = recommendTransport(identities);
    const preferred = pickPreferredIdentity(identities);

    console.log("identities");
    if (process.platform !== "darwin") {
      console.log("  – (codesign discovery is macOS-only; use --dev-cert on Windows)");
    } else if (!identities.length) {
      console.log("  – none found (ad-hoc only)");
    } else {
      for (const i of identities) {
        const mark = i.team ? "✓" : "–";
        console.log(`  ${mark} ${i.name}${i.team ? `   team ${i.team}` : ""}`);
      }
    }
    console.log("  – ad-hoc (-)                                          no team");

    console.log("\napp group");
    if (widgets.appGroup) {
      console.log(`  ✓ ${widgets.appGroup}  (from plugins.widgets.appGroup)`);
    } else if (appGroup) {
      console.log(`  – ${appGroup}  (suggested from identifier — not in conf yet)`);
    } else {
      console.log("  – missing (need plugins.widgets.appGroup or identifier)");
    }

    console.log("\nverdict");
    if (hasTeam) {
      console.log(`  transport = appGroup      ← Team ID present; shared container can work`);
    } else {
      console.log(`  transport = widgetContainer  ← no Team ID certificate`);
      console.log(`  App Groups / shared container will not work without a team — expected for local ad-hoc.`);
    }
    if (widgets.transport && widgets.transport !== transport) {
      console.log(`  ⚠ conf has transport=${widgets.transport} (recommend: ${transport})`);
    }

    console.log("\ncommands");
    if (preferred) {
      console.log(`  export WIDGET_SIGN_IDENTITY="${preferred.name}"`);
    } else {
      console.log(`  # WIDGET_SIGN_IDENTITY=-  (ad-hoc)`);
    }
    console.log(`  pnpm tauri build`);
    console.log(`  npx tauri-widgets signing --apply`);
    console.log(`  npx tauri-widgets signing --write-entitlements`);
    console.log(`  npx tauri-widgets signing --dev-cert`);

    if (args.apply) {
      if (!conf) {
        console.error("\nERROR: tauri.conf.json not found — cannot --apply");
        process.exit(1);
      }
      if (!appGroup) {
        console.error("\nERROR: no appGroup / identifier — pass plugins.widgets.appGroup first");
        process.exit(1);
      }
      const extensionBundleId =
        widgets.extensionBundleId ||
        (detectTauriIdentifier(conf) ? `${detectTauriIdentifier(conf)}.widgetkit` : undefined);
      let applyTransport = transport;
      if (widgets.transport === "appGroup" && transport === "widgetContainer") {
        applyTransport = "appGroup";
      }
      const result = ensurePluginsWidgets(conf, {
        appGroup,
        transport: applyTransport,
        extensionBundleId,
        force: false,
      });
      console.log(
        result.wrote
          ? `\nWrote plugins.widgets (transport=${applyTransport}, appGroup=${appGroup})`
          : `\nplugins.widgets already matched (transport=${applyTransport})`,
      );
      const ents = syncMacosEntitlementsFromConf(cwd, readTauriConf(cwd));
      if (ents.wrote) {
        console.log(`Regenerated macos-widget entitlements for ${ents.appGroup}`);
      }
    }

    if (args["write-entitlements"]) {
      const conf2 = readTauriConf(cwd);
      if (!conf2?.data?.plugins?.widgets?.appGroup) {
        console.error("\nERROR: plugins.widgets.appGroup required for --write-entitlements");
        process.exit(1);
      }
      const ents = syncMacosEntitlementsFromConf(cwd, conf2);
      if (ents.wrote) {
        console.log(`\nRegenerated ${ents.dir!.replace(cwd + "/", "")}/*.entitlements (${ents.appGroup})`);
      } else {
        console.error(`\nERROR: could not write entitlements (${ents.reason})`);
        process.exit(1);
      }
    }
  },
});
