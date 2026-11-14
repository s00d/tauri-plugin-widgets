<script setup lang="ts">
import { withBase } from 'ufo'

/** A recorded demo from `docs/public/media/<src>.mp4` (+ `<src>.jpg` poster), looped and muted. */
const props = defineProps<{ src: string; caption?: string }>()
const base = useRuntimeConfig().app.baseURL
const video = computed(() => withBase(`/media/${props.src}.mp4`, base))
const poster = computed(() => withBase(`/media/${props.src}.jpg`, base))

// Reduced motion: no autoplay, the poster and controls instead.
const reduced = ref(false)
onMounted(() => {
  reduced.value = window.matchMedia('(prefers-reduced-motion: reduce)').matches
})
</script>

<template>
  <figure class="demo-video">
    <video
      :key="`${video}-${reduced}`"
      :src="video"
      :poster="poster"
      :autoplay="!reduced"
      :controls="reduced"
      loop
      muted
      playsinline
      preload="metadata"
    />
    <figcaption v-if="caption">{{ caption }}</figcaption>
  </figure>
</template>
