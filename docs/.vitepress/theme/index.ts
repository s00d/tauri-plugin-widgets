import DefaultTheme from "vitepress/theme";
import ShotGrid from "./components/ShotGrid.vue";
import Playground from "./components/Playground.vue";
import "./custom.css";

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component("ShotGrid", ShotGrid);
    app.component("Playground", Playground);
  },
};
