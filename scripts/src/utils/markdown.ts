/**
 * Markdown helpers for docs generation (marker blocks + tables).
 */

/** Escape a value so it survives a table cell. */
export function cell(value: unknown): string {
  return String(value ?? "")
    .replace(/\|/g, "\\|")
    .replace(/\n+/g, " ")
    .trim();
}

export function code(value: string | number | boolean | null | undefined): string {
  if (value === null || value === undefined || value === "") return "—";
  const text = cell(value);
  const longest = Math.max(0, ...[...text.matchAll(/`+/g)].map((m) => m[0].length));
  const fence = "`".repeat(longest + 1);
  const pad = text.startsWith("`") || text.endsWith("`") ? " " : "";
  return `${fence}${pad}${text}${pad}${fence}`;
}

export function table(headers: string[], rows: string[][]): string {
  if (rows.length === 0) return "";
  return [
    `| ${headers.join(" | ")} |`,
    `| ${headers.map(() => "---").join(" | ")} |`,
    ...rows.map((row) => `| ${row.join(" | ")} |`),
  ].join("\n");
}

export const BLOCK_START = (id: string): string =>
  `<!-- generated:${id} — do not edit; run \`pnpm docs:generate\` -->`;
export const BLOCK_END = (id: string): string => `<!-- /generated:${id} -->`;

export class MissingBlockError extends Error {
  constructor(public readonly id: string) {
    super(`no <!-- generated:${id} --> region`);
  }
}

/**
 * Replace the contents of the `id` region in `source`.
 * Throws when the region is absent.
 */
export function replaceBlock(source: string, id: string, body: string): string {
  const start = source.indexOf(BLOCK_START(id));
  const end = source.indexOf(BLOCK_END(id));
  if (start === -1 || end === -1 || end < start) throw new MissingBlockError(id);
  return `${source.slice(0, start)}${BLOCK_START(id)}\n\n${body.trim()}\n\n${BLOCK_END(id)}${source.slice(end + BLOCK_END(id).length)}`;
}

export function ensureBlockShell(title: string, id: string, intro = ""): string {
  return `# ${title}\n\n${intro}${intro ? "\n\n" : ""}${BLOCK_START(id)}\n\n${BLOCK_END(id)}\n`;
}
