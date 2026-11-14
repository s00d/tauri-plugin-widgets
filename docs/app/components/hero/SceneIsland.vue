<script setup lang="ts">
/** Hero scene: a live activity in the desktop Dynamic Island — compact around the notch,
 *  expanded on hover, collapsed again. Region names show which field draws what. */
import Cursor from './Cursor.vue'
import { clamp01, lerp, path, span } from './math'

const props = defineProps<{ t: number }>()
const T = computed(() => props.t)

// Screen 600x290 at (20, 18); the notch at the top centre.
const SCREEN = { x: 20, y: 18, w: 600, h: 290 }
const CX = 320
const open = computed(() => span(T.value, 1250, 1650) * (1 - span(T.value, 4000, 4400)))
const progress = computed(() => lerp(0.18, 0.92, clamp01(T.value / 5400)))
const speed = computed(() => Math.round(36 + 9 * Math.sin(T.value / 260)))
const pill = computed(() => {
  const k = open.value
  const w = lerp(270, 400, k)
  const h = lerp(30, 168, k)
  return { x: CX - w / 2, y: SCREEN.y, w, h, r: lerp(15, 30, k) }
})
const cursor = computed(() =>
  path(T.value, [
    { at: 0, x: 520, y: 260 },
    { at: 900, x: 520, y: 250 },
    { at: 1300, x: CX + 40, y: 36 },
    { at: 3700, x: CX + 60, y: 100 },
    { at: 4100, x: 540, y: 270 },
  ]),
)
const inner = computed(() => clamp01((open.value - 0.55) / 0.45))
const compact = computed(() => clamp01(1 - open.value * 2.5))
const state = computed(() => (open.value > 0.5 ? 'Expanded on hover: islandLeading · islandTrailing · layout' : 'Compact: compactLeading · compactTrailing around the notch'))
const ring = (p: number, r: number) => {
  const a = p * Math.PI * 2 - Math.PI / 2
  return `M${CX - 108},${SCREEN.y + 15 - r} A${r},${r} 0 ${p > 0.5 ? 1 : 0} 1 ${CX - 108 + r * Math.cos(a)},${SCREEN.y + 15 + r * Math.sin(a)}`
}
</script>

<template>
  <svg viewBox="0 0 640 420" class="block w-full" role="img" aria-label="A live activity in the desktop Dynamic Island, compact around the notch and expanded on hover">
    <defs>
      <radialGradient id="is-wall" cx="50%" cy="0%" r="120%">
        <stop offset="0" stop-color="#3b4a85" />
        <stop offset="0.55" stop-color="#18203d" />
        <stop offset="1" stop-color="#0b0e1c" />
      </radialGradient>
      <clipPath id="is-screen"><rect :x="SCREEN.x" :y="SCREEN.y" :width="SCREEN.w" :height="SCREEN.h" rx="14" /></clipPath>
    </defs>

    <!-- Laptop screen with a menu bar and the camera notch. -->
    <rect :x="SCREEN.x - 6" :y="SCREEN.y - 6" :width="SCREEN.w + 12" :height="SCREEN.h + 12" rx="20" class="is-bezel" />
    <g clip-path="url(#is-screen)">
      <rect :x="SCREEN.x" :y="SCREEN.y" :width="SCREEN.w" :height="SCREEN.h" fill="url(#is-wall)" />
      <rect :x="SCREEN.x" :y="SCREEN.y" :width="SCREEN.w" height="30" fill="#00000055" />
      <text :x="SCREEN.x + 18" :y="SCREEN.y + 20" class="is-menu"> Finder File Edit View</text>
      <text :x="SCREEN.x + SCREEN.w - 18" :y="SCREEN.y + 20" text-anchor="end" class="is-menu">Wed 14:30</text>

      <!-- The pill: compact around the notch, expanded on hover. -->
      <rect :x="pill.x" :y="pill.y - 20" :width="pill.w" :height="pill.h + 20" :rx="pill.r" class="is-pill" />
      <g :opacity="compact">
        <circle :cx="CX - 108" :cy="SCREEN.y + 15" r="8" fill="none" stroke="#60a5fa33" stroke-width="2.5" />
        <path :d="ring(progress, 8)" fill="none" stroke="#60a5fa" stroke-width="2.5" stroke-linecap="round" />
        <text :x="CX + 112" :y="SCREEN.y + 20" text-anchor="end" class="is-pct">{{ Math.round(progress * 100) }}%</text>
      </g>
      <g :opacity="inner">
        <text :x="pill.x + 22" :y="SCREEN.y + 20" class="is-title">macOS-27.dmg</text>
        <text :x="pill.x + pill.w - 22" :y="SCREEN.y + 20" text-anchor="end" class="is-speed">{{ speed }} MB/s</text>
        <rect :x="pill.x + 22" :y="SCREEN.y + 50" :width="pill.w - 44" height="7" rx="3.5" fill="#60a5fa2e" />
        <rect :x="pill.x + 22" :y="SCREEN.y + 50" :width="(pill.w - 44) * progress" height="7" rx="3.5" fill="#60a5fa" />
        <text :x="pill.x + 22" :y="SCREEN.y + 78" class="is-sub">{{ (progress * 4.2).toFixed(2) }} / 4.20 GB</text>
        <text :x="pill.x + pill.w - 22" :y="SCREEN.y + 78" text-anchor="end" class="is-sub">{{ Math.max(1, Math.round((1 - progress) * 12)) }}s left</text>
        <rect :x="pill.x + 22" :y="SCREEN.y + 96" :width="(pill.w - 56) / 2" height="30" rx="9" fill="#1e3a5f" />
        <text :x="pill.x + 22 + (pill.w - 56) / 4" :y="SCREEN.y + 115" text-anchor="middle" class="is-btn">Show in Finder</text>
        <rect :x="pill.x + 34 + (pill.w - 56) / 2" :y="SCREEN.y + 96" :width="(pill.w - 56) / 2" height="30" rx="9" fill="#3b1d24" />
        <text :x="pill.x + 34 + (3 * (pill.w - 56)) / 4" :y="SCREEN.y + 115" text-anchor="middle" class="is-btn" fill="#f87171">Cancel</text>
      </g>
      <!-- Camera notch (drawn over the pill: the regions sit either side of it). -->
      <rect :x="CX - 62" :y="SCREEN.y - 10" width="124" height="34" rx="10" fill="#000" />
      <Cursor :x="cursor.x" :y="cursor.y" />
    </g>

    <text x="320" y="345" text-anchor="middle" class="is-state">{{ state }}</text>
    <text x="320" y="368" text-anchor="middle" class="is-code">startActivity({ id, title, progress, compactLeading, islandLeading, layout })</text>
  </svg>
</template>

<style scoped>
.is-bezel { fill: #0a0a0c; stroke: var(--ui-border-accented); }
.is-menu { fill: #e5e7ebcc; font: 500 11px ui-sans-serif, system-ui, sans-serif; }
.is-pill { fill: #000; stroke: #60a5fa; stroke-width: 1.2; }
.is-pct { fill: #60a5fa; font: 700 12px ui-sans-serif, system-ui, sans-serif; }
.is-title { fill: #fff; font: 600 13px ui-sans-serif, system-ui, sans-serif; }
.is-speed { fill: #60a5fa; font: 600 12px ui-sans-serif, system-ui, sans-serif; }
.is-sub { fill: #cbd5e1; font: 500 11px ui-sans-serif, system-ui, sans-serif; }
.is-btn { fill: #93c5fd; font: 600 11px ui-sans-serif, system-ui, sans-serif; }
.is-state { fill: var(--ui-text-highlighted); font: 600 13px ui-sans-serif, system-ui, sans-serif; }
.is-code { fill: var(--ui-text-muted); font: 500 11px ui-monospace, SFMono-Regular, Menlo, monospace; }
</style>
