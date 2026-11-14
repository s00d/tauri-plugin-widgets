// The VitePress site linked section indexes with a trailing slash (/guide/, /guide/setup/, …).
// Nuxt writes guide.html (autoSubfolderIndex: false), so GitHub Pages would 404 on /guide/:
// write guide/index.html that redirects to /guide.
import { existsSync, mkdirSync, writeFileSync } from 'node:fs'

const base = process.env.NUXT_APP_BASE_URL || '/tauri-plugin-widgets/'
const out = new URL('../.output/public/', import.meta.url)
for (const section of ['guide', 'guide/setup', 'elements', 'contributing']) {
  const target = `${base}${section}`
  const file = new URL(`${section}/index.html`, out)
  if (!existsSync(new URL(`${section}/`, out))) throw new Error(`redirects.mjs: no ${section}/ in .output/public`)
  writeFileSync(file, `<!doctype html><meta charset="utf-8"><title>Redirecting…</title><link rel="canonical" href="${target}"><meta http-equiv="refresh" content="0; url=${target}"><a href="${target}">${target}</a>\n`)
  console.log(`redirect ${section}/ -> ${target}`)
}

// Feature pages moved out of Guide > Recipes into the Features section: keep old links alive.
const moved = {
  'guide/recipes/actions': 'features/interactive',
  'guide/recipes/live-activities': 'features/live-activities',
  'guide/recipes/scheduled-and-data': 'features/timelines',
  'guide/recipes/updating-data': 'features/widget-updater',
  'guide/recipes/instances': 'features/instances',
  'guide/recipes/images': 'features/images',
  // The constructor became a full-width page of its own.
  'guide/constructor': 'constructor',
}
for (const [from, to] of Object.entries(moved)) {
  const target = `${base}${to}`
  const file = new URL(`${from}.html`, out)
  mkdirSync(new URL('./', file), { recursive: true })
  writeFileSync(file, `<!doctype html><meta charset="utf-8"><title>Redirecting…</title><link rel="canonical" href="${target}"><meta http-equiv="refresh" content="0; url=${target}"><a href="${target}">${target}</a>\n`)
  console.log(`redirect ${from} -> ${target}`)
}
