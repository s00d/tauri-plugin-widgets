#!/usr/bin/env node
/**
 * Vision triage for failed geometry/pixel diffs (manual — NOT a CI gate).
 *
 * Builds a contact sheet (Android | iOS | Desktop) via sharp and optionally
 * sends it to a vision model. Expects JSON:
 *   [{ platform, element, property, actual, expected }]
 *
 * Usage:
 *   node --experimental-strip-types tools/triage.ts \
 *     --android path/a.png --ios path/i.png --desktop path/d.png \
 *     --config tests/fixtures/bugs/null-fields.json \
 *     --out /tmp/contact.png
 *
 * Env:
 *   OPENAI_API_KEY / ANTHROPIC_API_KEY — optional; without keys only contact sheet is written.
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import sharp from "sharp";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

function arg(name, fallback = undefined) {
  const i = process.argv.indexOf(`--${name}`);
  if (i >= 0 && process.argv[i + 1]) return process.argv[i + 1];
  return fallback;
}

function usage() {
  console.log(`Usage: node tools/triage.ts --android a.png --desktop d.png [--ios i.png] [--config fixture.json] [--out contact.png]`);
}

async function loadOrBlank(p, label, w = 360, h = 380) {
  if (p && fs.existsSync(p)) {
    return sharp(p).resize(w, h, { fit: "contain", background: { r: 20, g: 20, b: 24, alpha: 1 } }).png().toBuffer();
  }
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}">
    <rect width="100%" height="100%" fill="#141418"/>
    <text x="50%" y="50%" fill="#888" font-size="18" text-anchor="middle" dominant-baseline="middle">${label}: missing</text>
  </svg>`;
  return sharp(Buffer.from(svg)).png().toBuffer();
}

async function contactSheet({ android, ios, desktop, out }) {
  const w = 360;
  const h = 380;
  const labelH = 28;
  const panels = await Promise.all([
    loadOrBlank(android, "Android", w, h),
    loadOrBlank(ios, "iOS", w, h),
    loadOrBlank(desktop, "Desktop", w, h),
  ]);
  const labels = ["Android", "iOS", "Desktop"];
  const composites = [];
  for (let i = 0; i < 3; i++) {
    const labelSvg = Buffer.from(
      `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${labelH}">
        <rect width="100%" height="100%" fill="#0b0b0e"/>
        <text x="12" y="18" fill="#e5e7eb" font-size="14" font-family="sans-serif">${labels[i]}</text>
      </svg>`,
    );
    composites.push({
      input: await sharp(labelSvg).png().toBuffer(),
      left: i * w,
      top: 0,
    });
    composites.push({
      input: panels[i],
      left: i * w,
      top: labelH,
    });
  }
  await sharp({
    create: {
      width: w * 3,
      height: h + labelH,
      channels: 4,
      background: { r: 11, g: 11, b: 14, alpha: 1 },
    },
  })
    .composite(composites)
    .png()
    .toFile(out);
  return out;
}

async function askVision({ contactPath, configJson }) {
  const key = process.env.OPENAI_API_KEY;
  if (!key) {
    console.log("No OPENAI_API_KEY — skipped vision call. Contact sheet only.");
    return null;
  }
  const b64 = fs.readFileSync(contactPath).toString("base64");
  const prompt = `You are triaging widget render diffs. Left=Android, middle=iOS, right=Desktop.
Config JSON:
\`\`\`json
${configJson.slice(0, 6000)}
\`\`\`
Return ONLY a JSON array of objects: [{ "platform": "android"|"ios"|"desktop", "element": string, "property": string, "actual": string, "expected": string }].
Focus on layout/geometry/color mismatches. Empty array if nothing material.`;

  const res = await fetch("https://api.openai.com/v1/chat/completions", {
    method: "POST",
    headers: {
      Authorization: `Bearer ${key}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      model: process.env.TRIAGE_MODEL || "gpt-4o-mini",
      messages: [
        {
          role: "user",
          content: [
            { type: "text", text: prompt },
            { type: "image_url", image_url: { url: `data:image/png;base64,${b64}` } },
          ],
        },
      ],
      temperature: 0,
    }),
  });
  if (!res.ok) {
    throw new Error(`vision API ${res.status}: ${await res.text()}`);
  }
  const body = await res.json();
  const text = body.choices?.[0]?.message?.content || "[]";
  const match = text.match(/\[[\s\S]*\]/);
  return match ? JSON.parse(match[0]) : [];
}

async function main() {
  const android = arg("android");
  const ios = arg("ios");
  const desktop = arg("desktop");
  const configPath = arg("config");
  const out = arg("out", path.join(process.cwd(), "triage-contact.png"));

  if (!android && !desktop && !ios) {
    usage();
    process.exit(2);
  }

  await contactSheet({ android, ios, desktop, out });
  console.log(`contact sheet → ${out}`);

  const configJson = configPath && fs.existsSync(configPath)
    ? fs.readFileSync(configPath, "utf8")
    : "{}";

  try {
    const findings = await askVision({ contactPath: out, configJson });
    if (findings) {
      console.log(JSON.stringify(findings, null, 2));
    }
  } catch (e) {
    console.error("vision triage failed:", e.message || e);
    process.exitCode = 1;
  }
}

main();
