---
layout: home

hero:
  name: tauri-plugin-widgets
  text: One JSON config. Five native surfaces.
  tagline: Android Glance, Apple WidgetKit, Windows Adaptive Cards, and desktop webviews — from a single declarative IR.
  image:
    src: /hero.jpg
    alt: Cross-platform widget panels from one JSON config
  actions:
    - theme: brand
      text: Get started
      link: /guide/first-widget
    - theme: alt
      text: Showcase
      link: /showcase
    - theme: alt
      text: GitHub
      link: https://github.com/s00d/tauri-plugin-widgets

features:
  - title: Declarative IR
    details: Author once in JSON / TypeScript. Rust models are the source of truth; native renderers map the same tree.
  - title: Tested screenshots
    details: Showcase images are the committed goldens — if a render regresses, the snapshot test and the docs image fail together.
  - title: Live playground
    details: The same desktop widget.html renderer runs in the docs sandbox with a stubbed invoke bridge.
---
