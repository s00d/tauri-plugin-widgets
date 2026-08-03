/** Minimal Tauri bridge used by the desktop widget webview. */
interface TauriInternals {
  invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
  transformCallback: (cb: (event: { payload: unknown }) => void) => number;
  metadata?: {
    currentWindow?: {
      label?: string;
    };
  };
}

interface Window {
  __TAURI_INTERNALS__?: TauriInternals;
}
