import {
  renderStack,
  renderZStack,
  renderGrid,
  renderContainer,
} from "./layout";
import { renderText, renderDate, renderTimer, renderLabel } from "./text";
import { renderImage, renderShape, renderCanvas } from "./media";
import { renderProgress, renderGauge, renderChart, renderList } from "./data";
import { renderButton, renderToggle, renderLink } from "./interactive";
import { renderDivider, renderSpacer } from "./spacing";
import type { ElNode, ParentAxis } from "../types";

export function renderEl(d: ElNode, parentAxis?: ParentAxis): HTMLElement | SVGElement {
  switch (d.type) {
    case "vstack":
      return renderStack(d, "column");
    case "hstack":
      return renderStack(d, "row");
    case "zstack":
      return renderZStack(d);
    case "grid":
      return renderGrid(d);
    case "container":
      return renderContainer(d);
    case "text":
      return renderText(d, parentAxis);
    case "image":
      return renderImage(d);
    case "progress":
      return renderProgress(d);
    case "gauge":
      return renderGauge(d);
    case "button":
      return renderButton(d);
    case "toggle":
      return renderToggle(d);
    case "divider":
      return renderDivider(d, parentAxis);
    case "spacer":
      return renderSpacer(d);
    case "date":
      return renderDate(d);
    case "chart":
      return renderChart(d);
    case "list":
      return renderList(d);
    case "link":
      return renderLink(d);
    case "shape":
      return renderShape(d);
    case "timer":
      return renderTimer(d);
    case "label":
      return renderLabel(d);
    case "canvas":
      return renderCanvas(d);
    default:
      return document.createElement("span");
  }
}
