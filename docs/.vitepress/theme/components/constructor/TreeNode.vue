<script setup lang="ts">
import { computed } from "vue";
import type { NodePath, WidgetNode } from "./model";
import { getChildren, isContainer } from "./model";

const props = defineProps<{
  node: WidgetNode;
  path: NodePath;
  selected: NodePath;
}>();

const emit = defineEmits<{
  select: [path: NodePath];
}>();

const kids = computed(() =>
  isContainer(props.node) ? getChildren(props.node) : [],
);

const isSelected = computed(
  () =>
    props.selected.length === props.path.length &&
    props.path.every((v, i) => v === props.selected[i]),
);
</script>

<template>
  <div class="wc__tn">
    <div
      class="wc__tn-row"
      :class="{ 'is-selected': isSelected }"
      @click.stop="emit('select', path)"
    >
      <span class="wc__tn-type">{{ node.type }}</span>
      <span v-if="kids.length" class="wc__tn-meta">{{ kids.length }}</span>
    </div>
    <div v-if="kids.length" class="wc__tn-kids">
      <TreeNode
        v-for="(child, i) in kids"
        :key="i"
        :node="child"
        :path="[...path, i]"
        :selected="selected"
        @select="emit('select', $event)"
      />
    </div>
  </div>
</template>
