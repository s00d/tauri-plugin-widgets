<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { withBase } from "vitepress";

const props = defineProps<{
  case?: string;
  /** Raw JSON string */
  config?: string;
  /** Base64-encoded JSON (preferred for gallery pages — no fetch) */
  configB64?: string;
  size?: string;
}>();

const editable = ref("");
const size = computed(() => props.size ?? "small");
const frameSize = computed(() => {
  const s = size.value;
  if (s === "large") return { width: "360px", height: "360px" };
  if (s === "medium") return { width: "360px", height: "200px" };
  return { width: "180px", height: "180px" };
});

const iframeSrc = computed(() =>
  withBase(`/widget-sandbox.html?size=${encodeURIComponent(size.value)}&sandbox=1`),
);
const iframeEl = ref<HTMLIFrameElement | null>(null);

function decodeConfig(): string | null {
  if (props.configB64) {
    try {
      const json = atob(props.configB64);
      JSON.parse(json);
      return json;
    } catch {
      /* fall through */
    }
  }
  if (props.config) {
    try {
      JSON.parse(props.config);
      return props.config;
    } catch {
      /* fall through */
    }
  }
  return null;
}

function push() {
  const w = iframeEl.value?.contentWindow;
  if (!w) return;
  try {
    const config = JSON.parse(editable.value);
    w.postMessage({ type: "render", config }, "*");
  } catch {
    /* ignore invalid JSON while typing */
  }
}

function onLoad() {
  push();
}

async function loadCaseConfig() {
  const embedded = decodeConfig();
  if (embedded) {
    try {
      editable.value = JSON.stringify(JSON.parse(embedded), null, 2);
      return;
    } catch {
      editable.value = embedded;
      return;
    }
  }
  if (!props.case) {
    editable.value = JSON.stringify(
      {
        small: {
          type: "text",
          content: "Edit me",
          color: "#fff",
          fontSize: 18,
        },
      },
      null,
      2,
    );
    return;
  }
  try {
    const res = await fetch(withBase(`/gallery-data/${props.case}.json`));
    if (res.ok) {
      editable.value = JSON.stringify(await res.json(), null, 2);
      return;
    }
  } catch {
    /* fall through */
  }
  editable.value = JSON.stringify(
    {
      small: {
        type: "vstack",
        padding: 12,
        background: "#1a1a2e",
        children: [{ type: "text", content: props.case, color: "#fff", fontSize: 16 }],
      },
    },
    null,
    2,
  );
}

onMounted(async () => {
  await loadCaseConfig();
  push();
});
watch(
  () => [props.case, props.config, props.configB64],
  async () => {
    await loadCaseConfig();
    push();
  },
);
</script>

<template>
  <div class="playground">
    <div class="playground__frame" :style="frameSize">
      <iframe
        ref="iframeEl"
        :src="iframeSrc"
        :title="`playground-${props.case || 'custom'}`"
        @load="onLoad"
      />
    </div>
    <textarea v-model="editable" class="playground__editor" spellcheck="false" @input="push" />
  </div>
</template>
