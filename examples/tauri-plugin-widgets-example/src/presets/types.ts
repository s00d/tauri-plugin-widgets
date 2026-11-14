import type { WidgetConfig } from "tauri-plugin-widgets-api";

export type LogFn = (message: string, isError?: boolean) => void;
/** Writes the widget's local state (`setWidgetState`, no reload: the rebuild after the action reloads). */
export type SetStateFn = (patch: Record<string, unknown>) => Promise<unknown>;

export interface PresetDef {
  icon: string;
  name: string;
  config: WidgetConfig;
  /** Rebuild config (actions / live). Alone does not start a timer. */
  builder?: () => WidgetConfig | Promise<WidgetConfig>;
  /** When > 0 with `builder`, demo polls this often. Omit for action-only builders. */
  intervalMs?: number;
  /** App round-trip taps (`action`). Taps that only need `set` never get here. */
  onAction?: (action: string, payload: string | undefined, log: LogFn, setState: SetStateFn) => void | Promise<void>;
}
