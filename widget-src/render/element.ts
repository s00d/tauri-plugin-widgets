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
import { setRenderEl } from "./child";

export function renderEl(d: ElNode, parentAxis?: ParentAxis): HTMLElement | SVGElement {
  const el = build(d, parentAxis);
  // `transition` hint: `renderWidget` animates marked elements whose content changed.
  if (typeof d.transition === "string" && d.transition !== "none") el.setAttribute("data-transition", d.transition);
  // Editor marker (docs widget constructor): maps a drawn element back to its tree node.
  if (d.__node != null) el.setAttribute("data-node", String(d.__node));
  return el;
}

setRenderEl(renderEl);

function build(d: ElNode, parentAxis?: ParentAxis): HTMLElement | SVGElement {
  switch (d.type) {
    case "vstack":
      return renderStack(d, "column", parentAxis);
    case "hstack":
      return renderStack(d, "row", parentAxis);
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
