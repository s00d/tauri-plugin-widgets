<script setup lang="ts">
/** Drawn pointer for the hero scenes; `down` shrinks it and shows a tap ripple. */
defineProps<{ x: number; y: number; down?: boolean; ripple?: number }>()
</script>

<template>
  <g>
    <circle v-if="ripple && ripple > 0 && ripple < 1" :cx="x" :cy="y" :r="6 + ripple * 22" fill="none" stroke="var(--ui-primary)" :stroke-opacity="1 - ripple" stroke-width="2" />
    <path
      :transform="`translate(${x},${y}) scale(${down ? 0.82 : 1})`"
      d="M0 0 L0 17 L4.5 13 L7.5 20 L10.5 18.6 L7.6 11.8 L13 11.8 Z"
      fill="#fff"
      stroke="#111"
      stroke-width="1.2"
      stroke-linejoin="round"
    />
  </g>
</template>
