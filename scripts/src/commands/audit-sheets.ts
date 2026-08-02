import fs from "node:fs";
import path from "node:path";
import { defineCommand } from "citty";
import sharp from "sharp";
import { repoRoot } from "../utils/workspace.js";

const PLATFORMS = [
  { id: "desktop", label: "Desktop" },
  { id: "ios", label: "iOS" },
  { id: "macos", label: "macOS" },
  { id: "android", label: "Android" },
  { id: "windows", label: "Windows" },
  { id: "linux", label: "Linux" },
] as const;

async function loadOrBlank(p: string, label: string, w: number, h: number) {
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

async function sheetForCase(
  caseName: string,
  golden: string,
  outLinux: string,
  outDir: string,
  panelW = 280,
  panelH = 300,
) {
  const labelH = 26;
  const titleH = 32;
  const n = PLATFORMS.length;
  const composites: sharp.OverlayOptions[] = [];

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
    let pngPath = path.join(golden, id, `${caseName}.png`);
    if (id === "linux" && !fs.existsSync(pngPath)) {
      const size = caseName.split(".").pop() || "small";
      const alt = path.join(outLinux, `${caseName}.png`);
      const alt2 = path.join(
        outLinux,
        `${caseName.replace(/\.(small|medium|large)$/, "")}-${size}.png`,
      );
      if (fs.existsSync(alt)) pngPath = alt;
      else if (fs.existsSync(alt2)) pngPath = alt2;
      else {
        const hits = fs.existsSync(outLinux)
          ? fs.readdirSync(outLinux).filter((f) => f.startsWith(caseName) && f.endsWith(".png"))
          : [];
        if (hits[0]) pngPath = path.join(outLinux, hits[0]);
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

  const outPath = path.join(outDir, `${caseName}.png`);
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

function writeIndex(outDir: string, caseNames: string[]) {
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
  fs.writeFileSync(path.join(outDir, "index.html"), html);
}

export const auditSheetsCommand = defineCommand({
  meta: {
    name: "audit-sheets",
    description: "Build contact sheets for all (or one) visual cases → out/audit/",
  },
  args: {
    case: {
      type: "string",
      description: "Only this case name (e.g. weather.small)",
    },
  },
  async run({ args }) {
    const root = repoRoot();
    const casesDir = path.join(root, "tests/cases");
    const golden = path.join(root, "tests/golden");
    const outDir = path.join(root, "out/audit");
    const outLinux = path.join(root, "out/linux");

    fs.mkdirSync(outDir, { recursive: true });
    const names = fs
      .readdirSync(casesDir)
      .filter((f) => f.endsWith(".json"))
      .map((f) => f.replace(/\.json$/, ""))
      .sort()
      .filter((n) => !args.case || n === args.case);

    if (names.length === 0) {
      console.error("no cases found");
      process.exit(2);
    }

    for (const name of names) {
      const p = await sheetForCase(name, golden, outLinux, outDir);
      console.log(p);
    }
    writeIndex(outDir, names);
    console.log(`index → ${path.join(outDir, "index.html")} (${names.length} cases)`);
  },
});
