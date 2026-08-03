import { defineCommand } from "citty";
import { copyFileSync, existsSync, mkdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { pluginRoot } from "../lib/paths.js";
import { detectTauriIdentifier, readTauriConf } from "../lib/tauri-conf.js";
import { windowsDevCertScript } from "../lib/signing.js";
import { cryptoRandomGuid, detectWindowsPublisher } from "../lib/windows.js";
import { copyTemplate } from "../lib/templates.js";

export interface InitWindowsOptions {
  displayName?: string;
  packageName?: string;
  clsid?: string;
  dir: string;
  force: boolean;
}

export function initWindowsAction(opts: InitWindowsOptions): void {
  const cwd = process.cwd();
  const conf = readTauriConf(cwd);

  let packageName = opts.packageName;
  let displayName = opts.displayName;
  let clsid = opts.clsid;

  if (!packageName) {
    const identifier = conf ? detectTauriIdentifier(conf) : null;
    if (identifier) {
      const parts = identifier.split(".");
      packageName = parts[parts.length - 1] || "TauriApp";
      console.log(`  Auto-detected package-name: ${packageName}`);
    } else {
      packageName = "TauriApp";
      console.log(`  Using default package-name: ${packageName}`);
    }
  }

  if (!displayName) {
    displayName = conf?.data?.productName || packageName;
    console.log(`  Using display-name: ${displayName}`);
  }

  if (!clsid) {
    clsid = cryptoRandomGuid();
    console.log(`  Generated CLSID: ${clsid}`);
  } else {
    clsid = String(clsid).replace(/[{}]/g, "").toLowerCase();
  }

  const widgetDir = resolve(cwd, opts.dir);
  const templateDir = join(pluginRoot(), "templates", "windows-widget");

  if (existsSync(widgetDir) && !opts.force) {
    console.error(`ERROR: ${opts.dir} already exists. Use --force to overwrite.`);
    process.exit(1);
  }

  const extensionId = `${packageName}.Widgets`;
  const definitionId = `${packageName}.Widget`;

  const replacements = {
    "{{WIDGET_PROVIDER_CLSID}}": clsid,
    "{{WIDGET_DISPLAY_NAME}}": displayName,
    "{{PACKAGE_NAME}}": packageName,
    "{{WIDGET_EXTENSION_ID}}": extensionId,
    "{{WIDGET_DEFINITION_ID}}": definitionId,
  };

  console.log(`\nCreating Windows widget provider in ${opts.dir}...\n`);

  mkdirSync(join(widgetDir, "WidgetProvider"), { recursive: true });

  for (const name of ["Provider.cs", "Store.cs", "Program.cs", "WidgetProvider.csproj"]) {
    copyTemplate(join(templateDir, "WidgetProvider", name), join(widgetDir, "WidgetProvider", name), replacements);
    console.log(`  Created WidgetProvider/${name}`);
  }

  mkdirSync(join(widgetDir, "PreviewHost"), { recursive: true });
  for (const name of ["Program.cs", "PreviewHost.csproj"]) {
    copyTemplate(join(templateDir, "PreviewHost", name), join(widgetDir, "PreviewHost", name), replacements);
    console.log(`  Created PreviewHost/${name}`);
  }

  mkdirSync(join(widgetDir, "Assets"), { recursive: true });
  const logoSrc = join(templateDir, "Assets", "StoreLogo.png");
  if (existsSync(logoSrc)) {
    copyFileSync(logoSrc, join(widgetDir, "Assets", "StoreLogo.png"));
    console.log("  Created Assets/StoreLogo.png");
  }

  copyTemplate(
    join(templateDir, "Package.appxmanifest.fragment.xml"),
    join(widgetDir, "Package.appxmanifest.fragment.xml"),
    replacements,
  );
  console.log("  Created Package.appxmanifest.fragment.xml");

  copyTemplate(join(templateDir, "README.md"), join(widgetDir, "README.md"), replacements);
  console.log("  Created README.md");

  const { publisher } = detectWindowsPublisher(cwd);
  writeFileSync(join(widgetDir, "DevCert.ps1"), windowsDevCertScript(publisher), "utf-8");
  console.log(`  Created DevCert.ps1 (Publisher=${publisher})`);

  writeFileSync(
    join(widgetDir, ".gitignore"),
    "bin/\nobj/\n*.user\n.vs/\nAppPackages/\nBundleArtifacts/\ndev.pfx\n",
    "utf-8",
  );
  console.log("  Created .gitignore");

  console.log(`
Done! Next steps:

  1. Merge Package.appxmanifest.fragment.xml into your MSIX Package.appxmanifest
     (add xmlns:com and xmlns:uap3 if missing). Copy Assets/StoreLogo.png into the package.
     Set Identity Publisher — then regenerate cert:
     npx tauri-widgets signing --dev-cert

  2. Point the provider at the Rust host store:
     set TAURI_WIDGETS_DATA to the directory that contains widget_data.json

  3. Build smoke (no VS Appx tools):
     dotnet build ${opts.dir}/WidgetProvider/WidgetProvider.csproj -c Release -p:Smoke=true

  4. Full provider (after VS Build Tools / WinAppSDK):
     dotnet build ${opts.dir}/WidgetProvider/WidgetProvider.csproj -c Release -p:Smoke=false

  5. Pack / sideload helpers (from plugin repo / UTM):
     just win-pack && just win-sideload

  6. Dev certificate (elevated PowerShell on Windows):
     powershell -ExecutionPolicy Bypass -File ${opts.dir}/DevCert.ps1

  Display name: ${displayName}
  Package name: ${packageName}
  CLSID:        ${clsid}
`);
}

export const initWindows = defineCommand({
  meta: {
    name: "init-windows",
    description: "Initialize Windows Widgets Board provider (C# Adaptive Cards) in src-tauri/windows-widget/",
  },
  args: {
    "display-name": {
      type: "string",
      description: "Widget display name shown on Widgets Board",
    },
    "package-name": {
      type: "string",
      description: "Assembly / package name stem (e.g. MyApp). Auto-detected from tauri.conf.json if omitted.",
    },
    clsid: {
      type: "string",
      description: "COM CLSID for IWidgetProvider (GUID). Generated if omitted.",
    },
    dir: {
      type: "string",
      description: "Target directory (default: src-tauri/windows-widget)",
      default: "src-tauri/windows-widget",
    },
    force: {
      type: "boolean",
      description: "Overwrite existing files",
      default: false,
    },
  },
  run({ args }) {
    initWindowsAction({
      displayName: args["display-name"],
      packageName: args["package-name"],
      clsid: args.clsid,
      dir: args.dir,
      force: args.force,
    });
  },
});
