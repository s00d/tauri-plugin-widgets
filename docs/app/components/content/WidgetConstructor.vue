<script setup lang="ts">
/**
 * Widget constructor: assemble a WidgetConfig from blocks with the real desktop renderer as
 * the preview. Blocks (left) go into the selected container; layers can be dragged; a click
 * on the preview selects that element; the inspector shows the main properties as controls
 * and every other schema property under "All properties". Saved in localStorage.
 */
import { withBase } from 'ufo'
import { renderWidget, type RenderHandle } from 'tauri-plugin-widgets-api/render'
import Field from './constructor/Field.vue'
import { LAYOUT_FIELDS, STYLE_FIELDS, getPath, metaOf, setPath, type FieldSpec } from './constructor/meta'
import {
  ELEMENT_GROUPS,
  type NodePath,
  type SizeKey,
  type WidgetConfigState,
  type WidgetNode,
  createDefaultConfig,
  createNode,
  duplicateAtPath,
  getAtPath,
  getChildren,
  insertAtPath,
  isContainer,
  moveSibling,
  removeAtPath,
} from './constructor/model'
import { type InspectorField, type JsonSchema, fieldsForType } from './constructor/schema'

const base = useRuntimeConfig().app.baseURL
const toast = useToast()
const STORE = 'tpw-constructor-v2'
const SIZES: { key: SizeKey; label: string; w: number; h: number }[] = [
  { key: 'small', label: 'Small', w: 170, h: 170 },
  { key: 'medium', label: 'Medium', w: 360, h: 170 },
  { key: 'large', label: 'Large', w: 360, h: 380 },
]
const TEMPLATES = [
  { id: 'weather.small', label: 'Weather', icon: 'i-lucide-sun' },
  { id: 'tasks.small', label: 'Tasks', icon: 'i-lucide-list-checks' },
  { id: 'music.medium', label: 'Music player', icon: 'i-lucide-music' },
  { id: 'fitness.medium', label: 'Fitness rings', icon: 'i-lucide-activity' },
  { id: 'upcoming-payments.medium', label: 'Payments', icon: 'i-lucide-credit-card' },
  { id: 'calculator.small', label: 'Calculator', icon: 'i-lucide-calculator' },
  { id: 'nested-dashboard.large', label: 'Dashboard', icon: 'i-lucide-layout-dashboard' },
]

// ── State ──────────────────────────────────────────────────────────────────────────────
const config = ref<WidgetConfigState>(createDefaultConfig())
const size = ref<SizeKey>('small')
const theme = ref<'dark' | 'light'>('dark')
const zoom = ref(1.5)
/** The zoom actually used: the chosen one, reduced so the widget fits the stage. */
const stage = ref<HTMLElement | null>(null)
const stageW = ref(0)
const fitZoom = computed(() => {
  if (!stageW.value) return zoom.value
  return Math.max(0.5, Math.min(zoom.value, (stageW.value - 48) / frame.value.w))
})
const view = ref<'preview' | 'json'>('preview')
const selected = ref<NodePath>([])
const schema = ref<JsonSchema | null>(null)
const jsonText = ref('')
const jsonError = ref('')
const showAll = ref(false)

const root = computed(() => (config.value[size.value] as WidgetNode | null | undefined) ?? null)
const node = computed(() => (root.value ? getAtPath(root.value, selected.value) : null))
const frame = computed(() => SIZES.find((s) => s.key === size.value)!)

// ── History (undo / redo) ──────────────────────────────────────────────────────────────
const past: string[] = []
const future: string[] = []
const canUndo = ref(false)
const canRedo = ref(false)
const snap = () => JSON.stringify(config.value)
/** Call before a change: remembers the current config for undo. */
function record() {
  past.push(snap())
  if (past.length > 100) past.shift()
  future.length = 0
  canUndo.value = true
  canRedo.value = false
}
function changed() {
  config.value = { ...config.value }
}
function undo() {
  const prev = past.pop()
  if (!prev) return
  future.push(snap())
  config.value = JSON.parse(prev)
  canUndo.value = past.length > 0
  canRedo.value = true
  clampSelection()
}
function redo() {
  const next = future.pop()
  if (!next) return
  past.push(snap())
  config.value = JSON.parse(next)
  canRedo.value = future.length > 0
  canUndo.value = true
  clampSelection()
}
function clampSelection() {
  while (selected.value.length && !(root.value && getAtPath(root.value, selected.value))) selected.value = selected.value.slice(0, -1)
}

// ── Tree edits ─────────────────────────────────────────────────────────────────────────
/** Where a new block goes: into the selected container, else after the selected element. */
const target = computed(() => {
  const r = root.value
  if (!r) return { parent: [] as NodePath, index: 0, label: 'Becomes the root' }
  const n = node.value
  if (n && isContainer(n)) return { parent: selected.value, index: getChildren(n).length, label: `Adds into ${metaOf(n.type).label}` }
  if (selected.value.length === 0) return { parent: [] as NodePath, index: 0, label: 'Select a container' }
  const parent = selected.value.slice(0, -1)
  return { parent, index: selected.value[selected.value.length - 1]! + 1, label: `Adds after ${metaOf(n!.type).label}` }
})

function add(type: string, parent = target.value.parent, index = target.value.index) {
  record()
  const n = createNode(type)
  if (!root.value) {
    config.value[size.value] = isContainer(n) ? n : { ...createNode('vstack'), children: [n] }
    selected.value = isContainer(n) ? [] : [0]
  } else if (insertAtPath(root.value, parent, index, n)) {
    selected.value = [...parent, index]
  } else {
    past.pop()
    return
  }
  changed()
}
function remove(path = selected.value) {
  if (!root.value || path.length === 0) return
  record()
  removeAtPath(root.value, path)
  selected.value = path.slice(0, -1)
  changed()
}
function duplicate(path = selected.value) {
  if (!root.value) return
  record()
  const p = duplicateAtPath(root.value, path)
  if (p) selected.value = p
  changed()
}
function move(dir: -1 | 1, path = selected.value) {
  if (!root.value) return
  record()
  const p = moveSibling(root.value, path, dir)
  if (p) selected.value = p
  else past.pop()
  changed()
}
/** Moves `from` before `to`, or into `to` when it is a container and `into` is set. */
function relocate(from: NodePath, to: NodePath, into: boolean) {
  const r = root.value
  if (!r || from.length === 0) return
  if (to.length >= from.length && from.every((v, i) => to[i] === v)) return // into itself
  record()
  const moving = removeAtPath(r, from)
  if (!moving) return
  // Removing shifts later siblings of the same parent.
  const adj = [...to]
  const d = from.length - 1
  if (adj.length > d && from.slice(0, d).every((v, i) => adj[i] === v) && adj[d]! > from[d]!) adj[d]!--
  const t = getAtPath(r, adj)
  if (into && t && isContainer(t)) {
    getChildren(t).push(moving)
    selected.value = [...adj, getChildren(t).length - 1]
  } else {
    const parent = adj.slice(0, -1)
    insertAtPath(r, parent, adj[adj.length - 1] ?? 0, moving)
    selected.value = adj
  }
  changed()
}

function setProp(spec: string, value: unknown) {
  if (!node.value) return
  record()
  setPath(node.value, spec, value)
  changed()
}

// ── Layers (flat list of the tree) ─────────────────────────────────────────────────────
type Row = { path: NodePath; depth: number; node: WidgetNode; key: string }
const rows = computed<Row[]>(() => {
  const out: Row[] = []
  const walk = (n: WidgetNode, path: NodePath) => {
    out.push({ path, depth: path.length, node: n, key: `n:${path.join('.')}` })
    if (isContainer(n)) getChildren(n).forEach((c, i) => walk(c, [...path, i]))
  }
  if (root.value) walk(root.value, [])
  return out
})
const isSel = (p: NodePath) => p.length === selected.value.length && p.every((v, i) => v === selected.value[i])
const rowTitle = (n: WidgetNode) => {
  const t = (n.content ?? n.label ?? n.text ?? n.systemName ?? '') as string
  return typeof t === 'string' && t ? t.slice(0, 28) : ''
}

// Drag and drop: palette blocks and layers onto layers or onto the preview.
type Drag = { kind: 'new'; type: string } | { kind: 'move'; path: NodePath }
type Place = 'before' | 'into' | 'after'
const drag = ref<Drag | null>(null)
/** Drop indicator: the row (or preview node) key and where it would land. */
const dropAt = ref<{ key: string; place: Place } | null>(null)

function startDrag(d: Drag, e: DragEvent) {
  drag.value = d
  // Without data Safari / Firefox never start the drag.
  e.dataTransfer?.setData('text/plain', d.kind === 'new' ? d.type : d.path.join('.'))
  if (e.dataTransfer) e.dataTransfer.effectAllowed = d.kind === 'new' ? 'copy' : 'move'
}
function endDrag() {
  drag.value = null
  dropAt.value = null
}
/** Top quarter: before; bottom quarter: after; the middle of a container: into it. */
function placeOf(el: Element, e: DragEvent, container: boolean, isRoot: boolean): Place {
  if (isRoot) return 'into'
  const r = el.getBoundingClientRect()
  const k = (e.clientY - r.top) / Math.max(1, r.height)
  if (container) return k < 0.25 ? 'before' : k > 0.75 ? 'after' : 'into'
  return k < 0.5 ? 'before' : 'after'
}
/** Applies the drop: a new block or a moved layer, before / into / after `path`. */
function dropOn(path: NodePath, place: Place) {
  const d = drag.value
  endDrag()
  if (!d || !root.value) {
    if (d?.kind === 'new') add(d.type)
    return
  }
  const t = getAtPath(root.value, path)
  if (!t) return
  const parent = place === 'into' ? path : path.slice(0, -1)
  const index = place === 'into' ? getChildren(t).length : path[path.length - 1]! + (place === 'after' ? 1 : 0)
  if (d.kind === 'new') add(d.type, parent, index)
  else relocate(d.path, place === 'into' ? path : [...parent, index], place === 'into')
}
function rowOver(row: Row, e: DragEvent) {
  dropAt.value = { key: row.key, place: placeOf(e.currentTarget as Element, e, isContainer(row.node), row.path.length === 0) }
}
function rowDrop(row: Row, e: DragEvent) {
  dropOn(row.path, placeOf(e.currentTarget as Element, e, isContainer(row.node), row.path.length === 0))
}
/** The preview: the drawn element under the pointer is the target. */
function nodeAt(e: DragEvent): { path: NodePath; el: Element } | null {
  const el = (e.target as Element | null)?.closest?.('[data-node]')
  if (!el) return null
  const raw = (el as HTMLElement).dataset.node ?? ''
  return { path: raw === '' ? [] : raw.split('.').map(Number), el }
}
function previewOver(e: DragEvent) {
  const hit = nodeAt(e)
  if (!hit || !root.value) return (dropAt.value = { key: 'preview', place: 'into' })
  const n = getAtPath(root.value, hit.path)
  const place = placeOf(hit.el, e, !!n && isContainer(n), hit.path.length === 0)
  dropAt.value = { key: `p:${hit.path.join('.')}`, place }
  host.value?.querySelectorAll('.cx-drop').forEach((x) => x.classList.remove('cx-drop'))
  hit.el.classList.add('cx-drop')
}
function previewDrop(e: DragEvent) {
  host.value?.querySelectorAll('.cx-drop').forEach((x) => x.classList.remove('cx-drop'))
  const hit = nodeAt(e)
  if (!hit || !root.value) return dropOn([], 'into')
  const n = getAtPath(root.value, hit.path)
  dropOn(hit.path, placeOf(hit.el, e, !!n && isContainer(n), hit.path.length === 0))
}
const dropClass = (r: Row) => (dropAt.value?.key === r.key ? `drop-${dropAt.value.place}` : '')

// ── Inspector ──────────────────────────────────────────────────────────────────────────
const schemaFields = computed<InspectorField[]>(() => (schema.value && node.value ? fieldsForType(schema.value, node.value.type) : []))
const enumOf = (key: string) => schemaFields.value.find((f) => f.key === key)?.enumValues
const shownKeys = computed(() => new Set([...metaOf(node.value?.type ?? '').fields, ...LAYOUT_FIELDS, ...STYLE_FIELDS].map((f) => f.key.split('.')[0])))
/** Schema fields not already shown above, as generic controls. */
const otherFields = computed(() =>
  schemaFields.value
    .filter((f) => !shownKeys.value.has(f.key) && f.key !== 'bind' && f.key !== 'when')
    .map((f): FieldSpec & { json: boolean } => ({
      key: f.key,
      label: f.key,
      kind: f.kind === 'enum' ? 'enum' : f.kind === 'boolean' ? 'bool' : f.kind === 'number' ? 'number' : 'text',
      min: -1000,
      max: 1000,
      json: f.kind === 'json',
    })),
)
const specFor = (s: FieldSpec) => ({ ...s, kind: s.kind })
function jsonProp(key: string) {
  const v = node.value?.[key]
  return v === undefined ? '' : JSON.stringify(v)
}
function setJsonProp(key: string, raw: string) {
  if (!raw.trim()) return setProp(key, undefined)
  try {
    setProp(key, JSON.parse(raw))
  } catch {
    toast.add({ title: `${key}: not valid JSON`, color: 'error' })
  }
}

// ── Preview ────────────────────────────────────────────────────────────────────────────
const host = ref<HTMLElement | null>(null)
let handle: RenderHandle | null = null
/** A copy of the tree with `__node` markers so a click maps back to a node. */
function marked(n: WidgetNode, path: NodePath): WidgetNode {
  const c: WidgetNode = { ...n, __node: path.join('.') }
  if (isContainer(n)) {
    const key = Array.isArray(n.items) && !Array.isArray(n.children) ? 'items' : 'children'
    c[key] = getChildren({ ...n }).map((k, i) => marked(k, [...path, i]))
  }
  return c
}
function render() {
  if (!host.value) return
  handle?.destroy()
  const r = root.value
  const cfg = r ? { ...stripMarkers(config.value), [size.value]: marked(r, []) } : null
  handle = renderWidget(host.value, cfg as never, { size: size.value, theme: theme.value })
  highlight()
}
function highlight() {
  nextTick(() => {
    host.value?.querySelectorAll('.cx-sel').forEach((e) => e.classList.remove('cx-sel'))
    host.value?.querySelector(`[data-node="${selected.value.join('.')}"]`)?.classList.add('cx-sel')
  })
}
function onPreviewClick(e: MouseEvent) {
  const el = (e.target as HTMLElement).closest('[data-node]') as HTMLElement | null
  if (!el) return
  const p = el.dataset.node ?? ''
  selected.value = p === '' ? [] : p.split('.').map(Number)
}

// ── JSON, import / export, templates ───────────────────────────────────────────────────
/** Removes `null`s (the gallery fixtures carry every optional key) and editor markers. */
function clean<T>(v: T): T {
  if (Array.isArray(v)) return v.map(clean) as T
  if (v && typeof v === 'object') {
    return Object.fromEntries(
      Object.entries(v as Record<string, unknown>)
        .filter(([k, x]) => x !== null && k !== '__node' && k !== '$schema')
        .map(([k, x]) => [k, clean(x)]),
    ) as T
  }
  return v
}
const stripMarkers = (c: WidgetConfigState) => clean(c)
const exported = computed(() => JSON.stringify({ version: 1, ...clean(config.value) }, null, 2))

function applyJson() {
  try {
    const parsed = JSON.parse(jsonText.value)
    record()
    config.value = clean(parsed)
    jsonError.value = ''
    clampSelection()
  } catch (e) {
    jsonError.value = String((e as Error).message)
  }
}
async function loadTemplate(id: string) {
  const res = await fetch(withBase(`/gallery-data/${id}.json`, base))
  if (!res.ok) return toast.add({ title: `Template ${id} not found`, color: 'error' })
  record()
  config.value = clean(await res.json())
  size.value = (id.split('.').pop() as SizeKey) ?? 'small'
  selected.value = []
}
function reset() {
  record()
  config.value = createDefaultConfig()
  selected.value = []
}
async function copy(text: string, what: string) {
  await navigator.clipboard.writeText(text)
  toast.add({ title: `${what} copied`, icon: 'i-lucide-check' })
}
function download() {
  const a = document.createElement('a')
  a.href = URL.createObjectURL(new Blob([exported.value], { type: 'application/json' }))
  a.download = 'widget.json'
  a.click()
  URL.revokeObjectURL(a.href)
}
const tsSnippet = computed(
  () => `import { setWidgetConfig } from "tauri-plugin-widgets-api";\n\nawait setWidgetConfig(${exported.value.replace(/\n/g, '\n')}, "group.com.example.app", "my-widget");\n`,
)
const fileInput = ref<HTMLInputElement | null>(null)
async function importFile(e: Event) {
  const f = (e.target as HTMLInputElement).files?.[0]
  if (!f) return
  jsonText.value = await f.text()
  applyJson()
  ;(e.target as HTMLInputElement).value = ''
}
const exportItems = computed(() => [
  [
    { label: 'Copy JSON', icon: 'i-lucide-copy', onSelect: () => copy(exported.value, 'JSON') },
    { label: 'Copy as TypeScript', icon: 'i-lucide-file-code', onSelect: () => copy(tsSnippet.value, 'TypeScript') },
    { label: 'Download widget.json', icon: 'i-lucide-download', onSelect: download },
  ],
  [{ label: 'Import JSON file…', icon: 'i-lucide-upload', onSelect: () => fileInput.value?.click() }],
])
const templateItems = computed(() => [
  TEMPLATES.map((t) => ({ label: t.label, icon: t.icon, onSelect: () => loadTemplate(t.id) })),
  [{ label: 'Blank', icon: 'i-lucide-file', onSelect: reset }],
])

// ── Lifecycle ──────────────────────────────────────────────────────────────────────────
watch([config, size, theme], () => {
  render()
  jsonText.value = exported.value
  try {
    localStorage.setItem(STORE, JSON.stringify({ config: config.value, size: size.value, theme: theme.value }))
  } catch {
    /* private mode */
  }
}, { deep: true })
watch(selected, highlight, { deep: true })

function onKey(e: KeyboardEvent) {
  const typing = (e.target as HTMLElement)?.closest('input, textarea, select, [contenteditable]')
  const mod = e.metaKey || e.ctrlKey
  if (mod && e.key.toLowerCase() === 'z') {
    e.preventDefault()
    if (e.shiftKey) redo()
    else undo()
  } else if (!typing && (e.key === 'Delete' || e.key === 'Backspace')) {
    e.preventDefault()
    remove()
  } else if (!typing && mod && e.key.toLowerCase() === 'd') {
    e.preventDefault()
    duplicate()
  }
}

onMounted(async () => {
  try {
    const saved = JSON.parse(localStorage.getItem(STORE) || 'null')
    if (saved?.config) {
      config.value = saved.config
      size.value = saved.size ?? 'small'
      theme.value = saved.theme ?? 'dark'
    }
  } catch {
    /* ignore */
  }
  try {
    const res = await fetch(withBase('/schemas/widget-config.v1.json', base))
    if (res.ok) schema.value = (await res.json()) as JsonSchema
  } catch {
    /* inspector still has the main fields */
  }
  jsonText.value = exported.value
  render()
  window.addEventListener('keydown', onKey)
  if (stage.value) {
    ro = new ResizeObserver(([e]) => (stageW.value = e!.contentRect.width))
    ro.observe(stage.value)
  }
})
let ro: ResizeObserver | null = null
onBeforeUnmount(() => {
  ro?.disconnect()
  handle?.destroy()
  window.removeEventListener('keydown', onKey)
})
</script>

<template>
  <div class="cx">
    <!-- Toolbar -->
    <div class="cx-bar">
      <UDropdownMenu :items="templateItems">
        <UButton icon="i-lucide-layout-template" label="Templates" color="neutral" variant="outline" size="sm" trailing-icon="i-lucide-chevron-down" />
      </UDropdownMenu>
      <UButton icon="i-lucide-undo-2" color="neutral" variant="ghost" size="sm" :disabled="!canUndo" aria-label="Undo (⌘Z)" @click="undo" />
      <UButton icon="i-lucide-redo-2" color="neutral" variant="ghost" size="sm" :disabled="!canRedo" aria-label="Redo (⇧⌘Z)" @click="redo" />
      <span class="cx-sep" />
      <div class="cx-seg" role="tablist" aria-label="Widget size">
        <UButton
          v-for="s in SIZES"
          :key="s.key"
          :label="s.label"
          size="sm"
          :color="size === s.key ? 'primary' : 'neutral'"
          :variant="size === s.key ? 'soft' : 'ghost'"
          @click="size = s.key; selected = []"
        />
      </div>
      <UButton
        :icon="theme === 'dark' ? 'i-lucide-moon' : 'i-lucide-sun'"
        color="neutral"
        variant="ghost"
        size="sm"
        :aria-label="`Theme: ${theme}`"
        @click="theme = theme === 'dark' ? 'light' : 'dark'"
      />
      <div class="cx-seg">
        <UButton v-for="z in [1, 1.5, 2]" :key="z" :label="`${z * 100}%`" size="sm" :color="zoom === z ? 'primary' : 'neutral'" :variant="zoom === z ? 'soft' : 'ghost'" @click="zoom = z" />
      </div>
      <span class="flex-1" />
      <div class="cx-seg">
        <UButton icon="i-lucide-eye" label="Preview" size="sm" :color="view === 'preview' ? 'primary' : 'neutral'" :variant="view === 'preview' ? 'soft' : 'ghost'" @click="view = 'preview'" />
        <UButton icon="i-lucide-braces" label="JSON" size="sm" :color="view === 'json' ? 'primary' : 'neutral'" :variant="view === 'json' ? 'soft' : 'ghost'" @click="view = 'json'" />
      </div>
      <UDropdownMenu :items="exportItems">
        <UButton icon="i-lucide-share" label="Export" size="sm" trailing-icon="i-lucide-chevron-down" />
      </UDropdownMenu>
      <input ref="fileInput" type="file" accept="application/json,.json" hidden @change="importFile" />
    </div>

    <div class="cx-body">
      <!-- Blocks + layers -->
      <aside class="cx-left">
        <p class="cx-h">Blocks <span class="cx-hint">{{ target.label }}</span></p>
        <div v-for="g in ELEMENT_GROUPS" :key="g.id" class="cx-group">
          <p class="cx-gl">{{ g.label }}</p>
          <div class="cx-blocks">
            <button
              v-for="t in g.types"
              :key="t"
              type="button"
              class="cx-block"
              draggable="true"
              :title="metaOf(t).description"
              @click="add(t)"
              @dragstart="startDrag({ kind: 'new', type: t }, $event)"
              @dragend="endDrag"
            >
              <UIcon :name="metaOf(t).icon" class="size-4 shrink-0" />
              <span>{{ metaOf(t).label }}</span>
            </button>
          </div>
        </div>

        <p class="cx-h mt-4">Layers <span class="cx-hint">drag to reorder</span></p>
        <div class="cx-layers" role="tree">
          <p v-if="!rows.length" class="cx-empty">Empty. Click a block above.</p>
          <div
            v-for="r in rows"
            :key="r.key"
            role="treeitem"
            :aria-selected="isSel(r.path)"
            class="cx-row"
            :class="[{ sel: isSel(r.path) }, dropClass(r)]"
            :style="{ paddingLeft: `${8 + r.depth * 14}px` }"
            :draggable="r.path.length > 0"
            @click="selected = [...r.path]"
            @dragstart="startDrag({ kind: 'move', path: r.path }, $event)"
            @dragend="endDrag"
            @dragover.prevent="rowOver(r, $event)"
            @drop.prevent="rowDrop(r, $event)"
          >
            <UIcon :name="metaOf(r.node.type).icon" class="size-3.5 shrink-0 opacity-70" />
            <span class="cx-row-t">{{ metaOf(r.node.type).label }}</span>
            <span class="cx-row-s">{{ rowTitle(r.node) }}</span>
          </div>
        </div>
      </aside>

      <!-- Preview / JSON -->
      <section class="cx-center">
        <div v-show="view === 'preview'" ref="stage" class="cx-stage" :class="theme">
          <div
            class="cx-frame"
            :class="{ 'drop-into': dropAt?.key === 'preview' }"
            :style="{ width: `${frame.w}px`, height: `${frame.h}px`, zoom: fitZoom }"
            @click="onPreviewClick"
            @dragover.prevent="previewOver"
            @dragleave="dropAt?.key.startsWith('p') && (dropAt = null)"
            @drop.prevent="previewDrop"
          >
            <div ref="host" class="cx-host" />
          </div>
          <p class="cx-caption">{{ frame.label }} · {{ frame.w }}×{{ frame.h }} · desktop renderer · click an element to select it</p>
          <UButton
            v-if="!root"
            class="mt-3"
            icon="i-lucide-copy-plus"
            :label="`Create ${frame.label.toLowerCase()} from small`"
            size="sm"
            :disabled="!config.small"
            @click="record(); config[size] = JSON.parse(JSON.stringify(config.small)); changed()"
          />
        </div>
        <div v-show="view === 'json'" class="cx-json">
          <UTextarea v-model="jsonText" :rows="22" class="w-full font-mono" spellcheck="false" />
          <div class="cx-json-bar">
            <span class="cx-err">{{ jsonError }}</span>
            <UButton label="Apply JSON" icon="i-lucide-check" size="sm" @click="applyJson" />
          </div>
        </div>
      </section>

      <!-- Inspector -->
      <aside class="cx-right">
        <template v-if="node">
          <div class="cx-ins-head">
            <UIcon :name="metaOf(node.type).icon" class="size-5" />
            <div class="min-w-0 flex-1">
              <p class="font-semibold">{{ metaOf(node.type).label }}</p>
              <p class="cx-hint">{{ metaOf(node.type).description }}</p>
            </div>
          </div>
          <div class="cx-actions">
            <UButton icon="i-lucide-arrow-up" color="neutral" variant="ghost" size="xs" :disabled="!selected.length" aria-label="Move up" @click="move(-1)" />
            <UButton icon="i-lucide-arrow-down" color="neutral" variant="ghost" size="xs" :disabled="!selected.length" aria-label="Move down" @click="move(1)" />
            <UButton icon="i-lucide-copy" color="neutral" variant="ghost" size="xs" :disabled="!selected.length" label="Duplicate" @click="duplicate()" />
            <UButton icon="i-lucide-trash-2" color="error" variant="ghost" size="xs" :disabled="!selected.length" label="Delete" @click="remove()" />
          </div>

          <p v-if="metaOf(node.type).fields.length" class="cx-sec">Content</p>
          <Field
            v-for="f in metaOf(node.type).fields"
            :key="`c-${f.key}`"
            :spec="specFor(f)"
            :value="getPath(node, f.key)"
            :options="enumOf(f.key)"
            @update="setProp(f.key, $event)"
          />
          <template v-if="node.type !== 'spacer'">
            <p class="cx-sec">Layout</p>
            <Field v-for="f in LAYOUT_FIELDS" :key="`l-${f.key}`" :spec="f" :value="getPath(node, f.key)" @update="setProp(f.key, $event)" />
            <p class="cx-sec">Style</p>
            <Field v-for="f in STYLE_FIELDS" :key="`s-${f.key}`" :spec="f" :value="getPath(node, f.key)" @update="setProp(f.key, $event)" />
          </template>

          <button type="button" class="cx-sec cx-toggle" @click="showAll = !showAll">
            <UIcon :name="showAll ? 'i-lucide-chevron-down' : 'i-lucide-chevron-right'" class="size-3.5" />
            All properties ({{ otherFields.length }})
          </button>
          <template v-if="showAll">
            <template v-for="f in otherFields" :key="`o-${f.key}`">
              <div v-if="f.json" class="cf-json">
                <label class="cx-hint">{{ f.key }}</label>
                <UTextarea :model-value="jsonProp(f.key)" :rows="1" autoresize size="xs" class="w-full font-mono" @change="setJsonProp(f.key, ($event.target as HTMLTextAreaElement).value)" />
              </div>
              <Field v-else :spec="f" :value="node[f.key]" :options="enumOf(f.key)" @update="setProp(f.key, $event)" />
            </template>
          </template>
        </template>
        <p v-else class="cx-empty">Select an element in the preview or in Layers.</p>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.cx { border: 1px solid var(--ui-border); border-radius: 12px; overflow: hidden; background: var(--ui-bg); }
.cx-bar { display: flex; flex-wrap: wrap; align-items: center; gap: 0.35rem; padding: 0.5rem 0.75rem; border-bottom: 1px solid var(--ui-border); background: var(--ui-bg-elevated); }
.cx-sep { width: 1px; height: 20px; background: var(--ui-border); margin: 0 0.25rem; }
.cx-seg { display: inline-flex; gap: 2px; padding: 2px; border-radius: 8px; background: var(--ui-bg-accented); }
.cx-body { display: grid; grid-template-columns: 260px minmax(0, 1fr) 340px; height: calc(100vh - var(--ui-header-height) - 5.5rem); min-height: 560px; }
.cx-left, .cx-right { padding: 0.75rem; overflow: auto; }
.cx-left { border-right: 1px solid var(--ui-border); }
.cx-right { border-left: 1px solid var(--ui-border); display: flex; flex-direction: column; gap: 0.5rem; }
.cx-h { font-size: 0.78rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em; color: var(--ui-text-muted); display: flex; justify-content: space-between; gap: 0.5rem; margin: 0 0 0.5rem; }
.cx-hint { font-size: 0.72rem; font-weight: 400; text-transform: none; letter-spacing: 0; color: var(--ui-text-dimmed); }
.cx-group { margin-bottom: 0.6rem; }
.cx-gl { font-size: 0.7rem; color: var(--ui-text-dimmed); margin: 0 0 0.25rem; }
.cx-blocks { display: grid; grid-template-columns: 1fr 1fr; gap: 0.3rem; }
.cx-block { display: flex; align-items: center; gap: 0.4rem; padding: 0.4rem 0.5rem; border: 1px solid var(--ui-border); border-radius: 8px; background: var(--ui-bg-elevated); font-size: 0.78rem; text-align: left; cursor: grab; transition: border-color .15s, background .15s; }
.cx-block:hover { border-color: var(--ui-primary); background: color-mix(in oklab, var(--ui-primary) 8%, var(--ui-bg-elevated)); }
.cx-block span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cx-layers { display: flex; flex-direction: column; gap: 1px; }
.cx-row { display: flex; align-items: center; gap: 0.4rem; padding: 0.3rem 0.5rem; border-radius: 6px; font-size: 0.8rem; cursor: pointer; border: 1px solid transparent; }
.cx-row:hover { background: var(--ui-bg-elevated); }
.cx-row.sel { background: color-mix(in oklab, var(--ui-primary) 16%, transparent); color: var(--ui-text-highlighted); }
.cx-row { position: relative; }
.cx-row.drop-into { border-color: var(--ui-primary); border-style: dashed; background: color-mix(in oklab, var(--ui-primary) 10%, transparent); }
.cx-row.drop-before::before, .cx-row.drop-after::after { content: ""; position: absolute; left: 6px; right: 6px; height: 2px; border-radius: 2px; background: var(--ui-primary); }
.cx-row.drop-before::before { top: -1px; }
.cx-row.drop-after::after { bottom: -1px; }
.cx-frame.drop-into { outline: 2px dashed var(--ui-primary); outline-offset: 4px; }
.cx-row-t { white-space: nowrap; }
.cx-row-s { color: var(--ui-text-dimmed); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 0.74rem; }
.cx-center { display: flex; flex-direction: column; min-width: 0; }
.cx-stage { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 2rem 1rem; overflow: auto;
  background-image: radial-gradient(var(--ui-border) 1px, transparent 1px); background-size: 16px 16px; }
.cx-stage.dark { background-color: #0b0d14; }
.cx-stage.light { background-color: #eef1f6; }
.cx-frame { position: relative; flex-shrink: 0; box-shadow: 0 12px 32px rgb(0 0 0 / 0.35); border-radius: 16px; }
.cx-host { width: 100%; height: 100%; }
.cx-caption { margin-top: 1rem; font-size: 0.75rem; color: var(--ui-text-dimmed); }
.cx-json { padding: 0.75rem; display: flex; flex-direction: column; gap: 0.5rem; }
.cx-json-bar { display: flex; justify-content: space-between; align-items: center; }
.cx-err { color: var(--ui-error); font-size: 0.78rem; }
.cx-ins-head { display: flex; align-items: center; gap: 0.6rem; }
.cx-actions { display: flex; flex-wrap: wrap; gap: 0.15rem; border-bottom: 1px solid var(--ui-border); padding-bottom: 0.5rem; }
.cx-sec { margin: 0.6rem 0 0.1rem; font-size: 0.72rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em; color: var(--ui-text-muted); }
.cx-toggle { display: flex; align-items: center; gap: 0.25rem; background: none; border: 0; padding: 0; cursor: pointer; text-align: left; }
.cx-empty { font-size: 0.8rem; color: var(--ui-text-dimmed); padding: 0.5rem 0; }
.cf-json { display: grid; grid-template-columns: 6.5rem 1fr; gap: 0.5rem; align-items: start; }
@media (max-width: 1100px) {
  .cx-body { grid-template-columns: 1fr; height: auto; }
  .cx-left, .cx-right { max-height: none; border: 0; border-top: 1px solid var(--ui-border); }
}
</style>

<style>
/* Preview selection: the renderer's DOM is outside the scoped styles. */
.cx-host [data-node] { cursor: pointer; }
.cx-host [data-node]:hover { outline: 1px dashed color-mix(in oklab, var(--ui-primary) 70%, transparent); outline-offset: 1px; }
.cx-host .cx-sel { outline: 2px solid var(--ui-primary) !important; outline-offset: 1px; }
.cx-host .cx-drop { outline: 2px dashed var(--ui-primary) !important; outline-offset: 2px; }
</style>
