<script setup lang="ts">
/** Hero scene: scheduled entries switch the widget on their own times, the app asleep. */
import { clamp01, lerp } from './math'

const props = defineProps<{ t: number }>()
const T = computed(() => props.t)

const entries = [
  { at: '09:00', title: 'Standup', sub: 'in 15 min · Room 4', icon: '☕', a: '#f59e0b', b: '#7c2d12' },
  { at: '12:30', title: 'Lunch', sub: 'with Anna · Café', icon: '🥗', a: '#22c55e', b: '#14532d' },
  { at: '17:00', title: 'Gym', sub: 'Legs · 45 min', icon: '🏋️', a: '#8b5cf6', b: '#3b0764' },
  { at: '22:00', title: 'Free', sub: 'No more events', icon: '🌙', a: '#3b82f6', b: '#0f172a' },
]
const STEP = 1300
const AXIS = { x0: 110, x1: 530, y: 330 }
const ex = (i: number) => lerp(AXIS.x0, AXIS.x1, i / (entries.length - 1))
// The "now" marker walks the axis; it reaches entry i at i * STEP + 400.
const now = computed(() => clamp01((T.value - 400) / (STEP * (entries.length - 1))) * (entries.length - 1))
const active = computed(() => Math.min(entries.length - 1, Math.floor(now.value + 0.001)))
/** Slide-in progress of entry i (the transition: slide). */
const slide = (i: number) => (i === 0 ? 1 : clamp01((now.value - i) * 6 + 1))
const CARD = { x: 220, y: 50, w: 200, h: 200 }
</script>

<template>
  <svg viewBox="0 0 640 420" class="block w-full" role="img" aria-label="A widget switching between scheduled entries through the day">
    <defs>
      <clipPath id="tl-card"><rect :x="CARD.x" :y="CARD.y" :width="CARD.w" :height="CARD.h" rx="26" /></clipPath>
      <linearGradient v-for="(e, i) in entries" :id="`tl-g${i}`" :key="i" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" :stop-color="e.a" />
        <stop offset="1" :stop-color="e.b" />
      </linearGradient>
    </defs>

    <g clip-path="url(#tl-card)">
      <g v-for="(e, i) in entries" v-show="i <= active + 1" :key="i" :transform="`translate(0, ${(1 - slide(i)) * CARD.h})`">
        <rect :x="CARD.x" :y="CARD.y" :width="CARD.w" :height="CARD.h" :fill="`url(#tl-g${i})`" />
        <text :x="CARD.x + 22" :y="CARD.y + 50" class="tl-icon">{{ e.icon }}</text>
        <text :x="CARD.x + CARD.w - 22" :y="CARD.y + 40" text-anchor="end" class="tl-time">{{ e.at }}</text>
        <text :x="CARD.x + 22" :y="CARD.y + 140" class="tl-title">{{ e.title }}</text>
        <text :x="CARD.x + 22" :y="CARD.y + 166" class="tl-sub">{{ e.sub }}</text>
      </g>
    </g>
    <rect :x="CARD.x" :y="CARD.y" :width="CARD.w" :height="CARD.h" rx="26" fill="none" stroke="#ffffff22" />

    <!-- Day axis with the entries and the moving "now". -->
    <line :x1="AXIS.x0" :x2="AXIS.x1" :y1="AXIS.y" :y2="AXIS.y" class="tl-axis" />
    <line :x1="AXIS.x0" :x2="lerp(AXIS.x0, AXIS.x1, now / (entries.length - 1))" :y1="AXIS.y" :y2="AXIS.y" class="tl-axis on" />
    <g v-for="(e, i) in entries" :key="`a${i}`">
      <circle :cx="ex(i)" :cy="AXIS.y" r="7" class="tl-dot" :class="{ on: i <= active }" :style="i <= active ? { fill: e.a } : {}" />
      <text :x="ex(i)" :y="AXIS.y - 16" text-anchor="middle" class="tl-at">{{ e.at }}</text>
      <text :x="ex(i)" :y="AXIS.y + 26" text-anchor="middle" class="tl-name">{{ e.title }}</text>
    </g>
    <path :transform="`translate(${lerp(AXIS.x0, AXIS.x1, now / (entries.length - 1))}, ${AXIS.y - 34})`" d="M-5 0 L5 0 L0 7 Z" class="tl-now" />

    <text x="320" y="392" text-anchor="middle" class="tl-code">entries: [{ at, small }, …] · transition: "slide"</text>
  </svg>
</template>

<style scoped>
.tl-icon { font-size: 30px; }
.tl-time { fill: #ffffffcc; font: 600 13px ui-sans-serif, system-ui, sans-serif; }
.tl-title { fill: #fff; font: 700 28px ui-sans-serif, system-ui, sans-serif; }
.tl-sub { fill: #ffffffcc; font: 500 13px ui-sans-serif, system-ui, sans-serif; }
.tl-axis { stroke: var(--ui-border-accented); stroke-width: 2; }
.tl-axis.on { stroke: var(--ui-primary); }
.tl-dot { fill: var(--ui-bg-elevated); stroke: var(--ui-border-accented); stroke-width: 2; }
.tl-at { fill: var(--ui-text-highlighted); font: 600 12px ui-sans-serif, system-ui, sans-serif; }
.tl-name { fill: var(--ui-text-muted); font: 500 11px ui-sans-serif, system-ui, sans-serif; }
.tl-now { fill: var(--ui-primary); }
.tl-code { fill: var(--ui-text-muted); font: 500 11px ui-monospace, SFMono-Regular, Menlo, monospace; }
</style>
