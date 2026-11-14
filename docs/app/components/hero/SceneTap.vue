<script setup lang="ts">
/** Hero scene: taps on a widget change local state in place (`set`), no app round trip. */
import Cursor from './Cursor.vue'
import { clamp01, path } from './math'

const props = defineProps<{ t: number }>()
const T = computed(() => props.t)

const CARD = { x: 190, y: 40, w: 260, h: 250 }
const rows = [
  { key: 'milk', label: 'Buy milk', y: 112 },
  { key: 'call', label: 'Call the bank', y: 150 },
  { key: 'ship', label: 'Ship v0.8', y: 188 },
]
// Taps: [time, target]. A row toggles, "+1" increments.
const taps: { at: number; x: number; y: number; patch: string; key: string }[] = [
  { at: 1000, x: 222, y: 112, key: 'milk', patch: 'set: { milk: "$toggle" }' },
  { at: 2000, x: 222, y: 188, key: 'ship', patch: 'set: { ship: "$toggle" }' },
  { at: 3000, x: 404, y: 252, key: 'water', patch: 'set: { water: "$inc" }' },
  { at: 3600, x: 404, y: 252, key: 'water', patch: 'set: { water: "$inc" }' },
  { at: 4400, x: 222, y: 112, key: 'milk', patch: 'set: { milk: "$toggle" }' },
]
const done = (key: string) => taps.filter((p) => p.key === key && T.value >= p.at).length % 2 === 1
const water = computed(() => 3 + taps.filter((p) => p.key === 'water' && T.value >= p.at).length)
const last = computed(() => [...taps].reverse().find((p) => T.value >= p.at))
const cursor = computed(() =>
  path(T.value, [
    { at: 0, x: 560, y: 330 },
    ...taps.flatMap((p) => [{ at: p.at - 380, x: p.x, y: p.y }, { at: p.at + 200, x: p.x, y: p.y }]),
    { at: 5300, x: 560, y: 330 },
  ]),
)
const ripple = computed(() => {
  const p = last.value
  return p ? clamp01((T.value - p.at) / 280) : 0
})
const down = computed(() => taps.some((p) => T.value >= p.at - 90 && T.value < p.at + 90))
const chipIn = computed(() => (last.value ? clamp01((T.value - last.value.at) / 200) : 0))
</script>

<template>
  <svg viewBox="0 0 640 420" class="block w-full" role="img" aria-label="Tapping rows of a tasks widget toggles them in place through local state">
    <rect :x="CARD.x" :y="CARD.y" :width="CARD.w" :height="CARD.h" rx="26" class="tp-card" />
    <text :x="CARD.x + 24" :y="CARD.y + 38" class="tp-title">Today</text>
    <text :x="CARD.x + CARD.w - 24" :y="CARD.y + 38" text-anchor="end" class="tp-count">{{ rows.filter((r) => done(r.key)).length }}/{{ rows.length }}</text>

    <g v-for="r in rows" :key="r.key">
      <circle cx="222" :cy="r.y" r="10" class="tp-box" :class="{ on: done(r.key) }" />
      <path v-if="done(r.key)" :d="`M216 ${r.y} l4 4 l8 -8`" class="tp-check" />
      <text x="244" :y="r.y + 5" class="tp-row" :class="{ on: done(r.key) }">{{ r.label }}</text>
    </g>

    <line :x1="CARD.x + 20" :x2="CARD.x + CARD.w - 20" y1="218" y2="218" class="tp-div" />
    <text x="214" y="257" class="tp-row">Water</text>
    <g v-for="i in 8" :key="i">
      <rect :x="262 + (i - 1) * 13" y="244" width="9" height="18" rx="3" :class="i <= water ? 'tp-cup on' : 'tp-cup'" />
    </g>
    <rect x="384" y="238" width="40" height="28" rx="9" class="tp-plus" />
    <text x="404" y="257" text-anchor="middle" class="tp-plus-text">+1</text>

    <!-- The patch the tap applied. -->
    <g v-if="last" :opacity="chipIn" :transform="`translate(0, ${(1 - chipIn) * 8})`">
      <rect x="170" y="318" width="300" height="32" rx="10" class="tp-chip" />
      <text x="320" y="339" text-anchor="middle" class="tp-chip-text">{{ last.patch }}</text>
    </g>
    <text x="320" y="378" text-anchor="middle" class="tp-note">State lives in the widget: no app launch, no network</text>

    <Cursor :x="cursor.x" :y="cursor.y" :down="down" :ripple="ripple" />
  </svg>
</template>

<style scoped>
.tp-card { fill: #16213b; stroke: #ffffff1a; }
.tp-title { fill: #fff; font: 700 17px ui-sans-serif, system-ui, sans-serif; }
.tp-count { fill: #94a3b8; font: 600 13px ui-sans-serif, system-ui, sans-serif; }
.tp-box { fill: none; stroke: #64748b; stroke-width: 2; transition: fill 0.2s, stroke 0.2s; }
.tp-box.on { fill: #22c55e; stroke: #22c55e; }
.tp-check { fill: none; stroke: #fff; stroke-width: 2.4; stroke-linecap: round; stroke-linejoin: round; }
.tp-row { fill: #e2e8f0; font: 500 14px ui-sans-serif, system-ui, sans-serif; }
.tp-row.on { fill: #64748b; text-decoration: line-through; }
.tp-div { stroke: #ffffff14; }
.tp-cup { fill: #60a5fa22; }
.tp-cup.on { fill: #60a5fa; }
.tp-plus { fill: #1e3a5f; }
.tp-plus-text { fill: #93c5fd; font: 700 13px ui-sans-serif, system-ui, sans-serif; }
.tp-chip { fill: var(--ui-bg-elevated); stroke: var(--ui-primary); }
.tp-chip-text { fill: var(--ui-text-highlighted); font: 600 12px ui-monospace, SFMono-Regular, Menlo, monospace; }
.tp-note { fill: var(--ui-text-muted); font: 500 12px ui-sans-serif, system-ui, sans-serif; }
</style>
