import { readFileSync, writeFileSync } from "node:fs";

export function replaceAll(str: string, search: string, replacement: string): string {
  return str.split(search).join(replacement);
}

export function copyTemplate(
  src: string,
  dest: string,
  replacements: Record<string, string>,
): void {
  let content = readFileSync(src, "utf-8");
  for (const [search, replacement] of Object.entries(replacements)) {
    content = replaceAll(content, search, replacement);
  }
  writeFileSync(dest, content, "utf-8");
}

export interface WidgetTemplateOptions {
  widgetKind?: string;
  widgetId?: string;
}

export function widgetTemplateReplacements(
  appGroup: string,
  opts: WidgetTemplateOptions = {},
): Record<string, string> {
  const widgetKind = opts.widgetKind || "MyTauriWidget";
  const widgetId = opts.widgetId || "default";
  return {
    "{{APP_GROUP}}": appGroup,
    "{{WIDGET_KIND}}": widgetKind,
    "{{WIDGET_ID}}": widgetId,
    "group.com.example.myapp": appGroup,
  };
}
