import { defineConfig } from "vitepress";

const docsBase = process.env.NODE_ENV === "production" ? "/tauri-plugin-widgets/" : "/";

const sidebar = [
  {
    text: "Guide",
    collapsed: false,
    items: [
      { text: "Overview", link: "/guide/" },
      { text: "Install", link: "/guide/install" },
      { text: "First widget", link: "/guide/first-widget" },
      { text: "Concepts", link: "/guide/concepts" },
      { text: "Transport", link: "/guide/transport" },
      { text: "Core vs extended", link: "/guide/tiers" },
      { text: "Troubleshooting", link: "/guide/troubleshooting" },
    ],
  },
  {
    text: "Platform setup",
    collapsed: true,
    items: [
      { text: "Choose a platform", link: "/guide/setup/" },
      { text: "Android", link: "/guide/setup/android" },
      { text: "iOS", link: "/guide/setup/ios" },
      { text: "macOS", link: "/guide/setup/macos" },
      { text: "Windows", link: "/guide/setup/windows" },
      { text: "Linux", link: "/guide/setup/linux" },
      { text: "Desktop webview", link: "/guide/setup/desktop" },
    ],
  },
  {
    text: "Recipes",
    collapsed: true,
    items: [
      { text: "Updating data", link: "/guide/recipes/updating-data" },
      { text: "Actions", link: "/guide/recipes/actions" },
      { text: "Images", link: "/guide/recipes/images" },
      { text: "Multi-widget", link: "/guide/recipes/multi-widget" },
      { text: "Testing", link: "/guide/recipes/testing" },
    ],
  },
  {
    text: "Elements",
    collapsed: true,
    items: [
      { text: "Index", link: "/elements/" },
      { text: "Layout", link: "/elements/layout" },
      { text: "Text", link: "/elements/text" },
      { text: "Media", link: "/elements/media" },
      { text: "Data", link: "/elements/data" },
      { text: "Interactive", link: "/elements/interactive" },
      { text: "Spacing", link: "/elements/spacing" },
      { text: "Style", link: "/elements/style" },
      { text: "Colors", link: "/elements/colors" },
    ],
  },
  {
    text: "API",
    collapsed: true,
    items: [
      { text: "JavaScript", link: "/api/js" },
      { text: "Rust", link: "/api/rust" },
      { text: "Plugin config", link: "/api/plugin-config" },
      { text: "Cargo features", link: "/api/cargo-features" },
      { text: "Environment", link: "/api/env" },
      { text: "Permissions", link: "/api/permissions" },
      { text: "JSON Schema", link: "/api/json-schema" },
    ],
  },
];

export default defineConfig({
  lang: "en-US",
  title: "tauri-plugin-widgets",
  description:
    "Native widgets for Android, iOS, macOS, Windows, and Linux from one declarative JSON config.",
  titleTemplate: ":title | tauri-plugin-widgets",
  lastUpdated: true,
  cleanUrls: true,
  base: docsBase,

  ignoreDeadLinks: [
    /\/src\//,
    /\/tests\//,
    /\/examples\//,
    /README/,
    /interfaces\//,
  ],

  head: [
    ["meta", { name: "theme-color", content: "#0f172a" }],
    ["link", { rel: "icon", type: "image/png", sizes: "32x32", href: `${docsBase}favicon-32x32.png` }],
    ["link", { rel: "icon", type: "image/png", sizes: "16x16", href: `${docsBase}favicon-16x16.png` }],
    ["link", { rel: "apple-touch-icon", sizes: "180x180", href: `${docsBase}apple-touch-icon.png` }],
    ["meta", { property: "og:type", content: "website" }],
    ["meta", { property: "og:title", content: "tauri-plugin-widgets" }],
    [
      "meta",
      {
        property: "og:description",
        content:
          "Native widgets for Android, iOS, macOS, Windows, and Linux from one declarative JSON config.",
      },
    ],
    ["meta", { property: "og:image", content: "https://s00d.github.io/tauri-plugin-widgets/og.jpg" }],
    ["meta", { name: "twitter:card", content: "summary_large_image" }],
    ["meta", { name: "twitter:image", content: "https://s00d.github.io/tauri-plugin-widgets/og.jpg" }],
  ],

  themeConfig: {
    logo: "/logo.png",
    search: { provider: "local" },
    socialLinks: [
      { icon: "github", link: "https://github.com/s00d/tauri-plugin-widgets" },
    ],
    nav: [
      { text: "Guide", link: "/guide/" },
      { text: "Elements", link: "/elements/" },
      { text: "API", link: "/api/js" },
      { text: "Showcase", link: "/showcase" },
    ],
    sidebar,
    editLink: {
      pattern: "https://github.com/s00d/tauri-plugin-widgets/edit/main/docs/:path",
      text: "Edit this page on GitHub",
    },
    footer: {
      message: `Released under the MIT License. · <a href="${docsBase}contributing/">Contributing</a>`,
      copyright: "Copyright © s00d",
    },
  },
});
