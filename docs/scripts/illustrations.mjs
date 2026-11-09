// Page illustrations: docs/public/illustrations/<name>.{light,dark}.svg (committed).
//
//   node scripts/illustrations.mjs            light -> dark recolor of every illustration (free)
//   node scripts/illustrations.mjs --generate install tiers
//                                             new light take from fal (recraft v3, ~$0.04 each,
//                                             needs the `um` CLI and a fal key), then recolor
//
// One generated image per illustration; the dark variant is the same SVG with the grays inverted
// onto the zinc palette, so both themes show the same picture. Emerald accents are kept.
import { execFileSync } from 'node:child_process'
import { mkdtempSync, readdirSync, readFileSync, renameSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

const dir = fileURLToPath(new URL('../public/illustrations/', import.meta.url))

const STYLE =
  'Minimal flat vector illustration for developer documentation, clean geometric shapes, thin rounded outlines, ' +
  'generous whitespace, no text, no letters, no logos. Accent color emerald green #10b981, everything else neutral ' +
  'zinc grays. Light theme: solid off-white background #fafafa, gray #3f3f46 outlines.'

export const SUBJECTS = {
  guide: 'One JSON document card in the center with lines fanning out to a phone, a tablet, a laptop and a desktop monitor, each showing a small widget card.',
  install: 'Three steps in a row connected by a line: a package crate, a terminal window, a puzzle piece plugging into an app window.',
  'first-widget': 'A desktop monitor with a small floating rounded widget card showing a sun icon and a progress bar.',
  concepts: 'A tree of nested rounded boxes (layout tree) on the left flowing by arrows into finished widget cards on the right.',
  transport: 'Two app windows exchanging a document through a shared folder with a lock, arrows in both directions.',
  tiers: 'Two stacked layers of blocks: a full bottom row of solid blocks and a top row of partly filled outlined blocks, small check marks.',
  platforms: 'A phone, a tablet, a laptop and a desktop monitor side by side, each with the same small widget card on screen.',
  updating: 'A widget card with a refresh circular arrow and a small clock, a data stream of dots flowing into it.',
  actions: 'A finger tapping a button on a widget card, a curved arrow going back to an app window.',
  images: 'A widget card containing a picture frame with mountains and sun, a few smaller image tiles around it.',
  'multi-widget': 'A phone home screen with a grid of several different widget cards of different sizes.',
}

// Grays (r≈g≈b) are mirrored onto zinc: #fafafa background -> #18181b, #1a1a1a lines -> #e4e4e7.
function darkColor(r, g, b) {
  if (Math.max(r, g, b) - Math.min(r, g, b) > 24) return [r, g, b] // accent: keep
  const v = (r + g + b) / 3
  const t = Math.min(1, Math.max(0, (250 - v) / (250 - 26)))
  const mix = (a, z) => Math.round(a + (z - a) * t)
  return [mix(24, 228), mix(24, 228), mix(27, 231)]
}

function recolor(svg) {
  return svg
    .replace(/<metadata>[\s\S]*?<\/metadata>/, '') // C2PA manifest: ~30 KB of base64
    .replace(/rgb\((\d+),\s*(\d+),\s*(\d+)\)/g, (_, r, g, b) => `rgb(${darkColor(+r, +g, +b).join(',')})`)
    .replace(/#([0-9a-fA-F]{6})\b/g, (_, hex) => {
      const [r, g, b] = darkColor(...[0, 2, 4].map((i) => parseInt(hex.slice(i, i + 2), 16)))
      return '#' + [r, g, b].map((n) => n.toString(16).padStart(2, '0')).join('')
    })
}

function generate(name) {
  const subject = SUBJECTS[name]
  if (!subject) throw new Error(`illustrations.mjs: unknown illustration ${name}`)
  const out = mkdtempSync(join(tmpdir(), 'widgets-illustration-'))
  execFileSync('um', ['fal', 'run', 'fal-ai/recraft/v3/text-to-image', '--out', out, '--name', name, `prompt=${STYLE} ${subject}`, 'style=vector_illustration', 'image_size=landscape_4_3'], { stdio: 'inherit' })
  renameSync(join(out, `${name}.svg`), join(dir, `${name}.light.svg`))
}

const args = process.argv.slice(2)
if (args[0] === '--generate') for (const name of args.slice(1)) generate(name)

for (const file of readdirSync(dir).filter((f) => f.endsWith('.light.svg'))) {
  const light = readFileSync(join(dir, file), 'utf8').replace(/<metadata>[\s\S]*?<\/metadata>/, '')
  writeFileSync(join(dir, file), light)
  writeFileSync(join(dir, file.replace('.light.svg', '.dark.svg')), recolor(light))
  console.log(`illustration ${file.replace('.light.svg', '')}: light + dark`)
}
