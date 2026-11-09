// Builds the reference pages from the repository sources before `nuxt dev` / `nuxt generate`.
// Output (gitignored, rewritten on every run):
//   content/3.api/2.reference.md    JS functions and types (guest-js/)
//   content/3.api/4.commands.md     Tauri commands (src/commands.rs)
//   content/3.api/8.permissions.md  permissions (permissions/)
//   content/6.changelog.md          CHANGELOG.md
// Edit the sources, never the generated pages.
// Element pages, the showcase and the capability matrix are filled by `pnpm docs:generate` at the repo root.
import { readdirSync, readFileSync, writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import ts from 'typescript'
import { parse as parseToml } from 'smol-toml'

const root = fileURLToPath(new URL('../../', import.meta.url))
const content = fileURLToPath(new URL('../content/', import.meta.url))
const repo = 'https://github.com/s00d/tauri-plugin-widgets/blob/main/'

const read = (rel) => readFileSync(root + rel, 'utf8')
const page = (title, description, source, body) =>
  `---\ntitle: ${JSON.stringify(title)}\ndescription: ${JSON.stringify(description)}\ngenerated: true\n---\n\n` +
  `<!-- Generated from ${source} by docs/scripts/generate.mjs. Edit the source, not this file. -->\n\n${body.trim()}\n`
// Inline code inside a table cell: one line, `|` escaped for GFM.
const cell = (text) => (text ? '`' + text.replace(/\s+/g, ' ').replace(/\|/g, '\\|') + '`' : '')
const oneLine = (text) => (text || '').replace(/\s*\n\s*/g, ' ').replace(/\|/g, '\\|').trim()
const firstPara = (text) => (text || '').split('\n\n')[0]
const camel = (s) => s.replace(/_([a-z])/g, (_, c) => c.toUpperCase())
function check(cond, message) {
  if (!cond) throw new Error(`generate.mjs: ${message}`)
}

// ---------------------------------------------------------------- JS API (TypeScript compiler API)

const sourceFile = (rel) => ts.createSourceFile(rel, read(rel), ts.ScriptTarget.Latest, true)

// `{@link setWidgetConfig}` -> `setWidgetConfig`
const linkText = (text) => text.replace(/\{@link\s+([^}\s]+)\s*\}/g, '`$1`')

function jsdoc(node) {
  const doc = ts.getJSDocCommentsAndTags(node).filter(ts.isJSDoc).at(-1)
  if (!doc) return { text: '', params: [], tags: {} }
  const tags = {}
  const params = []
  for (const tag of doc.tags ?? []) {
    const comment = linkText(ts.getTextOfJSDocComment(tag.comment) ?? '').replace(/^-\s*/, '')
    if (ts.isJSDocParameterTag(tag)) params.push({ name: tag.name.getText(), text: comment })
    else tags[tag.tagName.text] = comment
  }
  return { text: linkText(ts.getTextOfJSDocComment(doc.comment) ?? ''), params, tags }
}

function docBlock({ text, params, tags }) {
  const out = []
  if ('deprecated' in tags) out.push(`**Deprecated.** ${tags.deprecated}`.trim())
  if (text) out.push(text)
  if (params.length) out.push(params.map((p) => `- \`${p.name}\`: ${oneLine(p.text)}`).join('\n'))
  if (tags.returns) out.push(`**Returns:** ${tags.returns}`)
  if (tags.example) out.push(`**Example**\n\n${tags.example.trim()}`)
  return out.join('\n\n')
}

// Declaration text without its body, whitespace collapsed.
function signature(node, sf) {
  const end = node.body ? node.body.getStart(sf) : node.end
  return sf.text
    .slice(node.getStart(sf), end)
    .replace(/^export\s+/, '')
    .replace(/\s+/g, ' ')
    .replace(/\(\s/g, '(')
    .replace(/,?\s\)/g, ')')
    .replace(/;$/, '')
    .trim()
}

// Public surface = what guest-js/index.ts re-exports from ./api/*, in that order.
const modules = [] // { file, names: Set }
for (const st of sourceFile('guest-js/index.ts').statements) {
  if (!ts.isExportDeclaration(st) || !st.exportClause || !ts.isNamedExports(st.exportClause)) continue
  const spec = st.moduleSpecifier.text
  if (!spec.startsWith('./api/')) continue
  modules.push({ file: `guest-js/${spec.slice(2)}.ts`, names: new Set(st.exportClause.elements.map((e) => e.name.text)) })
}
check(modules.length > 0, 'guest-js/index.ts re-exports nothing from ./api/*')

const jsFunctions = new Map() // name -> first paragraph, for the commands page
function referencePage() {
  const fnSections = []
  const typeSections = []
  const summary = []
  const seen = new Set()
  for (const { file, names } of modules) {
    const sf = sourceFile(file)
    for (const st of sf.statements) {
      const name = ts.isVariableStatement(st) ? st.declarationList.declarations[0].name.getText() : st.name?.text
      if (!name || !names.has(name)) continue
      seen.add(name)
      const doc = jsdoc(st)
      const src = `[\`${file}\`](${repo}${file})`
      if (ts.isFunctionDeclaration(st)) {
        jsFunctions.set(name, firstPara(doc.text))
        summary.push(`| [${cell(name + '()')}](#${name.toLowerCase()}) | ${oneLine(firstPara(doc.text))} |`)
        fnSections.push(`### \`${name}()\`\n\n\`\`\`ts\n${signature(st, sf)}\n\`\`\`\n\n${docBlock(doc)}\n\nSource: ${src}\n`)
      } else if (ts.isInterfaceDeclaration(st)) {
        const rows = st.members.map((m) => {
          const opt = m.questionToken ? '?' : ''
          // Index signature `[key: string]: unknown`: no name, show it whole.
          if (!m.name) return `| ${cell(m.getText(sf).replace(/;$/, ''))} | | ${oneLine(docBlock(jsdoc(m)))} |`
          return `| ${cell(m.name.getText() + opt)} | ${cell(m.type?.getText(sf) ?? 'unknown')} | ${oneLine(docBlock(jsdoc(m)))} |`
        })
        typeSections.push(`### \`${name}\`\n\n${docBlock(doc)}\n\n| Property | Type | Description |\n|----------|------|-------------|\n${rows.join('\n')}\n`)
      } else {
        // Type aliases and constants: the declaration itself.
        typeSections.push(`### \`${name}\`\n\n${docBlock(doc)}\n\n\`\`\`ts\n${st.getText(sf).replace(/^export\s+/, '')}\n\`\`\`\n`)
      }
    }
    const missing = [...names].filter((n) => !seen.has(n))
    check(missing.length === 0, `${file}: exported but not found: ${missing.join(', ')}`)
  }
  check(fnSections.length > 0, 'no exported functions found in guest-js/api')

  const body = [
    'Every function and type `tauri-plugin-widgets-api` exports, generated from the JSDoc in',
    `[\`guest-js/api/\`](${repo}guest-js/api). For how they fit together see the [JavaScript API guide](/api/js).`,
    'Widget IR types (`WidgetConfig`, `WidgetElement`, …) are generated from Rust into',
    `[\`guest-js/generated/widget-types.ts\`](${repo}guest-js/generated/widget-types.ts); they are documented per element on the [Elements](/elements) pages and in the [JSON Schema](/api/json-schema).`,
    '',
    '| Function | Description |',
    '|----------|-------------|',
    summary.join('\n'),
    '',
    '## Functions',
    '',
    fnSections.join('\n'),
    '## Types',
    '',
    typeSections.join('\n'),
  ].join('\n')
  return page('JS reference', 'Every exported function and type of tauri-plugin-widgets-api, generated from guest-js/.', 'guest-js/api/', body)
}

// ---------------------------------------------------------------- Permissions (permissions/**)

const permissions = [] // { identifier, description }
for (const file of readdirSync(root + 'permissions/autogenerated/commands').filter((f) => f.endsWith('.toml')).sort()) {
  for (const p of parseToml(read(`permissions/autogenerated/commands/${file}`)).permission ?? []) {
    permissions.push({ identifier: p.identifier, description: p.description ?? '' })
  }
}
const defaults = parseToml(read('permissions/default.toml')).default ?? { description: '', permissions: [] }
check(permissions.length > 0, 'no permissions found under permissions/autogenerated/commands')
const permissionIds = new Set(permissions.map((p) => p.identifier))

function permissionsPage() {
  const inDefault = new Set(defaults.permissions)
  const rows = permissions
    .map((p) => `| ${cell(`widgets:${p.identifier}`)} | ${oneLine(p.description)} | ${inDefault.has(p.identifier) ? '✓' : ''} |`)
    .join('\n')
  const body = [
    `Tauri capability permissions for this plugin, generated from [\`permissions/default.toml\`](${repo}permissions/default.toml)`,
    `and [\`permissions/autogenerated/\`](${repo}permissions/autogenerated), which Tauri writes from the command list.`,
    'Add them to a capability file, for example `"permissions": ["core:default", "widgets:default"]`',
    '(see [Install](/guide/install)). Each command is listed on the [Tauri commands](/api/commands) page.',
    '',
    '## Default permission set',
    '',
    'Identifier: `widgets:default`',
    '',
    defaults.description.trim(),
    '',
    `It includes ${defaults.permissions.length} permissions:`,
    '',
    defaults.permissions.map((p) => `\`${p}\``).join(', '),
    '',
    '## All permissions',
    '',
    '| Identifier | Description | In default |',
    '|------------|-------------|------------|',
    rows,
  ].join('\n')
  return page('Permissions', 'Every permission identifier of the plugin, generated from permissions/.', 'permissions/', body)
}

// ---------------------------------------------------------------- Tauri commands (src/commands.rs)

// Splits `a: T<X, Y>, b: U` on top-level commas.
function splitTopLevel(text) {
  const parts = []
  let depth = 0
  let cur = ''
  for (const ch of text) {
    if ('(<['.includes(ch)) depth++
    if (')>]'.includes(ch)) depth--
    if (ch === ',' && depth === 0) {
      parts.push(cur)
      cur = ''
    } else cur += ch
  }
  if (cur.trim()) parts.push(cur)
  return parts.map((p) => p.trim()).filter(Boolean)
}

function parseCommands(src) {
  const lines = src.split('\n')
  const commands = []
  let doc = []
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim()
    if (line.startsWith('///')) {
      doc.push(line.replace(/^\/\/\/ ?/, ''))
      continue
    }
    if (line !== '#[tauri::command]') {
      if (!line.startsWith('#[')) doc = []
      continue
    }
    let j = i + 1
    while (lines[j].trim().startsWith('#[')) j++
    let sig = ''
    while (!sig.includes('{')) sig += lines[j++] + '\n'
    sig = sig.slice(0, sig.indexOf('{'))
    const m = sig.match(/pub\s+(async\s+)?fn\s+(\w+)\s*(<[^(]*>)?\s*\(/)
    check(m, `cannot parse command signature near line ${i + 1} of src/commands.rs`)
    const open = sig.indexOf('(', m.index + m[0].length - 1)
    let depth = 0
    let close = open
    for (; close < sig.length; close++) {
      if (sig[close] === '(') depth++
      if (sig[close] === ')' && --depth === 0) break
    }
    const params = splitTopLevel(sig.slice(open + 1, close))
      .map((p) => {
        const [name, ...type] = p.split(':')
        return { name: name.trim(), type: type.join(':').trim() }
      })
      // Injected by Tauri, not passed from JS.
      .filter((p) => !/^(AppHandle|State|Window|WebviewWindow|Webview)\b/.test(p.type))
    const ret = sig.slice(close + 1).replace(/^\s*->\s*/, '').trim()
    const ok = ret.match(/^Result<([\s\S]*),\s*Error>$/)?.[1].trim() ?? ret
    commands.push({ name: m[2], async: Boolean(m[1]), params, returns: ok.replace(/\s+/g, ' '), doc: doc.join('\n').trim() })
    doc = []
    i = j - 1
  }
  return commands
}

function commandsPage() {
  const commands = parseCommands(read('src/commands.rs'))
  check(commands.length > 0, 'no #[tauri::command] functions found in src/commands.rs')
  const noPermission = commands.filter((c) => !permissionIds.has(`allow-${c.name.replace(/_/g, '-')}`)).map((c) => c.name)
  if (noPermission.length) console.warn(`generate.mjs: commands without an allow-* permission: ${noPermission.join(', ')}`)

  // Rust doc comment, else the JS wrapper's description (same name in camelCase).
  const describe = (c) => linkText(c.doc.replace(/\[`([^`\]]+)`\]/g, '`$1`')) || jsFunctions.get(camel(c.name)) || ''
  const jsLink = (c) => (jsFunctions.has(camel(c.name)) ? ` JS wrapper: [\`${camel(c.name)}()\`](/api/reference#${camel(c.name).toLowerCase()}).` : '')

  const summary = commands.map((c) => `| [${cell(c.name)}](#${c.name}) | ${oneLine(firstPara(describe(c)))} |`).join('\n')
  const details = commands
    .map((c) => {
      const permission = `allow-${c.name.replace(/_/g, '-')}`
      const params = c.params.length
        ? '| Argument (JS) | Rust parameter | Type |\n|---------------|----------------|------|\n' +
          c.params.map((p) => `| ${cell(camel(p.name))} | ${cell(p.name)} | ${cell(p.type)} |`).join('\n')
        : 'No arguments.'
      return [
        `### \`${c.name}\``,
        '',
        describe(c),
        '',
        '```ts',
        `invoke('plugin:widgets|${c.name}'${c.params.length ? `, { ${c.params.map((p) => camel(p.name)).join(', ')} }` : ''})`,
        '```',
        '',
        params,
        '',
        `Returns ${cell(c.returns)}. ${c.async ? 'Rust: `async fn`.' : 'Rust: plain (sync) `fn`.'}` +
          (permissionIds.has(permission) ? ` Permission: ${cell(`widgets:${permission}`)}.` : '') +
          jsLink(c),
        '',
      ].join('\n')
    })
    .join('\n')

  const body = [
    `The \`#[tauri::command]\` functions in [\`src/commands.rs\`](${repo}src/commands.rs), which the JS package calls`,
    "with `invoke('plugin:widgets|<name>', args)`. You normally use the [JS functions](/api/reference) instead;",
    'this list is for calling commands directly. Arguments are sent in camelCase; `AppHandle` and `State`',
    'parameters are injected by Tauri and not listed. Rust-side API: [Rust](/api/rust).',
    '',
    '| Command | Description |',
    '|---------|-------------|',
    summary,
    '',
    '## Commands',
    '',
    details,
  ].join('\n')
  return page('Tauri commands', 'Every Tauri command of the plugin with its arguments, generated from src/commands.rs.', 'src/commands.rs', body)
}

// ---------------------------------------------------------------- Changelog (CHANGELOG.md)

function changelogPage() {
  let body = read('CHANGELOG.md').replace(/^# .*\n/, '')
  // Relative links would 404 on the site.
  body = body.replace(/\]\((?!https?:|#|\/|mailto:)([^)]+)\)/g, (_, target) => `](${repo}${target})`)
  return page('Changelog', 'All notable changes to tauri-plugin-widgets and tauri-plugin-widgets-api.', 'CHANGELOG.md', body)
}

// ---------------------------------------------------------------- write

// referencePage() first: commandsPage() reuses the JS descriptions.
const out = {
  [`${content}3.api/2.reference.md`]: referencePage(),
  [`${content}3.api/4.commands.md`]: commandsPage(),
  [`${content}3.api/8.permissions.md`]: permissionsPage(),
  [`${content}6.changelog.md`]: changelogPage(),
}
for (const [file, text] of Object.entries(out)) {
  writeFileSync(file, text)
  console.log(`generated ${file.slice(root.length)}`)
}
