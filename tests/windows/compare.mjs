/**
 * Compare out/windows/*.png against tests/golden/windows/*.png (same tol as desktop).
 * Usage: node tests/windows/compare.mjs [caseName]
 *
 * If no goldens exist, exits 0 with a manual-track note (WinUI3 VM required).
 */
import { readFileSync, existsSync, mkdirSync, writeFileSync, readdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "../..");
const goldenDir = join(root, "tests/golden/windows");
const outDir = join(root, "out/windows");
const TOL = 0.02;
const CHANNEL = 8;

const only = process.argv[2];
const names = only
  ? [only]
  : ["weather.small", "chart-mix.medium", "canvas.small"];

mkdirSync(outDir, { recursive: true });

const goldenFiles = existsSync(goldenDir)
  ? readdirSync(goldenDir).filter((f) => f.endsWith(".png"))
  : [];
if (goldenFiles.length === 0) {
  console.log(
    "windows: no PNG goldens under tests/golden/windows/ — track is manual (WinUI3 VM). See docs/contributing/windows-surfaces.md",
  );
  process.exit(0);
}

const { default: sharp } = await import("sharp");

async function mismatchFrac(aBuf, bBuf) {
  const a = await sharp(aBuf).ensureAlpha().raw().toBuffer({ resolveWithObject: true });
  const b = await sharp(bBuf).ensureAlpha().raw().toBuffer({ resolveWithObject: true });
  if (a.info.width !== b.info.width || a.info.height !== b.info.height) {
    return 1;
  }
  const n = a.data.length / 4;
  let bad = 0;
  for (let i = 0; i < a.data.length; i += 4) {
    for (let c = 0; c < 3; c++) {
      if (Math.abs(a.data[i + c] - b.data[i + c]) > CHANNEL) {
        bad++;
        break;
      }
    }
  }
  return bad / n;
}

const available = names.filter((name) => existsSync(join(goldenDir, `${name}.png`)));
let failed = 0;
for (const name of available) {
  const g = join(goldenDir, `${name}.png`);
  const a = join(outDir, `${name}.png`);
  if (!existsSync(a)) {
    console.error(`missing actual ${a} — run UTM shot -Mode visual or copy PNG to out/windows/`);
    failed++;
    continue;
  }
  const frac = await mismatchFrac(readFileSync(a), readFileSync(g));
  console.log(`${name}: mismatch=${frac.toFixed(4)}`);
  if (frac > TOL) {
    failed++;
    writeFileSync(join(outDir, `${name}.fail.txt`), `frac=${frac}\n`);
  }
}
if (only && available.length === 0) {
  console.error(`missing golden ${join(goldenDir, `${only}.png`)}`);
  failed++;
}
process.exit(failed ? 1 : 0);
