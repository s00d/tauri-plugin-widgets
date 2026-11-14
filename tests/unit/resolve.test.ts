import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import fc from "fast-check";
import { afterAll, describe, expect, it } from "vitest";
import {
  activeLayouts,
  interpolate,
  mergeData,
  nextChangeMs,
  pickLocaleTable,
  resolveConfig,
  sourceRequest,
  DEFAULT_REFRESH_MINUTES,
  MAX_SOURCE_BYTES,
  sourceUrlAllowed,
} from "../../widget-src/resolve";

/* eslint-disable @typescript-eslint/no-explicit-any */
const dir = join(__dirname, "../fixtures/resolve");

/** Fixture format v2: run unless `platforms` leaves "ts" out or `skip.ts` gives a reason. */
function runsHere(c: { platforms?: string[]; skip?: Record<string, string> }): boolean | string {
  if (typeof c.skip?.ts === "string" && c.skip.ts.trim()) return false;
  if (!Array.isArray(c.platforms) || c.platforms.includes("ts")) return true;
  return "not in `platforms` and no `skip.ts` reason";
}

describe("shared resolve fixtures (same files as Rust / Swift / Kotlin)", () => {
  const files = readdirSync(dir).filter((f) => f.endsWith(".json")).sort();
  const ran: string[] = [];
  // Given the shared fixture files, when each resolves, then output, next change and source match the expected JSON.
  it("has fixtures", () => expect(files.length).toBeGreaterThanOrEqual(5));
  for (const f of files) {
    const c = JSON.parse(readFileSync(join(dir, f), "utf8"));
    const runs = runsHere(c);
    if (runs === false) {
      it.skip(`${f} (${c.skip.ts})`, () => {});
      continue;
    }
    it(f, () => {
      expect(runs, f).toBe(true);
      const ctx = {
        nowMs: c.ctx.nowMs,
        locale: c.ctx.locale ?? "",
        instance: c.ctx.instance ?? null,
        data: mergeData([c.config.data, c.ctx.data]),
        state: mergeData([c.config.state, c.ctx.state]),
      };
      expect(resolveConfig(c.config, ctx)).toEqual(c.expected);
      expect(nextChangeMs(c.config, ctx.nowMs)).toBe(c.nextChangeMs ?? null);
      if ("source" in c) expect(sourceRequest(c.config, ctx)).toEqual(c.source);
      ran.push(f);
    });
  }
  // Executed-case manifest.
  afterAll(() => console.log(`resolve fixtures ran (ts): ${ran.join(", ")}`));
});

// Scenario chains: each test walks one behaviour area step by step, asserting along the way.

const ctx = (nowMs = 0, locale = "en", data?: unknown, state?: unknown) => ({ nowMs, locale, instance: null, data, state });
const R = (config: any, data?: any, state?: any, locale = "en"): any => resolveConfig(config, ctx(0, locale, data, state));
const text = (content: string) => ({ type: "text", content });
const vstack = (children: any[]) => ({ type: "vstack", children });

describe("schedule: a day of entries", () => {
  // Given a base layout and unordered / equal-at entries, when time passes each `at`, then the active layout and next change follow.
  it("walks the timeline: base, equal-at entries, last entry, then nothing", () => {
    const cfg = {
      small: text("a"),
      entries: [
        { at: 300, small: text("c") },
        { at: 100, small: text("b") },
        { at: 100, medium: text("m") },
        { at: -1, small: text("neg") },
        { small: text("noat") },
        null,
      ],
    };
    // Before the first entry: base layout, next change at 100.
    expect(activeLayouts(cfg, 99).small.content).toBe("a");
    expect(nextChangeMs(cfg, 99)).toBe(100);
    // At 100: equal `at`s, the later entry wins (Rust max_by_key keeps the last maximum).
    expect(activeLayouts(cfg, 100)).toEqual({ small: text("a"), medium: text("m") });
    expect(nextChangeMs(cfg, 100)).toBe(300);
    // At 300: last entry, no further change.
    expect(activeLayouts(cfg, 300).small.content).toBe("c");
    expect(nextChangeMs(cfg, 300)).toBeNull();
    // Resolve follows the same timeline.
    expect(resolveConfig(cfg, ctx(300)).small).toEqual(text("c"));
    expect("medium" in resolveConfig(cfg, ctx(99))).toBe(false);
  });
  // Given edge entries (at 0, unordered, null family, empty), when layouts are picked, then base and entries combine safely.
  it("edge entries: at 0, unordered, null family, empty configs", () => {
    expect(activeLayouts({ small: "a", entries: [{ at: 0, small: "z" }] }, 0)).toEqual({ small: "z" });
    expect(nextChangeMs({ entries: [{ at: 100 }, { at: 300 }, { at: 200 }] }, 0)).toBe(100);
    expect(activeLayouts({ small: 1, entries: [{ at: 0, small: null }] }, 0)).toEqual({ small: 1 });
    expect(activeLayouts({}, 5)).toEqual({});
    expect(activeLayouts(null, 5)).toEqual({});
  });
});

describe("locale: picking the strings table, then rendering with it", () => {
  // Given string tables and device tags, when a table is picked, then exact tag > language > defaultLocale > en > first.
  it("fallback chain from exact tag down to the first table", () => {
    const cfg = { defaultLocale: "de", strings: { en: {}, de: {}, "pt-BR": {}, ru: {} } };
    expect(pickLocaleTable(cfg, "pt_BR")).toBe("pt-BR");
    expect(pickLocaleTable(cfg, "PT-pt")).toBe("pt-BR");
    expect(pickLocaleTable(cfg, "ru-RU")).toBe("ru");
    expect(pickLocaleTable(cfg, "fr")).toBe("de");
    expect(pickLocaleTable(cfg, "")).toBe("de");
    expect(pickLocaleTable({ strings: { fr: {}, en: {} } }, "ja")).toBe("en");
    expect(pickLocaleTable({ strings: { fr: {}, de: {} } }, "ja")).toBe("de");
    expect(pickLocaleTable({ defaultLocale: "xx", strings: { fr: {}, de: {} } }, "ja")).toBe("de");
    expect(pickLocaleTable({}, "en")).toBeNull();
    expect(pickLocaleTable(null, "en")).toBeNull();
    expect(pickLocaleTable({ strings: {} }, "en")).toBeNull();
    // Exact tag beats same language; exact language beats a regional variant.
    expect(pickLocaleTable({ strings: { "pt-BR": {}, "pt-PT": {} } }, "pt-PT")).toBe("pt-PT");
    expect(pickLocaleTable({ strings: { "PT-BR": {}, pt: {} } }, "pt-AO")).toBe("pt");
    // Language is the part before "-", not the first letter.
    expect(pickLocaleTable({ strings: { pl: {}, pt: {} } }, "pt-BR")).toBe("pt");
    expect(pickLocaleTable({ strings: { de: {}, ru: {} } }, " ru ")).toBe("ru");
    // defaultLocale wins over sort order; "en" over the first table.
    expect(pickLocaleTable({ defaultLocale: "fr", strings: { de: {}, fr: {} } }, "ja")).toBe("fr");
    expect(pickLocaleTable({ strings: { fr: {}, en: {}, de: {} } }, "ja")).toBe("en");
  });
  // Given tags with extensions and script subtags, when a table is picked, then tags are normalised before matching.
  it("tag normalisation: extensions, script subtags, exact match", () => {
    // An empty or extension-only tag picks nothing by itself (a "" table is not "no locale").
    const empty = { strings: { "": {}, en: {} } };
    expect(pickLocaleTable(empty, "")).toBe("en");
    expect(pickLocaleTable(empty, "-u-ca")).toBe("en");
    // The earliest extension cuts: `-x-` before `-u-`.
    expect(pickLocaleTable({ strings: { de: {}, "de-AT": {} } }, "de-AT-x-1-u-2")).toBe("de-AT");
    // Only a 4-letter subtag right after the language is a script.
    expect(pickLocaleTable({ strings: { pt: {}, "pt-BR": {} } }, "pt-BR-abcd")).toBe("pt");
    expect(pickLocaleTable({ strings: { pt: {}, "pt-BR": {} } }, "pt-abcde-br")).toBe("pt");
    expect(pickLocaleTable({ strings: { zhtw: {}, zh: {} } }, "zh-tw")).toBe("zh");
    // The exact tag beats a same language-region table with another script.
    expect(pickLocaleTable({ strings: { "zh-Hans-TW": {}, "zh-Hant-TW": {} } }, "zh-Hant-TW")).toBe("zh-Hant-TW");
  });
  // Given tables missing a key, when `{{t.key}}` renders, then the default table, then the key itself, is used.
  it("missing string falls back to the default table, then to the key", () => {
    const cfg = { defaultLocale: "en", strings: { en: { a: "A", b: "B" }, ru: { a: "А" } } };
    expect(interpolate(cfg, ctx(0, "ru"), "{{t.a}}{{t.b}}{{t.c}}")).toBe("АBc");
    expect(interpolate(cfg, ctx(0, "en"), "{{t.a}}{{t.b}}")).toBe("AB");
    expect(interpolate({}, ctx(0, "ru"), "{{t.a}}")).toBe("a");
  });
});

describe("placeholders: data layers in, text out", () => {
  // Given data layers, when they merge and a new layer arrives, then the text re-renders with the later values.
  it("merges data layers, renders values, then re-renders after a new layer", () => {
    // Layers: later wins per key; non-objects replace; inputs are not mutated.
    expect(mergeData([{ x: 1, y: 1 }, null, undefined, { y: 2 }])).toEqual({ x: 1, y: 2 });
    expect(mergeData([{ x: 1 }, 5])).toBe(5);
    expect(mergeData([5, { y: 2 }])).toEqual({ y: 2 });
    expect(mergeData([[1], { y: 2 }])).toEqual({ y: 2 });
    expect(mergeData([])).toBeUndefined();
    const a = { x: 1 };
    mergeData([a, { x: 2 }]);
    expect(a).toEqual({ x: 1 });

    const cfg = {
      small: {
        type: "text",
        content:
          "{{data.t}}|{{data.f}}|{{data.b}}|{{data.n}}|{{data.list.1}}|{{data.o}}|{{data.missing}}|{{data.list.9}}|{{ data.t }}|{{data.list.x}}|{{data.t.deep}}",
      },
    };
    const base = { t: 72.0, f: 1.5, b: true, n: null, list: ["x", "y"], o: { k: 1 } };
    expect(resolveConfig(cfg, ctx(0, "en", mergeData([base]))).small.content).toBe('72|1.5|true||y|{"k":1}|||72||');
    const pushed = mergeData([base, { t: 10, missing: "now" }]);
    expect(resolveConfig(cfg, ctx(0, "en", pushed)).small.content).toBe('10|1.5|true||y|{"k":1}|now||10||');
    // Scalar and absent data roots.
    expect(resolveConfig({ small: { c: "{{data}}" } }, ctx(0, "en", 5)).small.c).toBe("5");
    expect(resolveConfig({ small: { c: "{{data.a}}" } }, ctx(0, "en", undefined)).small.c).toBe("");
  });
  // Given odd paths, braces and roots, when text interpolates, then unknown parts stay literal and known ones fill.
  it("paths, brace edge cases, unknown roots, instance and version", () => {
    const data = { list: Array.from({ length: 11 }, (_, i) => `i${i}`), s: "abc" };
    const r = (t: string) => interpolate({}, ctx(0, "en", data), t);
    // Only digits index arrays; strings have no properties.
    expect(r("{{data.list.10}}")).toBe("i10");
    expect(r("[{{data.list.1e0}}]")).toBe("[]");
    expect(r("[{{data.s.length}}]")).toBe("[]");
    expect(r("}}{{locale}}")).toBe("}}en");
    expect(r("{{locale}} a}}b")).toBe("en a}}b");
    // Unknown roots (incl. `index` outside a loop) and unclosed braces stay; arrays and numbers walk.
    const out = resolveConfig({ small: { c: "{{oops}} {{data.x", s: "{{S}}|{{index}}", n: 3, a: ["{{locale}}", 1, null] } }, ctx(0, "ru"));
    expect(out).toEqual({ version: 1, small: { c: "{{oops}} {{data.x", s: "{{S}}|{{index}}", n: 3, a: ["ru", 1, null] } });
    expect("medium" in out).toBe(false);
    expect(interpolate({}, { nowMs: 0, locale: "", instance: "nyc" }, "{{instance}}")).toBe("nyc");
    expect(interpolate({}, { nowMs: 0, locale: "" }, "[{{instance}}]")).toBe("[]");
    expect(resolveConfig({ version: 2 }, ctx())).toEqual({ version: 2 });
    expect(resolveConfig({}, ctx())).toEqual({ version: 1 });
    expect(resolveConfig(null, ctx())).toStrictEqual({ version: 1 });
    expect(MAX_SOURCE_BYTES).toBe(262144);
  });
});

describe("when: one widget, conditions re-checked as data changes", () => {
  // Each row is [label, condition, shown with `before`, shown with `after`].
  const rows: Array<[string, any, boolean, boolean]> = [
    ["on", { path: "data.on" }, true, false],
    ["on-trimmed", { path: " data.on " }, true, false],
    ["missing", { path: "data.missing" }, false, true],
    ["null", { path: "data.n" }, false, false],
    ["empty-str", { path: "data.e" }, false, true],
    ["unknown-root", { path: "nope" }, false, false], // missing, not the state object
    ["bad-path", { path: 5 }, false, false],
    ["no-path", { eq: 1 }, false, false],
    ["eq-null", { path: "data.n", eq: null }, false, false], // null counts as missing
    ["exists-null", { path: "data.n", exists: true }, false, false],
    ["not-exists", { path: "data.missing", exists: false }, true, false],
    ["eq1", { path: "data.x", eq: 1 }, true, false],
    ["eq2", { path: "data.x", eq: 2 }, false, false],
    ["missing-eq", { path: "data.missing", eq: 1 }, false, false],
    ["ne1", { path: "data.x", ne: 1 }, false, true],
    ["ne2", { path: "data.x", ne: 2 }, true, true],
    ["missing-ne", { path: "data.missing", ne: 1 }, false, true],
    ["in1", { path: "data.x", in: [1] }, true, false],
    ["in2", { path: "data.x", in: [2] }, false, false],
    ["null-in", { path: "data.n", in: [null] }, false, false],
    ["in-not-array", { path: "data.x", in: "1" }, false, false],
    ["str-gt", { path: "data.s", gt: 3 }, false, false], // ordering needs two numbers
    ["strs-gt", { path: "data.sb", gt: "a" }, false, false],
    ["inf-gt", { path: "data.inf", gt: 3 }, false, false],
    ["x-gte3", { path: "data.x", gte: 3 }, false, true],
    ["x-lt3", { path: "data.x", lt: 3 }, true, false],
    ["x-lte3", { path: "data.x", lte: 3 }, true, false],
    ["x-gt3", { path: "data.x", gt: 3 }, false, true],
    ["three-gte", { path: "data.three", gte: 3 }, true, true],
    ["five-gte", { path: "data.five", gte: 3 }, true, true],
    ["three-lt", { path: "data.three", lt: 3 }, false, false],
    ["five-lt", { path: "data.five", lt: 3 }, false, false],
    ["three-lte", { path: "data.three", lte: 3 }, true, true],
    ["five-lte", { path: "data.five", lte: 3 }, false, false],
    ["every-op", { path: "data.x", gt: 0, lt: 0 }, false, false],
    ["obj-eq", { path: "data.o", eq: { b: 2, a: 1 } }, true, true], // key order ignored
    ["obj-ne", { path: "data.o", eq: { a: 1, b: 3 } }, false, false],
    ["list-eq", { path: "data.l", eq: [{ b: 1, a: 2 }] }, true, true],
    ["all1", { all: [{ path: "data.on" }] }, true, false],
    ["all2", { all: [{ path: "data.on" }, { path: "data.missing" }] }, false, false],
    ["any2", { any: [{ path: "data.missing" }, { path: "data.on" }] }, true, true],
    ["any1", { any: [{ path: "data.missing" }] }, false, true],
  ];
  const cfg = { small: vstack(rows.map(([label, when]) => ({ ...text(label), when }))) };
  const shown = (data: any) => R(cfg, data, { a: 1 }).small.children.map((c: any) => c.content);
  const fixed = { n: null, s: "5", sb: "b", three: 3, five: 5, inf: Infinity, o: { a: 1, b: 2 }, l: [{ a: 2, b: 1 }] };

  // Given rows with every `when` operator, when data changes, then exactly the rows whose condition holds render.
  it("renders the rows whose condition holds, then re-renders after data changes", () => {
    const before = { ...fixed, on: true, e: "", x: 1 };
    expect(shown(before)).toEqual(rows.filter((r) => r[2]).map((r) => r[0]));
    // `on` turns off, `missing` appears, `e` gets text, x jumps to 4.
    const after = { ...fixed, on: false, missing: "here", e: "t", x: 4 };
    expect(shown(after)).toEqual(rows.filter((r) => r[3]).map((r) => r[0]));
  });
  // Given a root whose `when` fails, when it resolves, then the family key is omitted.
  it("a dropped root is omitted, not undefined", () => {
    expect(R({ small: { ...text("x"), when: { path: "data.missing" } } })).toStrictEqual({ version: 1 });
    expect(R({ small: { ...text("x"), when: { path: "data.on" } } }, { on: 1 }).small).toEqual(text("x"));
  });
});

describe("task list widget: templates + repeat + when + state + bind, then requires", () => {
  const cfg: any = {
    strings: { en: { title: "Tasks", n: 5 }, ru: { title: "Задачи" } },
    templates: {
      row: {
        type: "hstack",
        children: [
          text("{{index}}. {{arg.task.title}}"),
          { type: "toggle", isOn: false, bind: { isOn: "arg.task.done" } },
          { type: "use", template: "badge", when: { path: "arg.task.urgent" } },
        ],
      },
      badge: text("!"),
    },
    small: vstack([
      text("{{t.title}}"),
      {
        type: "repeat", each: "data.tasks", as: "task", limit: 3, when: { path: "state.show" },
        template: { type: "use", template: "row", args: { task: "{{task}}" } },
      },
      { ...text("empty"), when: { not: { path: "data.tasks" } } },
    ]),
  };
  const row = (i: number, title: string, done: boolean, urgent = false) => ({
    type: "hstack",
    children: [text(`${i}. ${title}`), { type: "toggle", isOn: done }, ...(urgent ? [text("!")] : [])],
  });
  const tasks = [
    { title: "A", done: true },
    { title: "B", done: false, urgent: true },
    { title: "C", done: false },
    { title: "D", done: true },
  ];

  // Given a task-list config, when state, data, locale and the required contract change, then the rows, empty row, strings and fallback follow.
  it("resolves, reacts to state, data and locale, then falls back on a newer contract", () => {
    // 1. Shown, four tasks, limit 3: index survives `use`, args keep JSON type, bind is typed.
    let out = R(cfg, { tasks }, { show: true });
    expect(out.small.children).toEqual([text("Tasks"), row(0, "A", true), row(1, "B", false, true), row(2, "C", false)]);
    // 2. State hides the list; tasks exist, so no "empty" either.
    out = R(cfg, { tasks }, { show: false });
    expect(out.small.children).toEqual([text("Tasks")]);
    // 3. No tasks: the repeat yields nothing and the "empty" row appears.
    out = R(cfg, { tasks: [] }, { show: true });
    expect(out.small.children).toEqual([text("Tasks"), text("empty")]);
    // 4. Another locale changes only the strings.
    out = R(cfg, { tasks: tasks.slice(0, 1) }, { show: true }, "ru");
    expect(out.small.children).toEqual([text("Задачи"), row(0, "A", true)]);
    // 5. A config that needs a newer contract shows its fallback (with strings) ...
    const newer = { ...cfg, requires: { resolve: 99 }, fallback: { small: text("{{t.title}}: update") } };
    expect(R(newer, { tasks }, { show: true })).toEqual({ version: 1, small: text("Tasks: update") });
    // 6. ... or the built-in notice when it has none.
    expect(R({ ...newer, fallback: undefined })).toEqual({ version: 1, small: text("Update the app to show this widget") });
  });

  // Given `repeat` variants, when they resolve, then when / each / template / name / limit / root and shadowing rules hold.
  it("loop rules: when, each, template, loop name, limit, root repeat, shadowing", () => {
    const data = { l: ["a", "b"] };
    const ab = [text("a"), text("b")];
    const rep = (extra: any) =>
      R({ small: vstack([{ type: "repeat", each: "data.l", template: text("{{item}}"), ...extra }]) }, data).small.children;
    expect(rep({ when: { path: "data.l" } })).toEqual(ab);
    expect(rep({ when: { path: "data.missing" } })).toEqual([]);
    // Bad `each` / missing template give nothing.
    for (const bad of [{ each: 5 }, { each: ["a"] }, { each: "data.missing" }, { template: undefined }]) expect(rep(bad)).toEqual([]);
    // Loop name defaults to `item` for absent / empty / non-string `as`.
    for (const as of [undefined, "", 5]) expect(rep({ as })).toEqual(ab);
    // limit: 0 → none, 1 → one, negative or non-integer → no limit.
    expect(rep({ limit: 0 })).toEqual([]);
    expect(rep({ limit: 1 })).toEqual([text("a")]);
    expect(rep({ limit: -1 })).toEqual(ab);
    expect(rep({ limit: "1" })).toEqual(ab);
    // A root repeat becomes a vstack.
    expect(R({ small: { type: "repeat", each: "data.l", template: text("{{item}}") } }, data).small).toEqual(vstack(ab));
    // Placeholders inside a loop.
    const one = (as: string, tpl: string) =>
      R({ small: vstack([{ type: "repeat", each: "data.l", as, template: text(tpl) }]) }, { l: [{ a: 1 }] }).small.children[0].content;
    expect(one("t", "{{t.a}}|{{t.zz}}")).toBe("1|"); // `t` shadows strings; missing field is empty, not the key
    expect(one("it", "{{oops}}|{{a.b}}")).toBe("{{oops}}|{{a.b}}");
    expect(one("a.b", "{{a.b}}")).toBe("{{a.b}}"); // a dotted loop name is not a path root
    expect(one("fo", "{{foo}}")).toBe("{{foo}}");
  });

  // Given `use` templates, when they resolve, then lookup, caller-scope `when`, typed args and the depth cap hold.
  it("template rules: lookup, caller-scope when, args, recursion cap", () => {
    const templates = {
      t: text("ok"),
      b: { ...text("lit"), bind: { content: "arg.v" } },
      r: vstack([{ type: "use", template: "r" }]),
      "5": text("n"),
    };
    const use = (el: any, data?: any) => R({ templates, small: { type: "use", ...el } }, data).small;
    // Lookup: unknown name, non-string name, no templates at all → nothing.
    expect(use({ template: "zz" })).toBeUndefined();
    expect(use({ template: 5 })).toBeUndefined();
    expect(R({ small: { type: "use", template: "t" } })).toStrictEqual({ version: 1 });
    // `when` on the use element runs in the caller's scope (`arg` is the caller's, not {x:1}).
    expect(use({ template: "t", when: { path: "data.missing" } })).toBeUndefined();
    expect(use({ template: "t", when: { path: "data.on" } }, { on: 1 })).toEqual(text("ok"));
    expect(use({ template: "t", args: { x: 1 }, when: { not: { path: "arg.x" } } })).toEqual(text("ok"));
    // Args: literals keep type; a whole `{{expr}}` is typed (trimmed); missing → ""; mixed → text.
    expect(use({ template: "b", args: { v: 5 } }).content).toBe(5);
    expect(use({ template: "b", args: { v: "{{ data.n }}" } }, { n: 7 }).content).toBe(7);
    expect(use({ template: "b", args: { v: "{{data.missing}}" } }).content).toBe("");
    expect(use({ template: "b", args: { v: "{{index}}" } }).content).toBe("");
    expect(use({ template: "b", args: { v: "n={{data.n}}" } }, { n: 7 }).content).toBe("n=7");
    // Self-recursion stops at depth 8.
    let node = use({ template: "r" });
    let depth = 0;
    for (; node && node.type === "vstack"; depth++) node = node.children[0];
    expect(depth).toBe(8);
  });

  // Given `bind` maps, when they resolve, then data / loop / index values are typed, t.* is text and missing keeps the literal.
  it("bind rules: typed for data / loop / index, text for t.*, literal kept when missing", () => {
    const out = R({
      strings: { en: { n: 5 } },
      small: vstack([{
        type: "repeat", each: "data.l",
        template: { ...text("c"), bind: { i: "index", v: "item", s: "t.n", m: "t.missing", d: " data.k " } },
      }]),
    }, { l: [3], k: 9 });
    expect(out.small.children).toStrictEqual([{ type: "text", content: "c", i: 0, v: 3, s: "5", m: "missing", d: 9 }]);
    const b = (bind: any) => R({ small: { ...text("c"), bind } }, { n: null }).small;
    expect(b({ content: "data.missing" })).toStrictEqual(text("c"));
    expect(b({ content: "data.n" })).toStrictEqual(text("c"));
    expect(b({ content: 5 })).toStrictEqual(text("c"));
    expect(b({ content: "index" })).toStrictEqual(text("{{index}}")); // no loop: not a known root
  });
});

describe("source: URL policy, then the request a config asks for", () => {
  // Given `source` configs, when the request is built, then only https / dev localhost pass and placeholders and clamping apply.
  it("allows https and dev localhost only, then builds the request", () => {
    for (const bad of ["http://localhost.evil.com/a", "http://localhost@evil.com/a", "http://127.0.0.1.nip.io/a", "http://localhost:80x/a", "file://localhost"]) {
      expect(sourceUrlAllowed(bad)).toBe(false);
    }
    expect(sourceUrlAllowed("http://LOCALHOST")).toBe(true);
    expect(sourceUrlAllowed("http://127.0.0.1:8080?x=1")).toBe(true);

    const get = (src: unknown) => sourceRequest({ source: src }, ctx(0, "ru"));
    expect(get({ url: "http://example.com" })).toBeNull();
    expect(get({ url: "ftp://x" })).toBeNull();
    expect(get({})).toBeNull();
    expect(get(null)).toBeNull();
    expect(sourceRequest({}, ctx())).toBeNull();
    expect(sourceRequest(null, ctx())).toBeNull();
    expect(get({ url: "http://localhost:1/a" })).toEqual({
      url: "http://localhost:1/a", method: "GET", headers: {}, body: null, refreshMinutes: DEFAULT_REFRESH_MINUTES,
    });
    expect(get({ url: "http://127.0.0.1/a" })).not.toBeNull();
    // Placeholders in url / headers / body; refresh clamped to [15, 1440].
    expect(get({ url: "https://a/{{locale}}", method: "post", body: "{{locale}}", refreshMinutes: 1, headers: { A: "{{locale}}", B: 1 } })).toEqual({
      url: "https://a/ru", method: "POST", headers: { A: "ru" }, body: "ru", refreshMinutes: 15,
    });
    expect(get({ url: "https://a", body: "x", refreshMinutes: 99999 })).toMatchObject({ body: null, refreshMinutes: 1440 });
    expect(get({ url: "https://a", refreshMinutes: -5 })?.refreshMinutes).toBe(DEFAULT_REFRESH_MINUTES);
    expect(get({ url: "https://a", refreshMinutes: 20 })?.refreshMinutes).toBe(20);
    expect(get({ url: "https://a", refreshMinutes: 0 })?.refreshMinutes).toBe(15);
    expect(get({ url: "https://a", method: "POST" })?.body).toBeNull();
    expect(get({ url: "https://a", method: "GET", body: "x" })).toMatchObject({ method: "GET", body: null });
  });
});

describe("state: model-based run of set patches, scope clears and redraws", () => {
  // The host applies `set` patches (src/store.rs `apply_state_patch`); this is its spec as a
  // test double. Under test: the renderer side, i.e. stored layers → mergeData → resolveConfig.
  type Obj = Record<string, unknown>;
  type Scope = "widget" | "instance";
  const KEYS = ["on", "n", "label"] as const;
  const step = (op: unknown, now: unknown): unknown => {
    if (op === "$toggle") return now !== true;
    if (op === "$inc" || op === "$dec") return (typeof now === "number" ? now : 0) + (op === "$inc" ? 1 : -1);
    return op;
  };
  const hostPatch = (layer: Obj, current: Obj, patch: Obj) => {
    for (const [k, v] of Object.entries(patch)) {
      if (v === null) delete layer[k];
      else layer[k] = step(v, current[k]);
    }
  };

  const cfgState = { on: false, n: 0 }; // no `label` default: it can be missing
  const config = {
    state: cfgState,
    small: vstack([
      text("n={{state.n}} label={{state.label}}"),
      { type: "toggle", isOn: false, bind: { isOn: "state.on" }, set: { on: "$toggle" } },
      { type: "button", label: "+1", set: { n: "$inc", label: "{{state.n}}" } },
      { ...text("on"), when: { path: "state.on" } },
      { ...text("big"), when: { path: "state.n", gt: 2 } },
      { ...text("labelled"), when: { path: "state.label", exists: true } },
    ]),
  };

  /** Model: plain per-scope objects; the effective value is looked up by precedence. */
  type Model = { widget: Obj; instance: Obj };
  const eff = (m: Model, placement: Scope, k: string): unknown => {
    const layers: Obj[] = placement === "instance" ? [m.instance, m.widget, cfgState] : [m.widget, cfgState];
    for (const l of layers) if (Object.prototype.hasOwnProperty.call(l, k)) return l[k];
    return undefined;
  };
  const txt = (v: unknown) => (v === undefined || v === null ? "" : typeof v === "string" ? v : JSON.stringify(v));
  const predict = (m: Model, p: Scope) => {
    const [on, n, label] = KEYS.map((k) => eff(m, p, k));
    const shown = (v: unknown) => !(v === undefined || v === null || v === false || v === 0 || v === "");
    return vstack([
      text(`n=${txt(n)} label=${txt(label)}`),
      { type: "toggle", isOn: on === undefined || on === null ? false : on, set: { on: "$toggle" } },
      { type: "button", label: "+1", set: { n: "$inc", label: txt(n) } },
      ...(shown(on) ? [text("on")] : []),
      ...(typeof n === "number" && n > 2 ? [text("big")] : []),
      ...(label !== undefined && label !== null ? [text("labelled")] : []),
    ]);
  };

  /** Real: the shared store holds JSON strings per key, like `state:{id}` / `state:{id}#{i}`. */
  type Real = { store: Map<Scope, string> };
  const layer = (r: Real, s: Scope): Obj => JSON.parse(r.store.get(s) ?? "{}");
  const merged = (r: Real, p: Scope) =>
    mergeData([config.state, ...(p === "instance" ? ["widget", "instance"] : ["widget"]).map((s) => layer(r, s as Scope))]);

  class SetPatch implements fc.Command<Model, Real> {
    constructor(readonly scope: Scope, readonly patch: Obj) {}
    check = () => true;
    run(m: Model, r: Real) {
      // Operators read the merged state the patching placement sees.
      const modelCurrent = Object.fromEntries(KEYS.map((k) => [k, eff(m, this.scope, k)]));
      hostPatch(m[this.scope], modelCurrent, this.patch);
      const l = layer(r, this.scope);
      hostPatch(l, merged(r, this.scope), this.patch);
      r.store.set(this.scope, JSON.stringify(l));
      invariant(m, r);
    }
    toString = () => `SetPatch(${this.scope}, ${JSON.stringify(this.patch)})`;
  }
  class ClearScope implements fc.Command<Model, Real> {
    constructor(readonly scope: Scope) {}
    check = () => true;
    run(m: Model, r: Real) {
      m[this.scope] = {};
      r.store.delete(this.scope);
      invariant(m, r);
    }
    toString = () => `ClearScope(${this.scope})`;
  }
  const render = (r: Real, p: Scope) =>
    resolveConfig(config, { nowMs: 0, locale: "en", instance: p === "instance" ? "i1" : null, state: merged(r, p) });
  /** Invariant after every step: both placements render what the model predicts. */
  const invariant = (m: Model, r: Real) => {
    for (const p of ["widget", "instance"] as const) expect(render(r, p)).toStrictEqual({ version: 1, small: predict(m, p) });
  };

  class Resolve implements fc.Command<Model, Real> {
    constructor(readonly placement: Scope) {}
    check = () => true;
    run(m: Model, r: Real) {
      expect(render(r, this.placement)).toStrictEqual({ version: 1, small: predict(m, this.placement) });
    }
    toString = () => `Resolve(${this.placement})`;
  }

  const scope = fc.constantFrom<Scope>("widget", "instance");
  const op = fc.oneof(
    fc.constantFrom("$toggle", "$inc", "$dec", null, true, false, "", "x"),
    fc.integer({ min: -3, max: 6 }),
  );
  const patch = fc.dictionary(fc.constantFrom(...KEYS), op, { minKeys: 1, maxKeys: 3 });
  const commands = fc.commands(
    [
      fc.tuple(scope, patch).map(([s, p]) => new SetPatch(s, p)),
      scope.map((s) => new ClearScope(s)),
      scope.map((s) => new Resolve(s)),
    ],
    { maxCommands: 30 },
  );

  // Given generated set / clear / resolve sequences, when run on the store and a model, then both placements render as the model predicts after every step.
  it("rendered output always matches the model's precedence lookup", () => {
    fc.assert(
      fc.property(commands, (cmds) => {
        fc.modelRun(() => ({ model: { widget: {}, instance: {} }, real: { store: new Map() } }), cmds);
      }),
      { numRuns: 150 },
    );
  });
});
