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
  { title: 'Declarative IR', description: 'Author once in JSON / TypeScript. Rust models are the source of truth; native renderers map the same tree.', icon: 'i-lucide-braces', to: '/guide/concepts' },
  { title: 'Five native surfaces', description: 'Android Glance, iOS and macOS WidgetKit, Windows Widgets Board (Adaptive Cards) and desktop webviews.', icon: 'i-lucide-monitor-smartphone', to: '/guide/setup' },
  { title: 'Tested screenshots', description: 'Showcase images are the committed goldens: if a render regresses, the snapshot test and the docs image fail together.', icon: 'i-lucide-images', to: '/showcase' },
  { title: 'Live playground', description: 'The same desktop widget.html renderer runs in the docs, with an editable config next to every preset.', icon: 'i-lucide-play', to: '/showcase' },
  { title: 'Widget constructor', description: 'Build a widget tree in the browser with a schema-driven inspector and export the JSON.', icon: 'i-lucide-blocks', to: '/guide/constructor' },
  { title: 'Generated reference', description: 'Elements, JS functions, Tauri commands and permissions, built from the sources on every release.', icon: 'i-lucide-book-open', to: '/api/reference' },
]
</script>

<template>
  <div>
    <UPageHero
      title="One JSON config. Five native surfaces."
      description="Android Glance, Apple WidgetKit, Windows Adaptive Cards, and desktop webviews, from a single declarative IR."
      :links="heroLinks"
    >
      <template #headline>
        <UButton to="/changelog" label="0.6.2: new documentation site" trailing-icon="i-lucide-arrow-right" color="neutral" variant="subtle" size="sm" />
      </template>
    </UPageHero>

    <UPageSection title="What you get" :features="features" />

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
