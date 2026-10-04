import { h, nextTick, watch } from "vue";
import type { Theme } from "vitepress";
import DefaultTheme from "vitepress/theme";
import { useData } from "vitepress";
import { createMermaidRenderer } from "vitepress-mermaid-renderer";
import "vitepress-mermaid-renderer/css";
import ShotGrid from "./components/ShotGrid.vue";
import Playground from "./components/Playground.vue";
import WidgetConstructor from "./components/WidgetConstructor.vue";
import "./custom.css";

export default {
  extends: DefaultTheme,
  Layout: () => {
    const { isDark } = useData();
    const apply = () => {
      createMermaidRenderer({
        startOnLoad: false,
        theme: isDark.value ? "dark" : "neutral",
      });
    };
    nextTick(apply);
    watch(isDark, apply);
    return h(DefaultTheme.Layout);
  },
  enhanceApp({ app }) {
    app.component("ShotGrid", ShotGrid);
    app.component("Playground", Playground);
    app.component("WidgetConstructor", WidgetConstructor);
  },
} satisfies Theme;
