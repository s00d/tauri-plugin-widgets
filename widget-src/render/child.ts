import type { ElNode, ParentAxis } from "../types";

/**
 * Child rendering for container elements (stacks, lists, buttons with content).
 * `element.ts` registers its dispatcher here, so the per-type renderers do not import
 * `element.ts` back (that was an import cycle).
 */
type Render = (d: ElNode, parentAxis?: ParentAxis) => HTMLElement | SVGElement;

let impl: Render | null = null;

export function setRenderEl(f: Render) {
  impl = f;
}

export function renderEl(d: ElNode, parentAxis?: ParentAxis): HTMLElement | SVGElement {
  if (!impl) throw new Error("renderEl used before render/element.ts was loaded");
  return impl(d, parentAxis);
}
