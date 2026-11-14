<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { withBase } from "ufo";

const base = useRuntimeConfig().app.baseURL;

const props = defineProps<{ case: string }>();

const platforms = ["desktop", "ios", "macos", "android", "linux", "windows"] as const;
const active = ref<(typeof platforms)[number]>("desktop");
const missing = ref(false);

const src = computed(() => withBase(`/shots/${active.value}/${props.case}.png`, base));
// Goldens differ in pixel density (desktop / linux 1x, iOS / macOS 2x, Android 3x): show every
// platform at the widget's logical width so tabs compare like for like and dense ones stay sharp.
const width = computed(() => (/\.small$/.test(props.case) ? 170 : 360));

watch(src, () => {
  missing.value = false;
});

function onError() {
  missing.value = true;
}

function onLoad() {
  missing.value = false;
}
</script>

<template>
  <div class="shot-grid">
    <div class="shot-grid__tabs">
      <button
        v-for="p in platforms"
        :key="p"
        type="button"
        class="shot-grid__tab"
        :class="{ 'is-active': active === p }"
        @click="active = p"
      >
        {{ p }}
      </button>
    </div>
    <div class="shot-grid__frame">
      <img
        v-show="!missing"
        :key="src"
        :src="src"
        :alt="`${props.case} on ${active}`"
        :style="{ width: `${width}px`, maxWidth: '100%', height: 'auto' }"
        @error="onError"
        @load="onLoad"
      />
      <p v-if="missing" class="shot-missing" style="opacity: 0.7; margin: 0">
        Not recorded yet for <code>{{ active }}</code>.
      </p>
    </div>
  </div>
</template>
