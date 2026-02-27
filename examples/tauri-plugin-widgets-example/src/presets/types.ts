import type { WidgetConfig } from "tauri-plugin-widgets-api";

export type LogFn = (message: string, isError?: boolean) => void;

export interface PresetDef {
  icon: string;
  name: string;
  config: WidgetConfig;
  builder?: () => WidgetConfig | Promise<WidgetConfig>;
  intervalMs?: number;
  onAction?: (action: string, payload: string | undefined, log: LogFn) => void;
}
