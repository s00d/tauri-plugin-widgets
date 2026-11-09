// Renders docs/diagrams/*.mmd to docs/public/diagrams/<name>.{light,dark}.svg with mermaid-cli.
// The SVGs are committed: the site build and CI never run mermaid or a browser.
// Run after editing a diagram: `pnpm diagrams` (needs Chrome; set PUPPETEER_EXECUTABLE_PATH if not found).
import { execFileSync } from 'node:child_process'
import { mkdirSync, readdirSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath } from 'node:url'

const src = fileURLToPath(new URL('../diagrams/', import.meta.url))
const out = fileURLToPath(new URL('../public/diagrams/', import.meta.url))
mkdirSync(out, { recursive: true })

// SVG <text> labels instead of HTML in <foreignObject>: renders the same inside <img>.
const config = join(tmpdir(), 'widgets-mermaid.json')
writeFileSync(config, JSON.stringify({ htmlLabels: false, flowchart: { htmlLabels: false } }))

const themes = { light: 'neutral', dark: 'dark' }
for (const file of readdirSync(src).filter((f) => f.endsWith('.mmd')).sort()) {
  const name = file.replace(/\.mmd$/, '')
  for (const [variant, theme] of Object.entries(themes)) {
    const target = join(out, `${name}.${variant}.svg`)
    execFileSync('pnpm', ['dlx', '@mermaid-js/mermaid-cli@11', '-q', '-i', join(src, file), '-o', target, '-t', theme, '-b', 'transparent', '-c', config], { stdio: 'inherit' })
    console.log(`rendered ${target.slice(out.length - 'public/diagrams/'.length)}`)
  }
}
