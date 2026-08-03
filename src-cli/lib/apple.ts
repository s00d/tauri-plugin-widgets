import { existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { widgetTemplateReplacements } from "./templates.js";

export function findIosWidgetEntryFiles(cwd: string): string[] {
  const appleDir = resolve(cwd, "src-tauri", "gen", "apple");
  if (!existsSync(appleDir)) return [];

  const entries = readdirSync(appleDir, { withFileTypes: true });
  const result: string[] = [];

  for (const entry of entries) {
    if (!entry.isDirectory()) continue;
    const name = entry.name;

    if (
      name.startsWith(".") ||
      name.endsWith(".xcodeproj") ||
      name === "Externals" ||
      name === "Sources" ||
      name === "assets" ||
      name === "Assets.xcassets"
    ) {
      continue;
    }

    const dirPath = join(appleDir, name);
    const files = readdirSync(dirPath, { withFileTypes: true })
      .filter((f) => f.isFile() && f.name.endsWith(".swift"))
      .map((f) => join(dirPath, f.name));

    const candidates = files.filter((filePath) => {
      const fileName = filePath.split("/").pop() || "";
      if (fileName.endsWith("Bundle.swift")) return false;
      const content = readFileSync(filePath, "utf-8");
      return content.includes("import WidgetKit") && content.includes("WidgetConfiguration");
    });

    if (!candidates.length) continue;

    const preferred = candidates.find((p) => p.endsWith(`/${name}.swift`));
    result.push(preferred || candidates[0]);
  }

  return Array.from(new Set(result));
}

export function findIosWidgetTargetNames(cwd: string): string[] {
  const appleDir = resolve(cwd, "src-tauri", "gen", "apple");
  if (!existsSync(appleDir)) return [];

  const entries = readdirSync(appleDir, { withFileTypes: true });
  const targets = new Set<string>();

  for (const entry of entries) {
    if (!entry.isDirectory() || !entry.name.endsWith(".xcodeproj")) continue;
    const pbxprojPath = join(appleDir, entry.name, "project.pbxproj");
    if (!existsSync(pbxprojPath)) continue;

    const pbx = readFileSync(pbxprojPath, "utf-8");
    const sectionStart = pbx.indexOf("/* Begin PBXNativeTarget section */");
    const sectionEnd = pbx.indexOf("/* End PBXNativeTarget section */");
    if (sectionStart === -1 || sectionEnd === -1 || sectionEnd <= sectionStart) continue;

    const section = pbx.slice(sectionStart, sectionEnd);
    const blockRe = /\bisa = PBXNativeTarget;[\s\S]*?};/g;
    let match: RegExpExecArray | null;
    while ((match = blockRe.exec(section)) !== null) {
      const block = String(match[0] || "");
      if (!block.includes('productType = "com.apple.product-type.app-extension";')) continue;

      const nameMatch = block.match(/\bname = ([^;]+);/);
      const rawName = String(nameMatch?.[1] || "").trim();
      if (!rawName) continue;
      const cleanName = rawName.replace(/^"|"$/g, "");
      if (cleanName) targets.add(cleanName);
    }
  }

  return Array.from(targets);
}

export function findApplePbxproj(cwd: string): string[] {
  const appleDir = join(cwd, "src-tauri", "gen", "apple");
  if (!existsSync(appleDir)) return [];
  const out: string[] = [];
  for (const e of readdirSync(appleDir, { withFileTypes: true })) {
    if (!e.isDirectory() || !e.name.endsWith(".xcodeproj")) continue;
    const p = join(appleDir, e.name, "project.pbxproj");
    if (existsSync(p)) out.push(p);
  }
  return out;
}

export interface PatchIosCodeSignEntitlementsResult {
  patched: number;
}

/**
 * Ensure CODE_SIGN_ENTITLEMENTS is set on application + app-extension targets.
 * `entsByProductType` maps productType → relative path from .xcodeproj parent.
 */
export function patchIosCodeSignEntitlements(
  pbxPath: string,
  entsByProductType: Record<string, string>,
): PatchIosCodeSignEntitlementsResult {
  if (!existsSync(pbxPath)) return { patched: 0 };
  let pbx = readFileSync(pbxPath, "utf-8");
  let patched = 0;

  // Map target id → productType for native targets we care about
  const targetProduct = new Map<string, string>();
  const targetBlockRe =
    /([A-F0-9]{24})\s*\/\*\s*[^*]+\s*\*\/\s*=\s*\{\s*isa = PBXNativeTarget;[\s\S]*?productType = "([^"]+)";[\s\S]*?\};/g;
  let tm: RegExpExecArray | null;
  while ((tm = targetBlockRe.exec(pbx)) !== null) {
    const productType = tm[2];
    if (
      productType !== "com.apple.product-type.application" &&
      productType !== "com.apple.product-type.app-extension"
    ) {
      continue;
    }
    const block = tm[0];
    const listMatch = block.match(/buildConfigurationList = ([A-F0-9]{24})/);
    if (listMatch) targetProduct.set(listMatch[1], productType);
  }

  // Map configuration list → build config ids
  const configIds: { id: string; productType: string }[] = [];
  for (const [listId, productType] of targetProduct) {
    const listRe = new RegExp(
      `${listId}\\s*/\\*[^*]*\\*/\\s*=\\s*\\{[\\s\\S]*?buildConfigurations = \\(([\\s\\S]*?)\\);`,
    );
    const lm = pbx.match(listRe);
    if (!lm) continue;
    const ids = [...lm[1].matchAll(/([A-F0-9]{24})/g)].map((x) => x[1]);
    for (const id of ids) configIds.push({ id, productType });
  }

  for (const { id, productType } of configIds) {
    const ents = entsByProductType[productType];
    if (!ents) continue;
    const cfgRe = new RegExp(
      `(${id}\\s*/\\*[^*]*\\*/\\s*=\\s*\\{[\\s\\S]*?buildSettings = \\{)([\\s\\S]*?)(\\n\\t\\t\\t\\};)`,
    );
    const cm = pbx.match(cfgRe);
    if (!cm) continue;
    const settings = cm[2];
    const quoted = ents.includes(" ") || ents.includes("/") ? `"${ents}"` : ents;
    if (/CODE_SIGN_ENTITLEMENTS\s*=/.test(settings)) {
      const next = settings.replace(
        /CODE_SIGN_ENTITLEMENTS\s*=\s*[^;]+;/,
        `CODE_SIGN_ENTITLEMENTS = ${quoted};`,
      );
      if (next !== settings) {
        pbx = pbx.replace(cm[0], `${cm[1]}${next}${cm[3]}`);
        patched++;
      }
    } else {
      const inject = `\n\t\t\t\tCODE_SIGN_ENTITLEMENTS = ${quoted};`;
      pbx = pbx.replace(cm[0], `${cm[1]}${inject}${settings}${cm[3]}`);
      patched++;
    }
  }

  if (patched) writeFileSync(pbxPath, pbx, "utf-8");
  return { patched };
}

/**
 * Render an iOS widget Swift template for a given app group, adapting it for
 * Xcode-generated widget targets that already declare `@main` in `<Name>Bundle.swift`.
 */
export function renderIosWidgetSwift(templatePath: string, destPath: string, appGroup: string): string {
  let content = readFileSync(templatePath, "utf-8");
  const replacements = widgetTemplateReplacements(appGroup);
  for (const [search, replacement] of Object.entries(replacements)) {
    content = content.split(search).join(replacement);
  }

  // Xcode-generated widget targets usually include `<Name>Bundle.swift` with @main.
  // In that case this file should declare `struct <Name>: Widget` (without @main)
  // so Bundle can reference it.
  const widgetName = basename(destPath, ".swift");
  const bundlePath = join(dirname(destPath), `${widgetName}Bundle.swift`);
  if (existsSync(bundlePath)) {
    content = content.replace(/@main\s*\n\s*struct\s+MyWidget\s*:\s*Widget/, `struct ${widgetName}: Widget`);
  }

  return content;
}
