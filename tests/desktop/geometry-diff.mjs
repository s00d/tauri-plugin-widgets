#!/usr/bin/env node
/**
 * Level-1 geometry gate: Android vs Desktop etalon.
 *
 * Desktop baselines are the etalon. Android trees use different View kinds and
 * density, so we compare:
 *   1) both baselines exist
 *   2) no literal "null"/"undefined" text
 *   3) shared text leaves: centers within ±TOL px after scaling Android→Desktop
 *   4) overlap: ≥ MIN_OVERLAP of desktop texts must appear on Android
 *      (allowlist can waive fixture-level quirks)
 *
 * Usage: node tests/desktop/geometry-diff.mjs
 * Env: GEOMETRY_TOL=2  GEOMETRY_MIN_OVERLAP=0.5
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "../..");
const GEO_DIR = path.join(REPO_ROOT, "tests/expected/geometry");
const TOL = Number(process.env.GEOMETRY_TOL || 8);
const MIN_OVERLAP = Number(process.env.GEOMETRY_MIN_OVERLAP || 0.5);

function listFixtureIds() {
  const fixturesRoot = path.join(REPO_ROOT, "tests/fixtures");
  const out = [];
  for (const folder of ["core", "bugs", "presets"]) {
    const dir = path.join(fixturesRoot, folder);
    if (!fs.existsSync(dir)) continue;
    for (const name of fs.readdirSync(dir).filter((f) => f.endsWith(".json")).sort()) {
      const abs = path.join(dir, name);
      const raw = JSON.parse(fs.readFileSync(abs, "utf8"));
      const sizes = ["small", "medium", "large"].filter((s) => raw[s] != null);
      for (const size of sizes) {
        out.push({ id: `${folder}/${name.replace(/\.json$/, "")}`, size });
      }
    }
  }
  return out;
}

function loadAllowlist() {
  const p = path.join(GEO_DIR, "allowlist.json");
  if (!fs.existsSync(p)) return { exact: new Set(), fixtures: new Set() };
  const raw = JSON.parse(fs.readFileSync(p, "utf8"));
  const exact = new Set();
  const fixtures = new Set();
  for (const e of raw.entries || []) {
    const key = `${e.fixture}|${e.size}`;
    if (e.waive === true || e.path === "*") fixtures.add(key);
    else exact.add(`${key}|${e.path || "*"}`);
  }
  return { exact, fixtures };
}

function normalize(node) {
  if (!node) return null;
  const kids = (node.children || []).map(normalize).filter(Boolean);
  const rect = (node.rect || [0, 0, 0, 0]).map((n) => Math.round(Number(n)));
  if (rect[2] <= 0 && rect[3] <= 0 && kids.length === 0 && !node.text) return null;
  const out = { kind: String(node.kind || "?"), rect };
  if (node.text) out.text = String(node.text);
  if (kids.length) out.children = kids;
  return out;
}

function textLeaves(node, acc = []) {
  if (!node) return acc;
  if (node.text && node.text !== "✕") acc.push({ text: node.text, rect: node.rect });
  for (const c of node.children || []) textLeaves(c, acc);
  return acc;
}

function scaleRect(rect, sx, sy) {
  return [
    Math.round(rect[0] * sx),
    Math.round(rect[1] * sy),
    Math.round(rect[2] * sx),
    Math.round(rect[3] * sy),
  ];
}

function center(rect) {
  return [rect[0] + rect[2] / 2, rect[1] + rect[3] / 2];
}

function allowed(allow, id, size, pathKey) {
  const fk = `${id}|${size}`;
  return (
    allow.fixtures.has(fk) ||
    allow.exact.has(`${fk}|*`) ||
    allow.exact.has(`${fk}|${pathKey}`)
  );
}

function comparePair(id, size, desktop, android, allow) {
  const errs = [];
  const dRoot = desktop.rect;
  const aRoot = android.rect;
  if (!dRoot?.[2] || !aRoot?.[2]) {
    errs.push("missing root rect");
    return errs;
  }
  const sx = dRoot[2] / aRoot[2];
  const sy = dRoot[3] / aRoot[3];

  const dLeaves = textLeaves(desktop);
  const aLeaves = textLeaves(android).map((l) => ({
    text: l.text,
    rect: scaleRect(l.rect, sx, sy),
  }));

  for (const l of [...dLeaves, ...aLeaves]) {
    if (l.text === "null" || l.text === "undefined") {
      const key = `literal-null:${l.text}`;
      if (!allowed(allow, id, size, key)) errs.push(key);
    }
  }

  if (allowed(allow, id, size, "*")) return errs;

  // Canvas / image-only fixtures may have zero text on one side.
  if (dLeaves.length === 0) return errs;

  const byText = new Map();
  for (const l of aLeaves) {
    if (!byText.has(l.text)) byText.set(l.text, []);
    byText.get(l.text).push(l);
  }

  let matched = 0;
  for (const dl of dLeaves) {
    const pool = byText.get(dl.text) || [];
    if (pool.length === 0) continue;
    matched++;
    let best = 0;
    let bestDist = Infinity;
    const [dcx, dcy] = center(dl.rect);
    for (let i = 0; i < pool.length; i++) {
      const [acx, acy] = center(pool[i].rect);
      const dist = (dcx - acx) ** 2 + (dcy - acy) ** 2;
      if (dist < bestDist) {
        bestDist = dist;
        best = i;
      }
    }
    const al = pool.splice(best, 1)[0];
    const [acx, acy] = center(al.rect);
    const dx = Math.abs(dcx - acx);
    const dy = Math.abs(dcy - acy);
    // Soft center gate: ±TOL on each axis after density normalize.
    if (dx > TOL || dy > TOL) {
      const key = `center:${dl.text}`;
      if (!allowed(allow, id, size, key)) {
        errs.push(`${key}: Δ=(${dx.toFixed(1)},${dy.toFixed(1)}) tol=${TOL}`);
      }
    }
  }

  const overlap = matched / dLeaves.length;
  if (overlap + 1e-9 < MIN_OVERLAP) {
    const key = `overlap:${overlap.toFixed(2)}`;
    if (!allowed(allow, id, size, key) && !allowed(allow, id, size, "overlap")) {
      const missing = [...new Set(dLeaves.map((l) => l.text))].filter(
        (t) => !aLeaves.some((a) => a.text === t),
      );
      errs.push(
        `${key} < ${MIN_OVERLAP} (matched ${matched}/${dLeaves.length}; missing: ${missing.slice(0, 8).join(", ")})`,
      );
    }
  }

  return errs;
}

function main() {
  const allow = loadAllowlist();
  const cases = listFixtureIds();
  let failed = 0;
  const report = [];

  for (const { id, size } of cases) {
    const safe = id.replace(/\//g, "__");
    const deskPath = path.join(GEO_DIR, `${safe}.${size}.desktop.json`);
    const andPath = path.join(GEO_DIR, `${safe}.${size}.android.json`);
    if (!fs.existsSync(deskPath)) {
      report.push(`SKIP ${id}[${size}]: missing desktop baseline`);
      continue;
    }
    if (!fs.existsSync(andPath)) {
      report.push(`FAIL ${id}[${size}]: missing android baseline`);
      failed++;
      continue;
    }
    const desktop = normalize(JSON.parse(fs.readFileSync(deskPath, "utf8")));
    const android = normalize(JSON.parse(fs.readFileSync(andPath, "utf8")));
    const errs = comparePair(id, size, desktop, android, allow);
    if (errs.length) {
      failed++;
      report.push(`FAIL ${id}[${size}]:\n  - ${errs.join("\n  - ")}`);
    } else {
      report.push(`OK   ${id}[${size}]`);
    }
  }

  console.log(report.join("\n"));
  if (failed) {
    console.error(
      `\ngeometry-diff: ${failed} fixture(s) failed (Desktop etalon, center ±${TOL}px, overlap≥${MIN_OVERLAP})`,
    );
    process.exit(1);
  }
  console.log(`\ngeometry-diff: all checked pairs ok (center ±${TOL}px, overlap≥${MIN_OVERLAP})`);
}

main();
