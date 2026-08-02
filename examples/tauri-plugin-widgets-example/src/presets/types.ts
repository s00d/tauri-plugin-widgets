import type { WidgetConfig } from "tauri-plugin-widgets-api";

export type LogFn = (message: string, isError?: boolean) => void;

export interface PresetDef {
  icon: string;
  name: string;
  config: WidgetConfig;
  /** Rebuild config (actions / live). Alone does not start a timer. */
  builder?: () => WidgetConfig | Promise<WidgetConfig>;
  /** When > 0 with `builder`, demo polls this often. Omit for action-only builders. */
  intervalMs?: number;
  onAction?: (action: string, payload: string | undefined, log: LogFn) => void;
}
