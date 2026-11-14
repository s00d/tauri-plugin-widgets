// Port of `src/resolve.rs`: scheduled entries and `{{…}}` placeholders. Checked against
// the same fixtures (`tests/fixtures/resolve/*.json`) as the Rust, Swift and Kotlin ports.

/* eslint-disable @typescript-eslint/no-explicit-any */
type Json = any;

export type ResolveContext = {
  /** Unix ms. */
  nowMs: number;
  /** BCP-47 or ICU tag. */
  locale: string;
  instance?: string | null;
  /** Data layers already merged with {@link mergeData}. */
  data?: Json;
  /** Local state already merged: config `state`, `state:{id}`, `state:{id}#{instance}`. */
  state?: Json;
};

const FAMILIES = ["small", "medium", "large", "accessory"] as const;

function entries(config: Json): Array<[number, Json]> {
  const list = config && Array.isArray(config.entries) ? config.entries : [];
  const out: Array<[number, Json]> = [];
  for (const e of list) {
    if (e && Number.isInteger(e.at) && e.at >= 0) out.push([e.at, e]);
  }
  return out;
}

function present(v: Json): boolean {
  return v !== undefined && v !== null;
}

/** Base layouts with the active entry (last `at <= now`) applied. */
export function activeLayouts(config: Json, nowMs: number): Record<string, Json> {
  const out: Record<string, Json> = {};
  for (const f of FAMILIES) if (config && present(config[f])) out[f] = config[f];
  let active: Json = null;
  let activeAt = -1;
  for (const [at, e] of entries(config)) {
    // `>=` keeps the last of equal `at`s, like Rust's `max_by_key`.
    if (at <= nowMs && at >= activeAt) {
      active = e;
      activeAt = at;
    }
  }
  if (active) for (const f of FAMILIES) if (present(active[f])) out[f] = active[f];
  return out;
}

/** First entry `at` after now, or null. */
export function nextChangeMs(config: Json, nowMs: number): number | null {
  let next: number | null = null;
  for (const [at] of entries(config)) if (at > nowMs && (next === null || at < next)) next = at;
  return next;
}

/** Shallow merge; later layers win per top-level key; a non-object replaces all before it. */
export function mergeData(layers: Json[]): Json {
  let out: Json = undefined;
  for (const layer of layers) {
    if (layer === undefined || layer === null) continue;
    if (isObject(out) && isObject(layer)) out = Object.assign({}, out, layer);
    else out = isObject(layer) ? Object.assign({}, layer) : layer;
  }
  return out;
}

function isObject(v: Json): boolean {
  return v !== null && typeof v === "object" && !Array.isArray(v);
}

/** Lower case, `_` → `-`, Unicode / private-use extensions (`-u-…`, `-x-…`) dropped. */
function normalizeLocale(tag: string): string {
  const t = tag.trim().replace(/_/g, "-").toLowerCase();
  const cuts = ["-u-", "-x-"].map((m) => t.indexOf(m)).filter((i) => i >= 0);
  return cuts.length ? t.slice(0, Math.min(...cuts)) : t;
}

/** `zh-hant-tw` → `zh-tw`: the tag without a 4-letter script subtag. */
function languageRegion(tag: string): string {
  return tag
    .split("-")
    .filter((p, i) => !(i === 1 && /^[a-z]{4}$/.test(p)))
    .join("-");
}

function language(tag: string): string {
  return tag.split("-")[0];
}

/** `strings` table name for a locale (same fallback chain as Rust). */
export function pickLocaleTable(config: Json, locale: string): string | null {
  const tables = config && isObject(config.strings) ? config.strings : null;
  if (!tables) return null;
  const keys = Object.keys(tables).sort();
  if (keys.length === 0) return null;
  const find = (pred: (k: string) => boolean) => keys.find((k) => pred(normalizeLocale(k))) ?? null;
  const want = normalizeLocale(locale || "");
  if (want) {
    const lang = language(want);
    const lr = languageRegion(want);
    const hit =
      find((k) => k === want) ??
      find((k) => languageRegion(k) === lr) ??
      find((k) => k === lang) ??
      find((k) => language(k) === lang);
    if (hit) return hit;
  }
  if (typeof config.defaultLocale === "string") {
    const def = normalizeLocale(config.defaultLocale);
    const hit = find((k) => k === def);
    if (hit) return hit;
  }
  return find((k) => k === "en") ?? keys[0];
}

function lookupPath(root: Json, path: string): Json {
  if (path === "") return root;
  let cur = root;
  for (const seg of path.split(".")) {
    if (Array.isArray(cur)) {
      if (!/^\d+$/.test(seg)) return undefined;
      cur = cur[Number(seg)];
    } else if (isObject(cur)) {
      cur = Object.prototype.hasOwnProperty.call(cur, seg) ? cur[seg] : undefined;
    } else return undefined;
  }
  return cur;
}

function valueText(v: Json): string {
  if (v === undefined || v === null) return "";
  if (typeof v === "string") return v;
  // Numbers and booleans print the same through JSON.stringify.
  return JSON.stringify(v);
}

/** Renderer contract version: a config with `requires.resolve` above it shows `fallback`. */
export const CONTRACT_VERSION = 3;
/** Hard caps: items per `repeat`, nested `use` depth. */
const MAX_REPEAT = 50;
const MAX_TEMPLATE_DEPTH = 8;

type Scope = {
  strings: Json;
  fallback: Json;
  ctx: ResolveContext;
  templates: Json;
  /** Loop / template variables, innermost last (`task`, `arg`, …). */
  locals: Array<[string, Json]>;
  /** Index of the innermost `repeat`, if any. */
  index: number | null;
  depth: number;
};

function child(scope: Scope, name: string, value: Json, index?: number): Scope {
  return {
    ...scope,
    locals: [...scope.locals, [name, value]],
    index: index === undefined ? scope.index : index,
  };
}

/** A local variable by name, innermost first. */
function local(scope: Scope, name: string): { found: boolean; value: Json } {
  for (let i = scope.locals.length - 1; i >= 0; i--) {
    if (scope.locals[i][0] === name) return { found: true, value: scope.locals[i][1] };
  }
  return { found: false, value: undefined };
}

/** Typed value of an expression (`data.a`, `task.x`, `arg.p`, `index`, `locale`, `t.k`);
 *  undefined when missing. */
function lookupExpr(scope: Scope, raw: string): Json {
  const expr = raw.trim();
  // Loop / template variables shadow the built-in roots (`t`, `data`, …).
  const dot0 = expr.indexOf(".");
  const l0 = local(scope, dot0 < 0 ? expr : expr.slice(0, dot0));
  if (l0.found) return lookupPath(l0.value, dot0 < 0 ? "" : expr.slice(dot0 + 1));
  if (expr === "index") return scope.index === null ? undefined : scope.index;
  if (expr === "locale") return scope.ctx.locale;
  if (expr === "instance") return scope.ctx.instance || "";
  if (expr.startsWith("t.")) {
    const key = expr.slice(2);
    for (const table of [scope.strings, scope.fallback]) {
      if (isObject(table) && Object.prototype.hasOwnProperty.call(table, key)) return table[key];
    }
    return undefined;
  }
  if (expr === "data" || expr.startsWith("data.")) return lookupPath(scope.ctx.data, expr.slice(5));
  if (expr === "state" || expr.startsWith("state.")) return lookupPath(scope.ctx.state, expr.slice(6));
  return undefined;
}

/** `data…` / `state…`: roots whose values keep their JSON type. */
function isDataRoot(expr: string): boolean {
  return expr === "data" || expr.startsWith("data.") || expr === "state" || expr.startsWith("state.");
}

function placeholder(scope: Scope, raw: string): string {
  const expr = raw.trim();
  const d0 = expr.indexOf(".");
  if (local(scope, d0 < 0 ? expr : expr.slice(0, d0)).found) return valueText(lookupExpr(scope, expr));
  if (expr.startsWith("t.")) {
    const v = lookupExpr(scope, expr);
    return v === undefined ? expr.slice(2) : valueText(v);
  }
  const dot = expr.indexOf(".");
  const head = dot < 0 ? expr : expr.slice(0, dot);
  const known =
    expr === "locale" || expr === "instance" || isDataRoot(expr) ||
    (expr === "index" && scope.index !== null) || local(scope, head).found;
  if (!known) return "{{" + expr + "}}";
  return valueText(lookupExpr(scope, expr));
}

function interpolateWith(scope: Scope, s: string): string {
  let out = "";
  let rest = s;
  for (;;) {
    const start = rest.indexOf("{{");
    if (start < 0) break;
    const end = rest.indexOf("}}", start + 2);
    if (end < 0) break;
    out += rest.slice(0, start) + placeholder(scope, rest.slice(start + 2, end));
    rest = rest.slice(end + 2);
  }
  return out + rest;
}

function isNum(v: Json): boolean {
  return typeof v === "number" && Number.isFinite(v);
}

function deepEqual(a: Json, b: Json): boolean {
  if (isNum(a) && isNum(b)) return a === b;
  return JSON.stringify(canon(a)) === JSON.stringify(canon(b));
}

function canon(v: Json): Json {
  if (Array.isArray(v)) return v.map(canon);
  if (isObject(v)) {
    const o: Record<string, Json> = {};
    for (const k of Object.keys(v).sort()) o[k] = canon(v[k]);
    return o;
  }
  return v;
}

function truthy(v: Json): boolean {
  if (v === undefined || v === null || v === false || v === 0 || v === "") return false;
  if (Array.isArray(v)) return v.length > 0;
  return true;
}

/** Structured condition: `{path, eq|ne|gt|gte|lt|lte|exists|in}` (all given ops must hold),
 *  `{all:[…]}`, `{any:[…]}`, `{not:{…}}`; a bare `{path}` tests truthiness. Missing values
 *  are false (so true under `not`); ordering needs two numbers; a bad shape is false. */
export function evalWhen(scope: Scope, cond: Json): boolean {
  if (!isObject(cond)) return false;
  if (Array.isArray(cond.all)) return cond.all.every((c: Json) => evalWhen(scope, c));
  if (Array.isArray(cond.any)) return cond.any.some((c: Json) => evalWhen(scope, c));
  if ("not" in cond) return !evalWhen(scope, cond.not);
  if (typeof cond.path !== "string") return false;
  const v = lookupExpr(scope, cond.path);
  const has = v !== undefined && v !== null;
  const ops = ["eq", "ne", "gt", "gte", "lt", "lte", "exists", "in"].filter((k) => k in cond);
  if (ops.length === 0) return truthy(v);
  return ops.every((op) => {
    const x = cond[op];
    switch (op) {
      case "exists":
        return has === (x === true);
      case "eq":
        return has && deepEqual(v, x);
      case "ne":
        return has && !deepEqual(v, x);
      case "in":
        return has && Array.isArray(x) && x.some((y: Json) => deepEqual(v, y));
      default: {
        if (!isNum(v) || !isNum(x)) return false;
        if (op === "gt") return v > x;
        if (op === "gte") return v >= x;
        if (op === "lt") return v < x;
        return v <= x;
      }
    }
  });
}

/** One array element → zero or more elements (`repeat` splices, `when` drops, `use`
 *  expands). */
function expand(scope: Scope, el: Json): Json[] {
  if (!isObject(el)) return [walk(scope, el)];
  if (el.type === "repeat") {
    if ("when" in el && !evalWhen(scope, el.when)) return [];
    const items = typeof el.each === "string" ? lookupExpr(scope, el.each) : undefined;
    if (!Array.isArray(items) || !isObject(el.template)) return [];
    const name = typeof el.as === "string" && el.as ? el.as : "item";
    const lim = Number.isInteger(el.limit) && el.limit >= 0 ? Math.min(el.limit, MAX_REPEAT) : MAX_REPEAT;
    const out: Json[] = [];
    for (let i = 0; i < Math.min(items.length, lim); i++) {
      out.push(...expand(child(scope, name, items[i], i), el.template));
    }
    return out;
  }
  if (el.type === "use") {
    if ("when" in el && !evalWhen(scope, el.when)) return [];
    const tpl = isObject(scope.templates) && typeof el.template === "string" ? scope.templates[el.template] : undefined;
    if (!isObject(tpl) || scope.depth >= MAX_TEMPLATE_DEPTH) return [];
    const args: Record<string, Json> = {};
    if (isObject(el.args)) {
      for (const k of Object.keys(el.args)) args[k] = argValue(scope, el.args[k]);
    }
    // Keys on the `use` element override the template root (instance wins).
    const root: Record<string, Json> = { ...tpl };
    for (const k of Object.keys(el)) if (!["type", "template", "args", "when"].includes(k)) root[k] = el[k];
    const inner = child({ ...scope, depth: scope.depth + 1 }, "arg", args);
    return expand(inner, root);
  }
  if ("when" in el && !evalWhen(scope, el.when)) return [];
  return [walk(scope, el)];
}

/** An argument that is exactly one `{{expr}}` keeps the expression's JSON type. */
function argValue(scope: Scope, v: Json): Json {
  if (typeof v !== "string") return v;
  const m = /^\{\{([^{}]*)\}\}$/.exec(v);
  if (m) {
    const val = lookupExpr(scope, m[1]);
    return val === undefined ? "" : val;
  }
  return interpolateWith(scope, v);
}

function walk(scope: Scope, v: Json): Json {
  if (typeof v === "string") return v.includes("{{") ? interpolateWith(scope, v) : v;
  if (Array.isArray(v)) return v.flatMap((x) => expand(scope, x));
  if (isObject(v)) {
    const o: Record<string, Json> = {};
    for (const k of Object.keys(v)) if (k !== "bind" && k !== "when") o[k] = walk(scope, v[k]);
    if (isObject(v.bind)) {
      for (const field of Object.keys(v.bind)) {
        const expr = v.bind[field];
        if (typeof expr !== "string") continue;
        const val = bound(scope, expr);
        if (val !== undefined) o[field] = val;
      }
    }
    return o;
  }
  return v;
}

/** `bind: {field: expr}`: data / loop / arg values keep their JSON type; other expressions
 *  render as text; a missing value keeps the literal (undefined). */
function bound(scope: Scope, raw: string): Json {
  const expr = raw.trim();
  const dot = expr.indexOf(".");
  const head = dot < 0 ? expr : expr.slice(0, dot);
  if (isDataRoot(expr) || local(scope, head).found || (expr === "index" && scope.index !== null)) {
    const v = lookupExpr(scope, expr);
    return v === null ? undefined : v;
  }
  return placeholder(scope, expr);
}

/** A family root: `use` / `repeat` / `when` apply there too (a dropped root is omitted). */
function walkRoot(scope: Scope, v: Json): Json {
  if (isObject(v) && v.type === "repeat") return { type: "vstack", children: expand(scope, v) };
  const out = expand(scope, v);
  return out.length ? out[0] : undefined;
}

function scopeFor(config: Json, ctx: ResolveContext): Scope {
  const t = pickLocaleTable(config, ctx.locale);
  const d = pickLocaleTable(config, "");
  return {
    strings: t !== null ? config.strings[t] : null,
    fallback: d !== null ? config.strings[d] : null,
    ctx,
    templates: config && isObject(config.templates) ? config.templates : null,
    locals: [],
    index: null,
    depth: 0,
  };
}

/** Fill placeholders in one string. */
export function interpolate(config: Json, ctx: ResolveContext, s: string): string {
  return interpolateWith(scopeFor(config, ctx), s);
}

/** `requires.resolve` above {@link CONTRACT_VERSION}: the config this renderer can show. */
function effectiveConfig(config: Json): Json {
  const need = config && isObject(config.requires) ? config.requires.resolve : undefined;
  if (!(Number.isInteger(need) && need > CONTRACT_VERSION)) return config;
  const fb = isObject(config.fallback)
    ? config.fallback
    : { small: { type: "text", content: "Update the app to show this widget" } };
  return { version: config.version, strings: config.strings, defaultLocale: config.defaultLocale, ...fb };
}

/** Active entry, then templates / repeat / when / placeholders / bind. Output: `version` +
 *  size families only. */
export function resolveConfig(config: Json, ctx: ResolveContext): Json {
  const cfg = effectiveConfig(config);
  const scope = scopeFor(cfg, ctx);
  const layouts = activeLayouts(cfg, ctx.nowMs);
  const out: Record<string, Json> = { version: cfg && present(cfg.version) ? cfg.version : 1 };
  for (const f of FAMILIES) {
    if (!(f in layouts)) continue;
    const r = walkRoot(scope, layouts[f]);
    if (r !== undefined) out[f] = r;
  }
  return out;
}

export type SourceRequest = {
  url: string;
  method: "GET" | "POST";
  headers: Record<string, string>;
  body: string | null;
  refreshMinutes: number;
};

export const DEFAULT_REFRESH_MINUTES = 30;
export const MAX_SOURCE_BYTES = 256 * 1024;

/** `https://` anywhere; `http://` only to exactly `localhost` / `127.0.0.1` (development). */
export function sourceUrlAllowed(url: string): boolean {
  if (url.startsWith("https://")) return true;
  if (!url.startsWith("http://")) return false;
  const authority = url.slice(7).split(/[/?#]/)[0];
  if (authority.includes("@")) return false;
  const i = authority.indexOf(":");
  const host = i < 0 ? authority : authority.slice(0, i);
  const port = i < 0 ? "" : authority.slice(i + 1);
  return (host.toLowerCase() === "localhost" || host === "127.0.0.1") && /^\d*$/.test(port);
}

/** The fetch this config asks for, or null (only https://, plus localhost for development). */
export function sourceRequest(config: Json, ctx: ResolveContext): SourceRequest | null {
  const src = config && isObject(config.source) ? config.source : null;
  if (!src || typeof src.url !== "string") return null;
  const url = interpolate(config, ctx, src.url);
  if (!sourceUrlAllowed(url)) return null;
  const method = typeof src.method === "string" && src.method.toLowerCase() === "post" ? "POST" : "GET";
  const headers: Record<string, string> = {};
  if (isObject(src.headers)) {
    for (const k of Object.keys(src.headers)) {
      if (typeof src.headers[k] === "string") headers[k] = interpolate(config, ctx, src.headers[k]);
    }
  }
  const body = method === "POST" && typeof src.body === "string" ? interpolate(config, ctx, src.body) : null;
  const m = src.refreshMinutes;
  const refreshMinutes = Number.isInteger(m) && m >= 0 ? Math.min(1440, Math.max(15, m)) : DEFAULT_REFRESH_MINUTES;
  return { url, method, headers, body, refreshMinutes };
}
