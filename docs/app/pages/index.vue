<script setup lang="ts">
import type { ButtonProps, PageFeatureProps } from '@nuxt/ui'

const { data: page } = await useAsyncData('landing', () => queryCollection('landing').path('/').first())

useSeoMeta({
  title: 'tauri-plugin-widgets',
  description: page.value?.description,
  ogTitle: 'tauri-plugin-widgets',
  ogDescription: page.value?.description,
})

const heroLinks: ButtonProps[] = [
  { label: 'Get started', to: '/guide/first-widget', size: 'lg', trailingIcon: 'i-lucide-arrow-right' },
  { label: 'GitHub', to: 'https://github.com/s00d/tauri-plugin-widgets', target: '_blank', size: 'lg', color: 'neutral', variant: 'outline', icon: 'i-simple-icons-github' },
]

const features: PageFeatureProps[] = [
  { title: 'Five native surfaces', description: 'Android Glance, iOS and macOS WidgetKit, Windows Widgets Board (Adaptive Cards) and desktop webviews, from one JSON tree.', icon: 'i-lucide-monitor-smartphone', to: '/guide/setup' },
  { title: 'Dynamic Island everywhere', description: 'Live activities as an island on iPhone and as a pill around the notch on macOS, Windows and Linux.', icon: 'i-lucide-pill', to: '/features/dynamic-island' },
  { title: 'Interactive widgets', description: 'Buttons and toggles that reach your app, or change the widget by themselves with local state.', icon: 'i-lucide-pointer', to: '/features/interactive' },
  { title: 'Timelines and live data', description: 'Prepared layouts for later times, values from the app or fetched by the widget itself.', icon: 'i-lucide-calendar-clock', to: '/features/timelines' },
  { title: 'Tested screenshots', description: 'Showcase images are the committed goldens: if a render regresses, the snapshot test and the docs image fail together.', icon: 'i-lucide-images', to: '/showcase' },
  { title: 'Widget constructor', description: 'Assemble a widget from blocks with a live preview and export the JSON.', icon: 'i-lucide-blocks', to: '/constructor' },
]
</script>

<template>
  <div>
    <UPageHero
      title="One JSON config. Five native surfaces."
      description="Android Glance, Apple WidgetKit, Windows Adaptive Cards, and desktop webviews, from a single declarative IR."
      orientation="horizontal"
      :links="heroLinks"
      :ui="{ container: 'lg:py-24' }"
    >
      <template #headline>
        <UButton to="/features/dynamic-island" label="New: Dynamic Island on desktop" trailing-icon="i-lucide-arrow-right" color="neutral" variant="subtle" size="sm" />
      </template>
      <LandingAssemble />
    </UPageHero>

    <UPageSection title="What you get" :features="features" :links="[{ label: 'All features', to: '/features', trailingIcon: 'i-lucide-arrow-right', color: 'neutral', variant: 'outline' }]" />

    <UPageSection
      title="Push a config, get a widget"
      description="setWidgetConfig stores the tree in the shared widget store; every surface renders it natively. On desktop, createWidgetWindow opens a frameless webview with the built-in renderer."
      orientation="horizontal"
      :links="[{ label: 'First widget', to: '/guide/first-widget', trailingIcon: 'i-lucide-arrow-right' }, { label: 'Install', to: '/guide/install', color: 'neutral', variant: 'outline' }]"
    >
      <div class="min-w-0">
        <ContentRenderer v-if="page" :value="page" />
      </div>
    </UPageSection>
  </div>
</template>
