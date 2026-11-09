import { fileURLToPath } from 'node:url'

// GitHub Pages serves the site from https://s00d.github.io/tauri-plugin-widgets/.
const baseURL = process.env.NUXT_APP_BASE_URL || (process.env.NODE_ENV === 'production' ? '/tauri-plugin-widgets/' : '/')

export default defineNuxtConfig({
  modules: ['@nuxt/ui', '@nuxt/content'],

  css: ['~/assets/css/main.css'],

  // Playground and constructor render with the same code as the desktop widget.html.
  alias: {
    'tauri-plugin-widgets-api/render': fileURLToPath(new URL('../widget-src/lib.ts', import.meta.url)),
  },

  app: {
    baseURL,
    head: {
      htmlAttrs: { lang: 'en' },
      meta: [{ name: 'theme-color', content: '#18181b' }],
      link: [{ rel: 'icon', type: 'image/svg+xml', href: `${baseURL}favicon.svg` }],
    },
  },

  icon: {
    // Bundle every icon so the static site never asks the Iconify API at runtime.
    clientBundle: {
      scan: true,
      icons: ['simple-icons:rust', 'simple-icons:toml', 'simple-icons:typescript', 'simple-icons:json', 'simple-icons:npm', 'lucide:terminal',
        // .navigation.yml icons are not found by the scanner.
        'lucide:rocket', 'lucide:monitor-smartphone', 'lucide:chef-hat', 'lucide:blocks', 'lucide:file-code', 'lucide:images', 'lucide:git-pull-request'],
      sizeLimitKb: 512,
    },
  },

  colorMode: {
    preference: 'system',
  },

  content: {
    build: {
      markdown: {
        toc: { depth: 3, searchDepth: 2 },
        highlight: {
          langs: ['rust', 'toml', 'bash', 'shell', 'ts', 'js', 'json', 'jsonc', 'yaml', 'typescript', 'xml', 'swift', 'powershell'],
        },
      },
    },
    experimental: {
      // node:sqlite (Node 22.5+): no native better-sqlite3 build.
      sqliteConnector: 'native',
    },
  },

  nitro: {
    prerender: {
      // `/guide/install.html`, not `/guide/install/index.html`: GitHub Pages serves it
      // without a redirect to the trailing-slash URL.
      autoSubfolderIndex: false,
      crawlLinks: true,
      failOnError: true,
      routes: ['/'],
    },
  },

  compatibilityDate: '2025-11-03',
})
