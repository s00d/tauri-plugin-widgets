import { invoke } from "@tauri-apps/api/core";
import { PLUGIN_ID } from "./plugin";
import type { ReloadOutcome } from "./config";

/** Element the renderer skipped, with reason. */
export interface SkippedElement {
  type: string;
  reason: string;
}

/** Cross-platform render receipt (diagnostics — not a critical path). */
export interface WidgetRenderReceipt {
  widgetId: string;
  group: string;
  instance: string;
  nonce: number;
  size?: string;
  theme?: string;
  schema?: number;
  /** prefs | state | appgroup | defaults | container | push | pull */
  source: string;
  /** reload | timeline | action | added | resize | snapshot */
  trigger?: string;
  rendered?: string[];
  skipped?: SkippedElement[];
  ts: number;
}

/** One host journal entry (`WIDGET_DEBUG` / debug builds). */
export type TraceEvent =
  | {
      kind: "configSet";
      widgetId: string;
      nonce: number;
      bytes: number;
      changed: boolean;
      skip?: { reason: "unchanged"; hash: number } | { reason: "noInstances" } | {
        reason: "transportUnavailable";
        name: string;
      };
    }
  | {
      kind: "write";
      transport: string;
      ok: boolean;
      durationMs: number;
      error?: string;
    }
  | { kind: "reload"; performed: boolean; reason: ReloadOutcome }
  | { kind: "poll"; count: number }
  | {
      kind: "render";
      instance: string;
      nonce: number;
      source: string;
      trigger: string;
      lagMs: number;
      skipped: SkippedElement[];
    };

export interface TraceEntry {
  ts: number;
  kind: TraceEvent["kind"];
  [key: string]: unknown;
}

export interface WidgetTrace {
  enabled: boolean;
  events: TraceEntry[];
  receipts: WidgetRenderReceipt[];
}

/**
 * Report that a renderer painted a config (desktop widget.html / tests).
 * Native widgets write receipts themselves; this is for the host webview path.
 */
export async function reportReceipt(receipt: WidgetRenderReceipt): Promise<boolean> {
  return await invoke<boolean>(`${PLUGIN_ID}|report_receipt`, { receipt });
}

/**
 * Live widget instances that recently rendered for `group`.
 */
export async function getWidgetDiagnostics(
  group: string,
): Promise<WidgetRenderReceipt[]> {
  if (!group) throw new Error("getWidgetDiagnostics: 'group' must not be empty");
  return await invoke<WidgetRenderReceipt[]>(`${PLUGIN_ID}|get_widget_diagnostics`, {
    group,
  });
}

/**
 * Host delivery journal + receipt history for `group`.
 * Active in debug builds or when `WIDGET_DEBUG=1`.
 */
export async function getWidgetTrace(
  group: string,
  opts?: { since?: number },
): Promise<WidgetTrace> {
  if (!group) throw new Error("getWidgetTrace: 'group' must not be empty");
  return await invoke<WidgetTrace>(`${PLUGIN_ID}|get_widget_trace`, {
    group,
    sinceMs: opts?.since ?? null,
  });
}

/** Force-flush the in-memory journal to disk (desktop). */
export async function flushWidgetTrace(): Promise<boolean> {
  return await invoke<boolean>(`${PLUGIN_ID}|flush_widget_trace`);
}
