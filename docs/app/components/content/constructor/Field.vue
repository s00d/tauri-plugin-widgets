<script setup lang="ts">
import type { FieldSpec } from './meta'

/** One inspector control. Emits the new value; `undefined` removes the property. */
const props = defineProps<{ spec: FieldSpec; value: unknown; options?: string[] }>()
const emit = defineEmits<{ update: [value: unknown] }>()

const hex = computed(() => (typeof props.value === 'string' && /^#[0-9a-f]{3,8}$/i.test(props.value) ? props.value : ''))
/** `<input type=color>` needs #rrggbb. */
const pickerValue = computed(() => {
  const h = hex.value
  if (h.length === 4) return '#' + [...h.slice(1)].map((c) => c + c).join('')
  return h.length >= 7 ? h.slice(0, 7) : '#000000'
})
const isComplex = computed(() => props.value !== null && typeof props.value === 'object')
const enumItems = computed(() => [{ label: '—', value: '__unset' }, ...(props.options ?? []).map((v) => ({ label: v, value: v }))])

function num(v: string | number) {
  if (v === '' || v === null) return emit('update', undefined)
  const n = Number(v)
  emit('update', Number.isFinite(n) ? n : undefined)
}
</script>

<template>
  <div class="cf-field">
    <label class="cf-label">{{ spec.label }}</label>

    <p v-if="isComplex" class="cf-complex">{{ JSON.stringify(value) }} <span>· edit in JSON</span></p>

    <UTextarea
      v-else-if="spec.kind === 'textarea'"
      :model-value="(value as string) ?? ''"
      :rows="1"
      autoresize
      size="sm"
      class="w-full"
      @update:model-value="emit('update', $event || undefined)"
    />

    <UInput
      v-else-if="spec.kind === 'text'"
      :model-value="(value as string) ?? ''"
      :placeholder="spec.placeholder"
      size="sm"
      class="w-full"
      @update:model-value="emit('update', $event || undefined)"
    />

    <UInput
      v-else-if="spec.kind === 'number'"
      type="number"
      :model-value="(value as number) ?? ''"
      :min="spec.min"
      :max="spec.max"
      :step="spec.step ?? 1"
      size="sm"
      class="w-full"
      @update:model-value="num"
    />

    <div v-else-if="spec.kind === 'slider'" class="cf-row">
      <USlider
        :model-value="typeof value === 'number' ? value : spec.def ?? spec.min ?? 0"
        :min="spec.min ?? 0"
        :max="spec.max ?? 1"
        :step="spec.step ?? 0.01"
        size="sm"
        class="flex-1"
        @update:model-value="emit('update', $event)"
      />
      <span class="cf-num" :class="{ unset: typeof value !== 'number' }">{{ typeof value === 'number' ? Math.round(value * 100) / 100 : spec.def ?? '—' }}</span>
    </div>

    <div v-else-if="spec.kind === 'color'" class="cf-row">
      <input type="color" class="cf-swatch" :value="pickerValue" @input="emit('update', ($event.target as HTMLInputElement).value)" />
      <UInput
        :model-value="(value as string) ?? ''"
        placeholder="#rrggbb or label"
        size="sm"
        class="flex-1"
        @update:model-value="emit('update', $event || undefined)"
      />
    </div>

    <USelect
      v-else-if="spec.kind === 'enum'"
      :model-value="(value as string) ?? '__unset'"
      :items="enumItems"
      size="sm"
      class="w-full"
      @update:model-value="emit('update', $event === '__unset' ? undefined : $event)"
    />

    <USwitch v-else-if="spec.kind === 'bool'" :model-value="!!value" size="sm" @update:model-value="emit('update', $event || undefined)" />
  </div>
</template>

<style scoped>
.cf-field { display: grid; grid-template-columns: 6.5rem 1fr; align-items: center; gap: 0.5rem; }
.cf-label { font-size: 0.78rem; color: var(--ui-text-muted); }
.cf-row { display: flex; align-items: center; gap: 0.5rem; }
.cf-num { min-width: 2.5rem; text-align: right; font-size: 0.78rem; font-variant-numeric: tabular-nums; color: var(--ui-text-toned); }
.cf-num.unset { color: var(--ui-text-dimmed); }
.cf-swatch { width: 28px; height: 28px; padding: 0; border: 1px solid var(--ui-border); border-radius: 6px; background: none; cursor: pointer; }
.cf-complex { font-size: 0.72rem; color: var(--ui-text-toned); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin: 0; }
.cf-complex span { color: var(--ui-text-dimmed); }
</style>
