// Differential test: the TypeScript port against the Rust reference (src/resolve.rs) on
// generated configs. Counterexamples belong in tests/fixtures/resolve/ (all ports run them).
import { execFileSync } from "node:child_process";
import { join } from "node:path";
import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { mergeData, nextChangeMs, resolveConfig, sourceRequest } from "../../widget-src/resolve";

const root = join(__dirname, "../..");
// Mutation runs (Stryker workers) mutate only the TS side; skip the Rust oracle there.
const skip = !!process.env.STRYKER_MUTATOR_WORKER;

const text = fc.stringMatching(/^[a-zA-Z0-9{}. _-]{0,16}$/);
const placeholder = fc.constantFrom(
  "{{t.k}}", "{{data.n}}", "{{data.s}}", "{{data.list.1}}", "{{locale}}", "{{instance}}", "{{x}}",
  "{{item}}", "{{index}}", "{{arg.v}}", "{{t}}",
  // Not `{{state}}`: a whole object's key order differs between ports (documented).
  "{{state.on}}", "{{state.k}}", "{{state.list.0}}", "{{statex}}",
);
const str = fc.oneof(text, placeholder, fc.tuple(text, placeholder, text).map(([a, b, c]) => a + b + c));
const leaf = fc.oneof(
  str.map((s) => ({ type: "text", content: s })),
  fc.record({ type: fc.constant("progress"), value: fc.double({ min: 0, max: 1, noNaN: true }), bind: fc.constantFrom({ value: "data.n" }, { value: "state.k" }) }),
  fc.record({ type: fc.constant("toggle"), isOn: fc.boolean(), bind: fc.constant({ isOn: "state.on" }), set: fc.constant({ on: "$toggle", k: "{{state.k}}" }) }),
);
// Contract v2: structured `when`, `repeat` (incl. a loop var shadowing `t`), `use`.
const cond = fc.oneof(
  fc.record({ path: fc.constantFrom("data.n", "data.s", "data.list", "data.missing", "item", "index", "state.on", "state.missing") }),
  fc.record({ path: fc.constantFrom("data.n", "item", "index"), gt: fc.integer({ min: -2, max: 2 }) }),
  fc.record({ path: fc.constantFrom("data.s", "locale", "data.n"), eq: fc.oneof(text, fc.integer({ min: 0, max: 2 }), fc.boolean()) }),
  fc.record({ path: fc.constantFrom("data.missing", "data.n"), exists: fc.boolean() }),
  fc.record({ not: fc.record({ path: fc.constant("data.n") }) }),
);
const withWhen = <T extends object>(el: fc.Arbitrary<T>) =>
  fc.tuple(el, fc.option(cond, { nil: undefined })).map(([e, w]) => (w === undefined ? e : { ...e, when: w }));
const element = fc.letrec((tie) => ({
  node: fc.oneof(
    { depthSize: "small", withCrossShrink: true },
    withWhen(leaf),
    withWhen(fc.array(tie("node"), { maxLength: 3 }).map((c) => ({ type: "vstack", children: c }))),
    fc.record({
      type: fc.constant("repeat"),
      each: fc.constantFrom("data.list", "data.s", "data.missing", "state.list"),
      as: fc.constantFrom("item", "t", "state"),
      limit: fc.integer({ min: 0, max: 5 }),
      template: tie("node"),
    }),
    fc.record({ type: fc.constant("use"), template: fc.constantFrom("row", "none"), args: fc.record({ v: fc.oneof(str, fc.constant("{{data.n}}")) }) }),
  ),
})).node;
const number = fc.oneof(fc.integer({ min: -1000, max: 1_000_000 }), fc.double({ min: -1e6, max: 1e6, noNaN: true, noDefaultInfinity: true }));
const config = fc.record({
  small: element,
  entries: fc.array(fc.record({ at: fc.integer({ min: 0, max: 10_000 }), small: element }), { maxLength: 4 }),
  strings: fc.dictionary(fc.constantFrom("en", "ru", "pt-BR", "zh-TW"), fc.dictionary(fc.constantFrom("k", "x"), text)),
  templates: fc.record({ row: fc.record({ type: fc.constant("text"), content: fc.constant("{{arg.v}}|{{index}}") }) }),
  data: fc.record({ n: number, s: text, list: fc.array(text, { maxLength: 3 }) }),
  state: fc.record({ on: fc.boolean(), k: number, list: fc.array(text, { maxLength: 3 }) }, { requiredKeys: [] }),
  source: fc.record({
    url: fc.constantFrom("https://a/{{instance}}", "http://localhost:1/{{locale}}", "http://localhost.evil/x", "ftp://x"),
    refreshMinutes: fc.integer({ min: 0, max: 3000 }),
    method: fc.constantFrom("GET", "post"),
  }),
});
const ctx = fc.record({
  nowMs: fc.integer({ min: 0, max: 12_000 }),
  locale: fc.constantFrom("en", "ru-RU", "pt_BR", "zh-Hant-TW", ""),
  instance: fc.constantFrom("i1", "nyc"),
  state: fc.option(fc.record({ on: fc.boolean(), k: number }, { requiredKeys: [] }), { nil: undefined }),
});

describe.skipIf(skip)("TS port = Rust reference (fast-check)", () => {
  // Given 300 generated configs and contexts, when both the TS port and the Rust reference resolve them, then resolved output, next change and source request are equal.
  it("300 generated configs agree", () => {
    const cases = fc.sample(fc.record({ config, ctx }), { numRuns: 300, seed: 42 });
    const input = cases.map((c) => JSON.stringify(c)).join("\n");
    const out = execFileSync(join(process.env.CARGO_TARGET_DIR ?? join(root, "target"), "debug/resolve-stdin"), [], { cwd: root, input, encoding: "utf8", maxBuffer: 64 << 20 })
      .trim()
      .split("\n")
      .map((l) => JSON.parse(l));
    expect(out.length).toBe(cases.length);
    cases.forEach((raw, i) => {
      // Same bytes as Rust saw (JSON has no -0 / undefined).
      const c = JSON.parse(JSON.stringify(raw));
      const data = mergeData([c.config.data, undefined]);
      const tsCtx = { ...c.ctx, data, state: mergeData([c.config.state, c.ctx.state]) };
      const want = out[i];
      const got = {
        resolved: resolveConfig(c.config, tsCtx),
        nextChangeMs: nextChangeMs(c.config, c.ctx.nowMs),
        source: sourceRequest(c.config, tsCtx),
      };
      if (got.source) got.source = { ...got.source, headers: Object.fromEntries(Object.entries(got.source.headers).sort()) };
      expect(got, JSON.stringify(c)).toEqual(want);
    });
  }, 120_000);
});
