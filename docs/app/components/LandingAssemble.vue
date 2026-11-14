<script setup lang="ts">
/**
 * Landing hero: short animated scenes in rotation (assemble, Dynamic Island, taps, timeline,
 * live data). Each scene is a pure function of its clock `t`; one rAF loop drives it.
 * Dots switch scenes; reduced motion shows each scene's final frame and no autoplay.
 */
import SceneAssemble from './hero/SceneAssemble.vue'
import SceneIsland from './hero/SceneIsland.vue'
import SceneTap from './hero/SceneTap.vue'
import SceneTimeline from './hero/SceneTimeline.vue'
import SceneData from './hero/SceneData.vue'

const scenes = [
  { comp: SceneAssemble, ms: 4600, title: 'One config, every surface', sub: 'Nodes of a WidgetConfig become a native widget', to: '/features', link: 'Features' },
  { comp: SceneIsland, ms: 5400, title: 'Dynamic Island on desktop', sub: 'A live activity around the notch, expanded on hover', to: '/features/dynamic-island', link: 'Dynamic Island' },
  { comp: SceneTap, ms: 5300, title: 'Interactive without the app', sub: 'Taps change local state right in the widget', to: '/features/interactive', link: 'Interactive' },
  { comp: SceneTimeline, ms: 4800, title: 'Scheduled entries', sub: 'The widget changes on its own times, the app asleep', to: '/features/timelines', link: 'Timelines' },
  { comp: SceneData, ms: 5200, title: 'Live data', sub: 'Push values, keep the layout', to: '/features/remote-data', link: 'Remote data' },
]
const HOLD = 1400

const idx = ref(0)
const t = ref(0)
const reduced = ref(false)
const paused = ref(false)
let raf = 0
let start = 0

const scene = computed(() => scenes[idx.value]!)

function frame(now: number) {
  t.value = now - start
  if (!paused.value && t.value >= scene.value.ms + HOLD) return go((idx.value + 1) % scenes.length)
  raf = requestAnimationFrame(frame)
}
function go(i: number) {
  cancelAnimationFrame(raf)
  idx.value = i
  if (reduced.value) {
    t.value = scenes[i]!.ms
    return
  }
  start = performance.now()
  t.value = 0
  raf = requestAnimationFrame(frame)
}

onMounted(() => {
  reduced.value = window.matchMedia('(prefers-reduced-motion: reduce)').matches
  go(0)
})
onBeforeUnmount(() => cancelAnimationFrame(raf))
</script>

<template>
  <figure
    class="landing-assemble relative w-full overflow-hidden rounded-xl border border-default bg-default shadow-sm"
    @mouseenter="paused = true"
    @mouseleave="paused = false"
  >
    <div class="flex min-h-14 items-center justify-between gap-3 border-b border-default px-4 py-2.5">
      <div class="min-w-0">
        <p class="truncate text-sm font-semibold text-highlighted">{{ scene.title }}</p>
        <p class="truncate text-xs text-muted">{{ scene.sub }}</p>
      </div>
      <UButton icon="i-lucide-rotate-ccw" color="neutral" variant="ghost" size="xs" aria-label="Replay" @click="go(idx)" />
    </div>

    <component :is="scene.comp" :t="t" />

    <figcaption class="flex items-center justify-between gap-4 border-t border-default bg-elevated/40 px-4 py-2.5 text-xs text-muted">
      <div class="flex items-center gap-1.5" role="tablist" aria-label="Hero scenes">
        <button
          v-for="(s, i) in scenes"
          :key="i"
          type="button"
          role="tab"
          :aria-selected="i === idx"
          :aria-label="s.title"
          :title="s.title"
          class="la-dot"
          :class="{ on: i === idx }"
          @click="go(i)"
        >
          <span v-if="i === idx && !reduced" class="la-dot-fill" :style="{ width: `${Math.min(100, (t / (s.ms + HOLD)) * 100)}%` }" />
        </button>
      </div>
      <NuxtLink :to="scene.to" class="inline-flex items-center gap-1 rounded-sm font-medium text-primary hover:underline">
        {{ scene.link }}
        <UIcon name="i-lucide-arrow-right" class="size-3.5" />
      </NuxtLink>
    </figcaption>
  </figure>
</template>

<style scoped>
.la-dot { position: relative; width: 8px; height: 8px; border-radius: 999px; background: var(--ui-border-accented); overflow: hidden; transition: width 0.25s; cursor: pointer; }
.la-dot.on { width: 28px; }
.la-dot-fill { position: absolute; inset: 0 auto 0 0; background: var(--ui-primary); }
.la-dot.on:not(:has(.la-dot-fill)) { background: var(--ui-primary); }
</style>
