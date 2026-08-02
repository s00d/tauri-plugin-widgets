import fs from "node:fs";
import path from "node:path";
import { defineCommand } from "citty";
import sharp from "sharp";

async function loadOrBlank(p: string | undefined, label: string, w = 360, h = 380) {
  if (p && fs.existsSync(p)) {
    return sharp(p)
      .resize(w, h, { fit: "contain", background: { r: 20, g: 20, b: 24, alpha: 1 } })
      .png()
      .toBuffer();
  }
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}">
    <rect width="100%" height="100%" fill="#141418"/>
    <text x="50%" y="50%" fill="#888" font-size="18" text-anchor="middle" dominant-baseline="middle">${label}: missing</text>
  </svg>`;
  return sharp(Buffer.from(svg)).png().toBuffer();
}

async function contactSheet(opts: {
  android?: string;
  ios?: string;
  desktop?: string;
  out: string;
}) {
  const w = 360;
  const h = 380;
  const labelH = 28;
  const panels = await Promise.all([
    loadOrBlank(opts.android, "Android", w, h),
    loadOrBlank(opts.ios, "iOS", w, h),
    loadOrBlank(opts.desktop, "Desktop", w, h),
  ]);
  const labels = ["Android", "iOS", "Desktop"];
  const composites: sharp.OverlayOptions[] = [];
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
    .toFile(opts.out);
  return opts.out;
}

async function askVision(contactPath: string, configJson: string) {
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
  const body = (await res.json()) as {
    choices?: { message?: { content?: string } }[];
  };
  const text = body.choices?.[0]?.message?.content || "[]";
  const match = text.match(/\[[\s\S]*\]/);
  return match ? JSON.parse(match[0]) : [];
}

export const triageCommand = defineCommand({
  meta: {
    name: "triage",
    description:
      "Build Android|iOS|Desktop contact sheet for visual triage (optional OpenAI vision)",
  },
  args: {
    android: { type: "string", description: "Android PNG path" },
    ios: { type: "string", description: "iOS PNG path" },
    desktop: { type: "string", description: "Desktop PNG path" },
    config: { type: "string", description: "Fixture JSON path" },
    out: {
      type: "string",
      default: path.join(process.cwd(), "triage-contact.png"),
      description: "Output contact sheet path",
    },
  },
  async run({ args }) {
    if (!args.android && !args.desktop && !args.ios) {
      console.error(
        "Usage: pnpm -C scripts cli triage --android a.png --desktop d.png [--ios i.png] [--config fixture.json] [--out contact.png]",
      );
      process.exit(2);
    }

    const out = args.out;
    await contactSheet({
      android: args.android,
      ios: args.ios,
      desktop: args.desktop,
      out,
    });
    console.log(`contact sheet → ${out}`);

    const configJson =
      args.config && fs.existsSync(args.config) ? fs.readFileSync(args.config, "utf8") : "{}";

    try {
      const findings = await askVision(out, configJson);
      if (findings) console.log(JSON.stringify(findings, null, 2));
    } catch (e) {
      console.error("vision triage failed:", e instanceof Error ? e.message : e);
      process.exitCode = 1;
    }
  },
});
