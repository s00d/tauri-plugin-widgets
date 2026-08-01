#!/usr/bin/env node
/**
 * Batch visual audit contact sheets for all declarative cases.
 *
 * Panels: Desktop | iOS | macOS | Android | Windows | Linux
 * Output: out/audit/<case>.png + out/audit/index.html
 *
 *   pnpm audit:sheets
 *   node --experimental-strip-types tools/audit-sheets.ts --case weather.small
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import sharp from "sharp";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(__dirname, "..");
const CASES = path.join(REPO, "tests/cases");
const GOLDEN = path.join(REPO, "tests/golden");
const OUT = path.join(REPO, "out/audit");
const OUT_LINUX = path.join(REPO, "out/linux");

const PLATFORMS = [
  { id: "desktop", label: "Desktop" },
  { id: "ios", label: "iOS" },
  { id: "macos", label: "macOS" },
  { id: "android", label: "Android" },
  { id: "windows", label: "Windows" },
  { id: "linux", label: "Linux" },
];

function arg(name, fallback) {
  const i = process.argv.indexOf(`--${name}`);
  if (i >= 0 && process.argv[i + 1]) return process.argv[i + 1];
  return fallback;
}

async function loadOrBlank(p, label, w, h) {
  if (p && fs.existsSync(p)) {
    return sharp(p)
      .resize(w, h, {
        fit: "contain",
        background: { r: 20, g: 20, b: 24, alpha: 1 },
      })
      .png()
      .toBuffer();
  }
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}">
    <rect width="100%" height="100%" fill="#141418"/>
    <text x="50%" y="50%" fill="#888" font-size="16" text-anchor="middle" dominant-baseline="middle">${label}: missing</text>
  </svg>`;
  return sharp(Buffer.from(svg)).png().toBuffer();
}

async function sheetForCase(caseName, panelW = 280, panelH = 300) {
  const labelH = 26;
  const titleH = 32;
  const n = PLATFORMS.length;
  const composites = [];

  const titleSvg = Buffer.from(
    `<svg xmlns="http://www.w3.org/2000/svg" width="${panelW * n}" height="${titleH}">
      <rect width="100%" height="100%" fill="#0b0b0e"/>
      <text x="12" y="22" fill="#f3f4f6" font-size="16" font-family="ui-sans-serif,system-ui">${caseName}</text>
    </svg>`,
  );
  composites.push({
    input: await sharp(titleSvg).png().toBuffer(),
    left: 0,
    top: 0,
  });

  for (let i = 0; i < n; i++) {
    const { id, label } = PLATFORMS[i];
    let pngPath = path.join(GOLDEN, id, `${caseName}.png`);
    // Linux triage shots land in out/linux/<case>-<size>.png before record-linux.
    if (id === "linux" && !fs.existsSync(pngPath)) {
      const size = caseName.split(".").pop() || "small";
      const alt = path.join(OUT_LINUX, `${caseName}.png`);
      const alt2 = path.join(OUT_LINUX, `${caseName.replace(/\.(small|medium|large)$/, "")}-${size}.png`);
      if (fs.existsSync(alt)) pngPath = alt;
      else if (fs.existsSync(alt2)) pngPath = alt2;
      else {
        const hits = fs.existsSync(OUT_LINUX)
          ? fs.readdirSync(OUT_LINUX).filter((f) => f.startsWith(caseName) && f.endsWith(".png"))
          : [];
        if (hits[0]) pngPath = path.join(OUT_LINUX, hits[0]);
      }
    }
    const panel = await loadOrBlank(pngPath, label, panelW, panelH);
    const labelSvg = Buffer.from(
      `<svg xmlns="http://www.w3.org/2000/svg" width="${panelW}" height="${labelH}">
        <rect width="100%" height="100%" fill="#111113"/>
        <text x="10" y="18" fill="#d1d5db" font-size="13" font-family="ui-sans-serif,system-ui">${label}</text>
      </svg>`,
    );
    composites.push({
      input: await sharp(labelSvg).png().toBuffer(),
      left: i * panelW,
      top: titleH,
    });
    composites.push({
      input: panel,
      left: i * panelW,
      top: titleH + labelH,
    });
  }

  const outPath = path.join(OUT, `${caseName}.png`);
  await sharp({
    create: {
      width: panelW * n,
      height: titleH + labelH + panelH,
      channels: 4,
      background: { r: 11, g: 11, b: 14, alpha: 1 },
    },
  })
    .composite(composites)
    .png()
    .toFile(outPath);
  return outPath;
}

function writeIndex(caseNames) {
  const cards = caseNames
    .map(
      (name) => `
    <figure>
      <figcaption><code>${name}</code></figcaption>
      <a href="./${name}.png" target="_blank"><img src="./${name}.png" alt="${name}" loading="lazy"/></a>
    </figure>`,
    )
    .join("\n");
  const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <title>Widget visual audit</title>
  <style>
    body { margin: 0; padding: 24px; background: #0b0b0e; color: #e5e7eb; font-family: ui-sans-serif, system-ui, sans-serif; }
    h1 { font-size: 20px; font-weight: 600; margin: 0 0 8px; }
    p { color: #9ca3af; margin: 0 0 24px; font-size: 14px; }
    main { display: grid; gap: 28px; }
    figure { margin: 0; }
    figcaption { margin-bottom: 8px; font-size: 13px; }
    img { max-width: 100%; height: auto; border-radius: 8px; border: 1px solid #27272a; }
    code { color: #fafafa; }
  </style>
</head>
<body>
  <h1>Visual audit — Desktop | iOS | macOS | Android</h1>
  <p>${caseNames.length} cases · checklist: padding, background, color, overflow/clip, empty/uniform</p>
  <main>${cards}</main>
</body>
</html>`;
  fs.writeFileSync(path.join(OUT, "index.html"), html);
}

async function main() {
  const only = arg("case");
  fs.mkdirSync(OUT, { recursive: true });
  const names = fs
    .readdirSync(CASES)
    .filter((f) => f.endsWith(".json"))
    .map((f) => f.replace(/\.json$/, ""))
    .sort()
    .filter((n) => !only || n === only);

  if (names.length === 0) {
    console.error("no cases found");
    process.exit(2);
  }

  for (const name of names) {
    const p = await sheetForCase(name);
    console.log(p);
  }
  writeIndex(names);
  console.log(`index → ${path.join(OUT, "index.html")} (${names.length} cases)`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
