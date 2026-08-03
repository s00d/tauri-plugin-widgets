import { existsSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

export interface WindowsPublisher {
  publisher: string;
  path: string | null;
}

export function detectWindowsPublisher(cwd: string): WindowsPublisher {
  const candidates = [
    join(cwd, "src-tauri", "windows-widget", "Package.appxmanifest"),
    join(cwd, "src-tauri", "Package.appxmanifest"),
    join(cwd, "Package.appxmanifest"),
  ];
  // Also scan windows-widget for any .appxmanifest*
  const winDir = join(cwd, "src-tauri", "windows-widget");
  if (existsSync(winDir)) {
    try {
      for (const e of readdirSync(winDir)) {
        if (/\.appxmanifest/i.test(e)) candidates.push(join(winDir, e));
      }
    } catch {
      /* ignore */
    }
  }
  for (const p of candidates) {
    if (!existsSync(p)) continue;
    const raw = readFileSync(p, "utf-8");
    const m = raw.match(/\bPublisher="([^"]+)"/i);
    if (m?.[1]) return { publisher: m[1], path: p };
  }
  return { publisher: "CN=TauriWidgetsDev", path: null };
}

/** RFC4122 v4-ish GUID for COM CLSID placeholders. */
export function cryptoRandomGuid(): string {
  const bytes = new Uint8Array(16);
  for (let i = 0; i < 16; i++) bytes[i] = Math.floor(Math.random() * 256);
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;
  const hex = [...bytes].map((b) => b.toString(16).padStart(2, "0")).join("");
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}
