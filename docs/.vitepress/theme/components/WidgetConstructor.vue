<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { withBase } from "vitepress";
import { renderWidget, type RenderHandle } from "tauri-plugin-widgets-api/render";
import TreeNode from "./constructor/TreeNode.vue";
import {
  ALL_TYPES,
  ELEMENT_GROUPS,
  type NodePath,
  type SizeKey,
  type WidgetConfigState,
  type WidgetNode,
  createDefaultConfig,
  createNode,
  getAtPath,
  getChildren,
  insertAllTypes,
  insertAtPath,
  isContainer,
  moveSibling,
  duplicateAtPath,
  removeAtPath,
  softValidate,
} from "./constructor/model";
import {
  type InspectorField,
  type JsonSchema,
  allTypesFromSchema,
  fieldsForType,
} from "./constructor/schema";

const size = ref<SizeKey>("small");
const config = ref<WidgetConfigState>(createDefaultConfig());
const selected = ref<NodePath>([]);
const schema = ref<JsonSchema | null>(null);
const schemaTypes = ref<string[]>([]);
const warnings = ref<string[]>([]);
const status = ref("");
const hostEl = ref<HTMLElement | null>(null);
const fileInput = ref<HTMLInputElement | null>(null);
const treeEl = ref<HTMLElement | null>(null);
let handle: RenderHandle | null = null;

const frameSize = computed(() => {
  if (size.value === "large") return { width: "360px", height: "360px" };
  if (size.value === "medium") return { width: "360px", height: "200px" };
  return { width: "180px", height: "180px" };
});

const root = computed(() => (config.value[size.value] as WidgetNode | null | undefined) ?? null);

const selectedNode = computed(() => {
  if (!root.value) return null;
  return getAtPath(root.value, selected.value);
});

const inspectorFields = computed((): InspectorField[] => {
  if (!schema.value || !selectedNode.value) return [];
  return fieldsForType(schema.value, selectedNode.value.type);
});

const paletteGroups = computed(() => {
  const known = new Set(schemaTypes.value.length ? schemaTypes.value : ALL_TYPES);
  return ELEMENT_GROUPS.map((g) => ({
    ...g,
    types: g.types.filter((t) => known.has(t)),
  })).filter((g) => g.types.length);
});

const insertTargetLabel = computed(() => {
  const r = root.value;
  if (!r) return "Insert as root";
  const node = selectedNode.value;
  if (!node || selected.value.length === 0) {
    return isContainer(r) ? `Insert into ${r.type}` : "Insert as root";
  }
  if (isContainer(node)) return `Insert into ${node.type}`;
  return `Insert after ${node.type}`;
});

const canMoveUp = computed(() => {
  const r = root.value;
  if (!r || selected.value.length === 0) return false;
  return selected.value[selected.value.length - 1] > 0;
});

const canMoveDown = computed(() => {
  const r = root.value;
  if (!r || selected.value.length === 0) return false;
  const parent = getAtPath(r, selected.value.slice(0, -1));
  if (!parent) return false;
  const idx = selected.value[selected.value.length - 1];
  return idx < getChildren(parent).length - 1;
});

const canDuplicate = computed(() => {
  if (!selectedNode.value) return false;
  if (selected.value.length === 0) return isContainer(selectedNode.value);
  return true;
});

function bump() {
  config.value = { ...config.value };
  warnings.value = softValidate(config.value);
  pushPreview();
}

function ensureRoot(): WidgetNode {
  let r = config.value[size.value] as WidgetNode | null | undefined;
  if (!r) {
    r = createNode("vstack");
    config.value[size.value] = r;
  }
  return r;
}

function selectPath(path: NodePath) {
  selected.value = [...path];
  treeEl.value?.focus();
}

function addType(type: string) {
  const node = createNode(type);
  const r = ensureRoot();
  if (selected.value.length === 0) {
    if (isContainer(r)) {
      getChildren(r).push(node);
      selectPath([getChildren(r).length - 1]);
    } else {
      config.value[size.value] = node;
      selectPath([]);
    }
    bump();
    return;
  }
  const target = getAtPath(r, selected.value);
  if (target && isContainer(target)) {
    getChildren(target).push(node);
    selectPath([...selected.value, getChildren(target).length - 1]);
  } else {
    const parent = selected.value.slice(0, -1);
    const idx = selected.value[selected.value.length - 1] + 1;
    insertAtPath(r, parent, idx, node);
    selectPath([...parent, idx]);
  }
  bump();
}

function deleteSelected() {
  const r = root.value;
  if (!r) return;
  if (selected.value.length === 0) {
    config.value[size.value] = null;
    selected.value = [];
    bump();
    return;
  }
  removeAtPath(r, selected.value);
  selected.value = selected.value.slice(0, -1);
  bump();
}

function moveSelected(dir: -1 | 1) {
  const r = root.value;
  if (!r) return;
  const next = moveSibling(r, selected.value, dir);
  if (!next) return;
  selected.value = next;
  bump();
}

function duplicateSelected() {
  const r = root.value;
  if (!r) return;
  const next = duplicateAtPath(r, selected.value);
  if (!next) return;
  selected.value = next;
  bump();
}

function onTreeKeydown(ev: KeyboardEvent) {
  const el = ev.target as HTMLElement | null;
  if (el && /^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName)) return;
  if (ev.key === "Backspace" || ev.key === "Delete") {
    ev.preventDefault();
    deleteSelected();
    return;
  }
  if ((ev.altKey || ev.metaKey) && ev.key === "ArrowUp") {
    ev.preventDefault();
    moveSelected(-1);
    return;
  }
  if ((ev.altKey || ev.metaKey) && ev.key === "ArrowDown") {
    ev.preventDefault();
    moveSelected(1);
  }
}

function onFieldInput(field: InspectorField, raw: string) {
  const node = selectedNode.value;
  if (!node) return;
  let value: unknown = raw;
  if (field.kind === "number") {
    value = raw === "" ? null : Number(raw);
    if (typeof value === "number" && Number.isNaN(value)) return;
  } else if (field.kind === "boolean") {
    value = raw === "true";
  } else if (field.kind === "json") {
    if (raw.trim() === "") {
      delete node[field.key];
      bump();
      return;
    }
    try {
      value = JSON.parse(raw);
    } catch {
      return;
    }
  } else if (raw === "" && !field.required) {
    delete node[field.key];
    bump();
    return;
  }
  node[field.key] = value;
  bump();
}

function fieldDisplay(field: InspectorField): string {
  const node = selectedNode.value;
  if (!node || !(field.key in node) || node[field.key] === undefined || node[field.key] === null) {
    return "";
  }
  const v = node[field.key];
  if (field.kind === "json" || typeof v === "object") return JSON.stringify(v, null, 2);
  return String(v);
}

function pushPreview() {
  if (!hostEl.value) return;
  handle?.destroy();
  handle = renderWidget(hostEl.value, JSON.parse(JSON.stringify(config.value)), {
    size: size.value,
    onAction: (action, payload) => console.log("[widget]", action, payload),
  });
}

async function loadSchema() {
  try {
    const res = await fetch(withBase("/schemas/widget-config.v1.json"));
    if (!res.ok) throw new Error(String(res.status));
    schema.value = (await res.json()) as JsonSchema;
    schemaTypes.value = allTypesFromSchema(schema.value);
  } catch (e) {
    status.value = `Schema load failed: ${e}`;
    schemaTypes.value = ALL_TYPES;
  }
}

function onSizeChange() {
  selected.value = [];
  nextTick(pushPreview);
}

function insertAll() {
  config.value[size.value] = insertAllTypes();
  selected.value = [];
  status.value = `Inserted ${ALL_TYPES.length - 1} element types under vstack`;
  bump();
}

function reset() {
  config.value = createDefaultConfig();
  selected.value = [];
  status.value = "Reset";
  bump();
}

async function copyJson() {
  const text = JSON.stringify(config.value, null, 2);
  try {
    await navigator.clipboard.writeText(text);
    status.value = "Copied JSON";
  } catch {
    status.value = "Clipboard failed — use Download";
  }
}

function downloadJson() {
  const blob = new Blob([JSON.stringify(config.value, null, 2)], {
    type: "application/json",
  });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = "widget-config.json";
  a.click();
  URL.revokeObjectURL(a.href);
  status.value = "Downloaded widget-config.json";
}

function importClick() {
  fileInput.value?.click();
}

function onImport(ev: Event) {
  const input = ev.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = () => {
    try {
      const parsed = JSON.parse(String(reader.result)) as WidgetConfigState;
      warnings.value = softValidate(parsed);
      config.value = parsed;
      selected.value = [];
      status.value = `Imported ${file.name}`;
      bump();
    } catch (e) {
      status.value = `Import failed: ${e}`;
    }
    input.value = "";
  };
  reader.readAsText(file);
}

onMounted(() => {
  pushPreview();
  void loadSchema().then(() => {
    warnings.value = softValidate(config.value);
  });
});
onBeforeUnmount(() => {
  handle?.destroy();
  handle = null;
});

watch(size, onSizeChange);
</script>

<template>
  <div class="wc">
    <div class="wc__toolbar">
      <label>
        Size
        <select v-model="size">
          <option value="small">small</option>
          <option value="medium">medium</option>
          <option value="large">large</option>
        </select>
      </label>
      <button type="button" @click="insertAll">Insert all types</button>
      <button type="button" @click="reset">Reset</button>
      <button type="button" @click="copyJson">Copy JSON</button>
      <button type="button" @click="downloadJson">Download</button>
      <button type="button" @click="importClick">Import</button>
      <input ref="fileInput" type="file" accept="application/json,.json" hidden @change="onImport" />
      <span v-if="status" class="wc__status">{{ status }}</span>
    </div>

    <div class="wc__grid">
      <aside class="wc__panel wc__palette">
        <h3>Palette</h3>
        <p class="wc__hint">Click a type to insert at the target below.</p>
        <div class="wc__groups">
          <div v-for="g in paletteGroups" :key="g.id" class="wc__group">
            <div class="wc__group-title">{{ g.label }}</div>
            <button
              v-for="t in g.types"
              :key="t"
              type="button"
              class="wc__chip"
              @click="addType(t)"
            >
              {{ t }}
            </button>
          </div>
        </div>
      </aside>

      <aside class="wc__panel wc__tree-panel">
        <h3>Tree</h3>
        <p class="wc__hint wc__insert-target">{{ insertTargetLabel }}</p>
        <div
          ref="treeEl"
          class="wc__tree"
          tabindex="0"
          @keydown="onTreeKeydown"
        >
          <TreeNode
            v-if="root"
            :node="root"
            :path="[]"
            :selected="selected"
            @select="selectPath"
          />
          <p v-else class="wc__hint">No root for {{ size }}. Click a type.</p>
        </div>
        <div class="wc__tree-actions">
          <button type="button" :disabled="!canMoveUp" @click="moveSelected(-1)">Up</button>
          <button type="button" :disabled="!canMoveDown" @click="moveSelected(1)">Down</button>
          <button type="button" :disabled="!canDuplicate" @click="duplicateSelected">Duplicate</button>
          <button type="button" :disabled="!selectedNode" @click="deleteSelected">Delete</button>
        </div>
      </aside>

      <section class="wc__panel wc__preview-panel">
        <h3>Preview</h3>
        <div ref="hostEl" class="wc__preview" :style="frameSize" />
      </section>

      <section class="wc__panel wc__inspector">
        <h3>Inspector</h3>
        <template v-if="selectedNode">
          <p class="wc__hint">
            <code>{{ selectedNode.type }}</code>
            · path {{ selected.length ? selected.join(".") : "root" }}
          </p>
          <div v-if="inspectorFields.length" class="wc__fields">
            <label
              v-for="f in inspectorFields"
              :key="f.key"
              class="wc__field"
              :title="f.description || ''"
            >
              <span>
                {{ f.key }}
                <em v-if="f.required">*</em>
                <small v-if="f.group === 'style'">style</small>
              </span>
              <select
                v-if="f.kind === 'enum'"
                :value="fieldDisplay(f)"
                @change="onFieldInput(f, ($event.target as HTMLSelectElement).value)"
              >
                <option v-if="!f.required" value="">—</option>
                <option v-for="opt in f.enumValues" :key="opt" :value="opt">{{ opt }}</option>
              </select>
              <select
                v-else-if="f.kind === 'boolean'"
                :value="fieldDisplay(f)"
                @change="onFieldInput(f, ($event.target as HTMLSelectElement).value)"
              >
                <option value="">—</option>
                <option value="true">true</option>
                <option value="false">false</option>
              </select>
              <input
                v-else-if="f.kind === 'string' || f.kind === 'number'"
                :type="f.kind === 'number' ? 'number' : 'text'"
                :value="fieldDisplay(f)"
                @change="onFieldInput(f, ($event.target as HTMLInputElement).value)"
              />
              <textarea
                v-else
                rows="3"
                spellcheck="false"
                :value="fieldDisplay(f)"
                @change="onFieldInput(f, ($event.target as HTMLTextAreaElement).value)"
              />
            </label>
          </div>
          <p v-else class="wc__hint">No schema fields (schema still loading?).</p>
        </template>
        <p v-else class="wc__hint">Select a node in the tree.</p>

        <ul v-if="warnings.length" class="wc__warn">
          <li v-for="(w, i) in warnings" :key="i">{{ w }}</li>
        </ul>
      </section>
    </div>
  </div>
</template>

<style scoped>
.wc {
  margin: 1rem 0 2rem;
  border: 1px solid var(--vp-c-divider);
  border-radius: 10px;
  background: var(--vp-c-bg-soft);
  padding: 0.75rem;
}

.wc__toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  align-items: center;
  margin-bottom: 0.75rem;
}

.wc__toolbar label {
  display: inline-flex;
  gap: 0.35rem;
  align-items: center;
  font-size: 0.85rem;
}

.wc__toolbar button,
.wc__tree-actions button,
.wc__chip {
  border: 1px solid var(--vp-c-divider);
  background: var(--vp-c-bg);
  color: var(--vp-c-text-1);
  border-radius: 6px;
  padding: 0.25rem 0.6rem;
  font-size: 0.8rem;
  cursor: pointer;
}

.wc__toolbar button:hover,
.wc__chip:hover,
.wc__tree-actions button:hover:not(:disabled) {
  border-color: var(--vp-c-brand-1);
}

.wc__tree-actions button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.wc__status {
  font-size: 0.8rem;
  color: var(--vp-c-brand-1);
}

.wc__grid {
  display: grid;
  grid-template-columns: minmax(220px, 0.9fr) minmax(200px, auto) minmax(280px, 1.3fr);
  grid-template-areas:
    "palette palette palette"
    "tree preview inspector";
  gap: 0.75rem;
  align-items: start;
}

.wc__palette {
  grid-area: palette;
  min-height: 0;
}

.wc__tree-panel {
  grid-area: tree;
}

.wc__preview-panel {
  grid-area: preview;
}

.wc__inspector {
  grid-area: inspector;
}

.wc__groups {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem 1.25rem;
}

@media (max-width: 960px) {
  .wc__grid {
    grid-template-columns: 1fr;
    grid-template-areas:
      "palette"
      "tree"
      "preview"
      "inspector";
  }
}

.wc__panel {
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  padding: 0.65rem;
  background: var(--vp-c-bg);
  min-height: 280px;
}

.wc__panel h3 {
  margin: 0 0 0.5rem;
  font-size: 0.95rem;
}

.wc__hint {
  font-size: 0.75rem;
  color: var(--vp-c-text-2);
  margin: 0 0 0.5rem;
}

.wc__group {
  margin: 0;
}

.wc__group-title {
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--vp-c-text-2);
  margin-bottom: 0.25rem;
}

.wc__chip {
  display: inline-block;
  margin: 0 0.25rem 0.25rem 0;
}

.wc__tree {
  min-height: 160px;
  max-height: 420px;
  overflow: auto;
  font-size: 0.82rem;
  outline: none;
}

.wc__tree:focus {
  box-shadow: inset 0 0 0 1px var(--vp-c-brand-1);
}

.wc__insert-target {
  font-family: var(--vp-font-family-mono);
}

.wc :deep(.wc__tn-kids) {
  margin-left: 0.85rem;
  border-left: 1px dashed var(--vp-c-divider);
  padding-left: 0.35rem;
}

.wc :deep(.wc__tn-row) {
  display: flex;
  gap: 0.35rem;
  align-items: center;
  padding: 0.2rem 0.35rem;
  border-radius: 4px;
  cursor: pointer;
}

.wc :deep(.wc__tn-row:hover) {
  background: var(--vp-c-bg-soft);
}

.wc :deep(.wc__tn-row.is-selected) {
  background: color-mix(in srgb, var(--vp-c-brand-1) 18%, transparent);
  outline: 1px solid var(--vp-c-brand-1);
}

.wc :deep(.wc__tn-type) {
  font-family: var(--vp-font-family-mono);
}

.wc :deep(.wc__tn-meta) {
  font-size: 0.7rem;
  color: var(--vp-c-text-2);
}

.wc__tree-actions {
  margin-top: 0.5rem;
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}

.wc__preview {
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  overflow: hidden;
  background: #0f172a;
  margin: 0 auto 1rem;
}

.wc__preview .widget-host {
  width: 100%;
  height: 100%;
}

.wc__fields {
  display: grid;
  gap: 0.45rem;
  max-height: 480px;
  overflow: auto;
}

.wc__field {
  display: grid;
  gap: 0.2rem;
  font-size: 0.78rem;
}

.wc__field em {
  color: var(--vp-c-brand-1);
  font-style: normal;
}

.wc__field small {
  margin-left: 0.35rem;
  color: var(--vp-c-text-2);
}

.wc__field input,
.wc__field select,
.wc__field textarea {
  width: 100%;
  border: 1px solid var(--vp-c-divider);
  border-radius: 4px;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-1);
  padding: 0.25rem 0.4rem;
  font: inherit;
}

.wc__field textarea {
  font-family: var(--vp-font-family-mono);
  font-size: 0.75rem;
}

.wc__warn {
  margin: 0.75rem 0 0;
  padding-left: 1.1rem;
  color: #f59e0b;
  font-size: 0.78rem;
}
</style>
