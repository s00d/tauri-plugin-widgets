<script setup lang="ts">
/** Hero scene: config nodes fly in as a linked tree, snap into a widget, light up the platforms. */
import { clamp01, ease, lerp } from './math'

type Piece = {
  id: string
  /** Node label while it is still a config node. */
  label: string
  /** Chip centre at the start (scattered). */
  from: [number, number]
  /** Where its visual sits in the widget (top-left of its box). */
  to: [number, number]
  /** Flight start (ms). */
  delay: number
}

// Scene in a 640x420 viewBox. The widget card: x 220..420, y 60..260.
const CARD = { x: 220, y: 60, w: 200, h: 200 }
const ROOT_FROM: [number, number] = [96, 210]
const pieces: Piece[] = [
  { id: 'image', label: 'image', from: [120, 70], to: [240, 82], delay: 700 },
  { id: 'city', label: 'text', from: [520, 60], to: [270, 84], delay: 820 },
  { id: 'temp', label: 'text', from: [560, 170], to: [240, 112], delay: 940 },
  { id: 'label', label: 'text', from: [540, 290], to: [240, 176], delay: 1060 },
  { id: 'bar', label: 'progress', from: [140, 330], to: [240, 192], delay: 1180 },
  { id: 'days', label: 'hstack', from: [330, 360], to: [240, 214], delay: 1300 },
]
const FLY_MS = 900
const SNAP_AT = 2350
const PLATFORM_AT = 2900

const platforms = [
  { id: 'ios', label: 'iOS', icon: 'i-simple-icons-apple' },
  { id: 'android', label: 'Android', icon: 'i-simple-icons-android' },
  { id: 'macos', label: 'macOS', icon: 'i-lucide-laptop' },
  { id: 'windows', label: 'Windows', icon: 'i-lucide-app-window' },
  { id: 'linux', label: 'Linux', icon: 'i-simple-icons-linux' },
] as const
// Platform chips along the bottom; links leave the card's bottom edge.
const platformX = (i: number) => 92 + i * 114
const PLATFORM_Y = 360


const props = defineProps<{ t: number }>()
const T = computed(() => props.t)
const fly = (p: Piece) => ease((T.value - p.delay) / FLY_MS)
function centre(p: Piece): [number, number] {
  const k = fly(p)
  return [lerp(p.from[0], p.to[0] + 40, k), lerp(p.from[1], p.to[1] + 8, k)]
}
const grow = computed(() => ease((T.value - 600) / 1500))
const card = computed(() => {
  const k = grow.value
  return { x: lerp(ROOT_FROM[0] - 46, CARD.x, k), y: lerp(ROOT_FROM[1] - 15, CARD.y, k), w: lerp(92, CARD.w, k), h: lerp(30, CARD.h, k), r: lerp(15, 26, k) }
})
const rootCentre = computed((): [number, number] => [card.value.x + card.value.w / 2, card.value.y + card.value.h / 2])
const appear = computed(() => ease(T.value / 500))
const snapped = computed(() => clamp01((T.value - SNAP_AT) / 400))
const value = computed(() => 0.7 * ease((T.value - SNAP_AT) / 900))
const temp = computed(() => Math.round(lerp(58, 72, ease((T.value - SNAP_AT) / 900))))
const platformOn = (i: number) => clamp01((T.value - PLATFORM_AT - i * 220) / 300)
const linksIn = computed(() => clamp01((T.value - PLATFORM_AT + 200) / 500))
const curve = (a: [number, number], b: [number, number]) => {
  const my = (a[1] + b[1]) / 2
  return `M${a[0]},${a[1]} C${a[0]},${my} ${b[0]},${my} ${b[0]},${b[1]}`
}
const link = (p: Piece) => {
  const [x, y] = centre(p)
  const [rx, ry] = rootCentre.value
  return `M${rx},${ry} Q${(rx + x) / 2},${ry - 40} ${x},${y}`
}
</script>

<template>
  <svg viewBox="0 0 640 420" class="block w-full" role="img" aria-label="Config nodes assemble into a weather widget that renders on iOS, Android, macOS, Windows and Linux">
      <defs>
        <linearGradient id="la-card" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stop-color="#1e2a4a" />
          <stop offset="1" stop-color="#141a30" />
        </linearGradient>
        <filter id="la-glow" x="-40%" y="-40%" width="180%" height="180%">
          <feGaussianBlur stdDeviation="8" />
        </filter>
      </defs>

      <!-- Links to the platforms, with packets once the widget is built. -->
      <g :opacity="linksIn">
        <path
          v-for="(p, i) in platforms"
          :key="`l-${p.id}`"
          :d="curve([CARD.x + CARD.w / 2, CARD.y + CARD.h], [platformX(i), PLATFORM_Y - 18])"
          class="la-link"
          :class="{ on: platformOn(i) > 0.5 }"
        />
        <template v-if="T >= PLATFORM_AT">
          <circle v-for="(p, i) in platforms" :key="`d-${p.id}`" r="3" class="la-packet">
            <animateMotion
              :dur="`${1.4 + i * 0.12}s`"
              repeatCount="indefinite"
              :path="curve([CARD.x + CARD.w / 2, CARD.y + CARD.h], [platformX(i), PLATFORM_Y - 18])"
            />
          </circle>
        </template>
      </g>

      <!-- Tree links from the root node to every piece (they follow the pieces in flight). -->
      <g :opacity="appear * (1 - snapped)">
        <path v-for="p in pieces" :key="`t-${p.id}`" :d="link(p)" class="la-tree" />
      </g>

      <!-- Glow once assembled. -->
      <rect
        :x="CARD.x"
        :y="CARD.y"
        :width="CARD.w"
        :height="CARD.h"
        rx="26"
        class="la-glow"
        :opacity="snapped * 0.55"
        filter="url(#la-glow)"
      />

      <!-- The root `vstack` node growing into the widget card. -->
      <rect :x="card.x" :y="card.y" :width="card.w" :height="card.h" :rx="card.r" fill="url(#la-card)" class="la-card" :opacity="appear" />
      <text
        :x="rootCentre[0]"
        :y="rootCentre[1] + 4"
        text-anchor="middle"
        class="la-chip-text"
        :opacity="appear * (1 - grow * 2.5)"
      >vstack</text>

      <!-- Pieces: a node chip that cross-fades into its visual as it lands. -->
      <g v-for="p in pieces" :key="p.id" :opacity="appear">
        <g :opacity="1 - clamp01((fly(p) - 0.55) / 0.3)">
          <rect :x="centre(p)[0] - 42" :y="centre(p)[1] - 13" width="84" height="26" rx="13" class="la-chip" />
          <circle :cx="centre(p)[0] - 28" :cy="centre(p)[1]" r="4" class="la-chip-dot" :class="`k-${p.id}`" />
          <text :x="centre(p)[0] + 4" :y="centre(p)[1] + 4" text-anchor="middle" class="la-chip-text">{{ p.label }}</text>
        </g>
        <g :opacity="clamp01((fly(p) - 0.55) / 0.3)" :transform="`translate(${centre(p)[0] - p.to[0] - 40},${centre(p)[1] - p.to[1] - 8})`">
          <template v-if="p.id === 'image'">
            <circle :cx="p.to[0] + 9" :cy="p.to[1] + 9" r="8" fill="#facc15" />
            <circle :cx="p.to[0] + 9" :cy="p.to[1] + 9" r="12" fill="none" stroke="#facc1555" stroke-width="2" />
          </template>
          <text v-else-if="p.id === 'city'" :x="p.to[0]" :y="p.to[1] + 13" class="la-w-city">Moscow</text>
          <text v-else-if="p.id === 'temp'" :x="p.to[0]" :y="p.to[1] + 46" class="la-w-temp">{{ temp }}°</text>
          <text v-else-if="p.id === 'label'" :x="p.to[0]" :y="p.to[1] + 10" class="la-w-label">Humidity</text>
          <g v-else-if="p.id === 'bar'">
            <rect :x="p.to[0]" :y="p.to[1]" width="160" height="7" rx="3.5" fill="#4caf5033" />
            <rect :x="p.to[0]" :y="p.to[1]" :width="Math.max(7, 160 * value)" height="7" rx="3.5" fill="#4caf50" />
          </g>
          <g v-else-if="p.id === 'days'">
            <g v-for="(d, i) in ['Mon', 'Tue', 'Wed']" :key="d">
              <rect :x="p.to[0] + i * 56" :y="p.to[1]" width="48" height="26" rx="8" fill="#ffffff12" />
              <text :x="p.to[0] + i * 56 + 24" :y="p.to[1] + 17" text-anchor="middle" class="la-w-day">{{ d }}</text>
            </g>
          </g>
        </g>
      </g>

      <!-- Platforms. -->
      <g v-for="(p, i) in platforms" :key="`p-${p.id}`" :opacity="0.35 * linksIn + 0.65 * platformOn(i)">
        <rect :x="platformX(i) - 46" :y="PLATFORM_Y - 18" width="92" height="36" rx="10" class="la-platform" :class="{ on: platformOn(i) > 0.5 }" />
        <text :x="platformX(i)" :y="PLATFORM_Y + 5" text-anchor="middle" class="la-platform-text">{{ p.label }}</text>
      </g>
    </svg>
</template>

<style scoped>
.la-tree { fill: none; stroke: var(--ui-text-dimmed); stroke-width: 1.4; stroke-dasharray: 4 4; }
.la-link { fill: none; stroke: var(--ui-border-accented); stroke-width: 1.6; transition: stroke 0.3s; }
.la-link.on { stroke: var(--ui-primary); }
.la-packet { fill: var(--ui-primary); }
.la-chip { fill: var(--ui-bg-elevated); stroke: var(--ui-border-accented); }
.la-chip-text { fill: var(--ui-text-highlighted); font: 600 12px ui-monospace, SFMono-Regular, Menlo, monospace; }
.la-chip-dot { fill: var(--ui-primary); }
.la-chip-dot.k-image { fill: #facc15; }
.la-chip-dot.k-bar { fill: #4caf50; }
.la-chip-dot.k-days { fill: #60a5fa; }
.la-card { stroke: #ffffff1a; }
.la-glow { fill: var(--ui-primary); }
.la-w-city { fill: #e2e8f0; font: 600 13px ui-sans-serif, system-ui, sans-serif; }
.la-w-temp { fill: #fff; font: 700 44px ui-sans-serif, system-ui, sans-serif; }
.la-w-label { fill: #4caf50; font: 500 11px ui-sans-serif, system-ui, sans-serif; }
.la-w-day { fill: #cbd5e1; font: 500 11px ui-sans-serif, system-ui, sans-serif; }
.la-platform { fill: var(--ui-bg-elevated); stroke: var(--ui-border); transition: stroke 0.3s, fill 0.3s; }
.la-platform.on { stroke: var(--ui-primary); }
.la-platform-text { fill: var(--ui-text-highlighted); font: 600 12px ui-sans-serif, system-ui, sans-serif; }
</style>
