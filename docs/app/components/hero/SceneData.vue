<script setup lang="ts">
/** Hero scene: the app pushes values with setWidgetData; the layout stays, the numbers move. */
import { clamp01, lerp } from './math'

const props = defineProps<{ t: number }>()
const T = computed(() => props.t)

const APP = { x: 50, y: 120, w: 120, h: 120 }
const CARD = { x: 330, y: 60, w: 260, h: 240 }
const SEND = 650 // one packet per SEND ms
const FLIGHT = 520
const series = [38, 44, 41, 52, 49, 61, 58, 66, 63, 72, 69, 78, 74, 81]
const from: [number, number] = [APP.x + APP.w, APP.y + APP.h / 2]
const to: [number, number] = [CARD.x, CARD.y + 110]
const curve = `M${from[0]},${from[1]} C${from[0] + 80},${from[1] - 60} ${to[0] - 80},${to[1] - 60} ${to[0]},${to[1]}`
/** Packets landed so far (each one shifts the chart and updates the numbers). */
const landed = computed(() => Math.max(0, Math.floor((T.value - FLIGHT) / SEND) + 1))
const shown = computed(() => series.slice(Math.min(landed.value, series.length - 7), Math.min(landed.value, series.length - 7) + 7))
const cpu = computed(() => shown.value[shown.value.length - 1]!)
const users = computed(() => 1200 + landed.value * 37)
const flying = computed(() => {
  const out: { k: number }[] = []
  for (let i = 0; i * SEND <= T.value; i++) {
    const k = (T.value - i * SEND) / FLIGHT
    if (k >= 0 && k <= 1 && i < 8) out.push({ k })
  }
  return out
})
/** Point on the cubic link at k. */
function at(k: number): [number, number] {
  const p = [from, [from[0] + 80, from[1] - 60], [to[0] - 80, to[1] - 60], to] as [number, number][]
  const u = 1 - k
  const c = [u * u * u, 3 * u * u * k, 3 * u * k * k, k * k * k]
  return [c.reduce((s, w, i) => s + w * p[i]![0], 0), c.reduce((s, w, i) => s + w * p[i]![1], 0)]
}
const CH = { x: CARD.x + 22, y: CARD.y + 120, w: CARD.w - 44, h: 80 }
const line = computed(() =>
  shown.value.map((v, i) => `${i ? 'L' : 'M'}${CH.x + (i / 6) * CH.w},${CH.y + CH.h - ((v - 30) / 60) * CH.h}`).join(' '),
)
const pulse = computed(() => clamp01(1 - ((T.value - FLIGHT) % SEND) / 300))
</script>

<template>
  <svg viewBox="0 0 640 420" class="block w-full" role="img" aria-label="The app pushes values to a dashboard widget; the chart and numbers update without a new layout">
    <!-- The app. -->
    <rect :x="APP.x" :y="APP.y" :width="APP.w" :height="APP.h" rx="28" class="dt-app" />
    <rect :x="APP.x + 36" :y="APP.y + 30" width="48" height="48" rx="12" fill="#24c8db" />
    <text :x="APP.x + 60" :y="APP.y + 61" text-anchor="middle" class="dt-app-glyph">T</text>
    <text :x="APP.x + 60" :y="APP.y + 102" text-anchor="middle" class="dt-app-name">Your app</text>

    <path :d="curve" class="dt-link" />
    <g v-for="(p, i) in flying" :key="i" :transform="`translate(${at(p.k)[0]},${at(p.k)[1]})`">
      <rect x="-30" y="-11" width="60" height="22" rx="11" class="dt-packet" />
      <text y="4" text-anchor="middle" class="dt-packet-text">data</text>
    </g>

    <!-- The widget. -->
    <rect :x="CARD.x" :y="CARD.y" :width="CARD.w" :height="CARD.h" rx="26" class="dt-card" :style="{ strokeOpacity: 0.15 + 0.6 * pulse }" />
    <text :x="CARD.x + 22" :y="CARD.y + 34" class="dt-label">CPU</text>
    <text :x="CARD.x + 22" :y="CARD.y + 76" class="dt-big">{{ cpu }}%</text>
    <text :x="CARD.x + CARD.w - 22" :y="CARD.y + 34" text-anchor="end" class="dt-label">Users</text>
    <text :x="CARD.x + CARD.w - 22" :y="CARD.y + 76" text-anchor="end" class="dt-mid">{{ users.toLocaleString('en-US') }}</text>
    <rect :x="CH.x" :y="CH.y - 18" :width="CH.w" height="6" rx="3" fill="#22c55e2a" />
    <rect :x="CH.x" :y="CH.y - 18" :width="CH.w * cpu / 100" height="6" rx="3" fill="#22c55e" />
    <path :d="`${line} L${CH.x + CH.w},${CH.y + CH.h} L${CH.x},${CH.y + CH.h} Z`" fill="#22c55e22" />
    <path :d="line" class="dt-line" />
    <circle :cx="CH.x + CH.w" :cy="CH.y + CH.h - ((cpu - 30) / 60) * CH.h" :r="lerp(4, 7, pulse)" fill="#22c55e" />

    <text x="320" y="352" text-anchor="middle" class="dt-code">setWidgetData("stats", { cpu, users })</text>
    <text x="320" y="376" text-anchor="middle" class="dt-note">Only the values travel; {{ '{' }}{{ '{' }}data.cpu{{ '}' }}{{ '}' }} in the layout picks them up</text>
  </svg>
</template>

<style scoped>
.dt-app { fill: var(--ui-bg-elevated); stroke: var(--ui-border-accented); }
.dt-app-glyph { fill: #fff; font: 800 24px ui-sans-serif, system-ui, sans-serif; }
.dt-app-name { fill: var(--ui-text-highlighted); font: 600 12px ui-sans-serif, system-ui, sans-serif; }
.dt-link { fill: none; stroke: var(--ui-border-accented); stroke-width: 1.6; stroke-dasharray: 5 5; }
.dt-packet { fill: var(--ui-primary); }
.dt-packet-text { fill: var(--ui-bg); font: 700 11px ui-monospace, SFMono-Regular, Menlo, monospace; }
.dt-card { fill: #101a2e; stroke: #22c55e; stroke-width: 1.5; }
.dt-label { fill: #94a3b8; font: 600 12px ui-sans-serif, system-ui, sans-serif; }
.dt-big { fill: #fff; font: 700 34px ui-sans-serif, system-ui, sans-serif; font-variant-numeric: tabular-nums; }
.dt-mid { fill: #e2e8f0; font: 700 22px ui-sans-serif, system-ui, sans-serif; font-variant-numeric: tabular-nums; }
.dt-line { fill: none; stroke: #22c55e; stroke-width: 2.4; stroke-linejoin: round; }
.dt-code { fill: var(--ui-text-highlighted); font: 600 12px ui-monospace, SFMono-Regular, Menlo, monospace; }
.dt-note { fill: var(--ui-text-muted); font: 500 12px ui-sans-serif, system-ui, sans-serif; }
</style>
