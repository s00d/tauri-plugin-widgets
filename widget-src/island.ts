// Desktop live-activity island (`activitySurface: "island"`): the page of the
// `activity-_island` window. The host (src/desktop/island.rs) sizes the window to the
// visible shapes and sends the view; this draws the pill and animates expand / collapse.
import type { ElNode, WidgetConfig } from "./types";
import { renderWidget } from "./lib";

type Pair = [number, number];
type Regions = {
  compactLeading?: ElNode | null;
  compactTrailing?: ElNode | null;
  minimal?: ElNode | null;
  leading?: ElNode | null;
  trailing?: ElNode | null;
  center?: ElNode | null;
  bottom: ElNode;
};
type IslandActivity = {
  id: string;
  group: string;
  widgetId: string;
  url?: string | null;
  keyline?: string | null;
  regions: Regions;
};
/** `activity-island` event payload / `activity_island` result (Rust `island::View`). */
export type IslandView = {
  expanded: boolean;
  flush: boolean;
  gap: number;
  band: number;
  compact: Pair;
  expandedSize: Pair;
  minimal: number[];
  activities: IslandActivity[];
};
export type IslandHost = {
  /** Ask the host to grow / shrink the window (`activity_island`); resolves with the new view. */
  setExpanded(expanded: boolean): Promise<IslandView | null>;
  focus(id: string): void;
};

/** Expand / collapse duration (keep in sync with the CSS below). */
const MOVE_MS = 320;
const EASE = "cubic-bezier(.2,.9,.25,1.06)";
const CSS = `
.isl{position:absolute;top:0;left:50%;transform:translateX(-50%);background:#000;color:#fff;overflow:hidden;
cursor:default;transition:width ${MOVE_MS}ms ${EASE},height ${MOVE_MS}ms ${EASE},border-radius ${MOVE_MS}ms ${EASE}}
.isl .layer{position:absolute;top:0;transition:opacity 160ms ease-out}
.isl .x{opacity:0;pointer-events:none}
.isl.open .c{opacity:0;pointer-events:none}
.isl.open .x{opacity:1;pointer-events:auto;transition:opacity 220ms ease-in 90ms}
.isl .slot{position:absolute;overflow:hidden}
.isl .band{padding:0 12px}
.mini{position:absolute;top:0;background:#000;border-radius:50%;overflow:hidden;cursor:pointer}
`;

const SPACER: ElNode = { type: "spacer" };

/** Align a region inside its slot: leading, trailing or centred, vertically centred. */
function align(node: ElNode | null | undefined, how: "lead" | "trail" | "center"): ElNode | null {
  if (!node) return null;
  const children = how === "lead" ? [node, SPACER] : how === "trail" ? [SPACER, node] : [SPACER, node, SPACER];
  // Fills the slot, so the region is centred vertically (a root row that does not fill
  // height is pinned to the top, like WidgetKit).
  return {
    type: "hstack",
    spacing: 0,
    alignment: "center",
    frame: { maxWidth: "infinity", maxHeight: "infinity" },
    children,
  } as ElNode;
}

function draw(host: HTMLElement, act: IslandActivity, node: ElNode | null): void {
  if (!node) {
    host.replaceChildren();
    return;
  }
  renderWidget(host, { medium: node } as WidgetConfig, {
    size: "medium",
    theme: "dark",
    group: act.group,
    // Taps report `widgetId = "activity.<id>"` like the other activity surfaces.
    widgetId: act.widgetId,
    surface: "transparent",
  });
}

function box(el: HTMLElement, x: number, y: number, w: number, h: number): void {
  el.style.left = x + "px";
  el.style.top = y + "px";
  el.style.width = Math.max(0, w) + "px";
  el.style.height = Math.max(0, h) + "px";
}

function div(cls: string, parent: HTMLElement): HTMLElement {
  const d = document.createElement("div");
  d.className = cls;
  parent.appendChild(d);
  return d;
}

/** Mount the island into `root`. Returns `update(view)` for each new view from the host. */
export function mountIsland(root: HTMLElement, host: IslandHost): (view: IslandView | null) => void {
  if (!document.getElementById("island-css")) {
    const st = document.createElement("style");
    st.id = "island-css";
    st.textContent = CSS;
    document.head.appendChild(st);
  }
  root.replaceChildren();
  const pill = div("isl", root);
  const compact = div("layer c", pill);
  const big = div("layer x", pill);
  const slots = {
    cl: div("slot band", compact),
    ct: div("slot band", compact),
    xl: div("slot band", big),
    xt: div("slot band", big),
    xc: div("slot", big),
    xb: div("slot", big),
  };
  const minis = div("minis", root);
  let view: IslandView | null = null;
  let open = false;
  /** Last hover / click intent (a slow host reply must not reopen a left island). */
  let want = false;
  let closeTimer: ReturnType<typeof setTimeout> | null = null;

  const paint = () => {
    const a = view && view.activities[0];
    root.style.display = a ? "" : "none";
    if (!view || !a) return;
    const { band, gap, flush } = view;
    const [cw, ch] = view.compact;
    const [ew, eh] = view.expandedSize;
    pill.classList.toggle("open", open);
    pill.style.width = (open ? ew : cw) + "px";
    pill.style.height = (open ? eh : ch) + "px";
    const r = open ? 22 : flush ? Math.round(band * 0.35) : band / 2;
    // At the notch the pill hangs from the screen edge: square top corners.
    pill.style.borderRadius = flush ? `0 0 ${r}px ${r}px` : r + "px";
    pill.style.boxShadow = a.keyline ? "inset 0 0 0 1px " + a.keyline : "";
    // Layers stay centred while the pill grows / shrinks around them.
    box(compact, 0, 0, cw, ch);
    box(big, 0, 0, ew, eh);
    compact.style.left = `calc(50% - ${cw / 2}px)`;
    big.style.left = `calc(50% - ${ew / 2}px)`;
    const side = (cw - gap) / 2;
    box(slots.cl, 0, 0, side, band);
    box(slots.ct, cw - side, 0, side, band);
    const xside = (ew - gap) / 2;
    const centerH = a.regions.center ? 30 : 0;
    box(slots.xl, 0, 0, xside, band);
    box(slots.xt, ew - xside, 0, xside, band);
    box(slots.xc, 0, band, ew, centerH);
    box(slots.xb, 0, band + centerH, ew, eh - band - centerH);
    const g = a.regions;
    draw(slots.cl, a, align(g.compactLeading, "lead"));
    draw(slots.ct, a, align(g.compactTrailing, "trail"));
    draw(slots.xl, a, align(g.leading, "lead"));
    draw(slots.xt, a, align(g.trailing, "trail"));
    draw(slots.xc, a, align(g.center, "center"));
    draw(slots.xb, a, g.bottom);
    minis.replaceChildren();
    if (open) return;
    view.minimal.forEach((dx, i) => {
      const other = view!.activities[i + 1];
      if (!other) return;
      const m = div("mini", minis);
      m.style.left = `calc(50% + ${dx - band / 2}px)`;
      m.style.width = m.style.height = band + "px";
      if (other.keyline) m.style.boxShadow = "inset 0 0 0 1px " + other.keyline;
      draw(m, other, align(other.regions.minimal, "center"));
      m.onclick = () => host.focus(other.id);
    });
  };

  const setOpen = (next: boolean) => {
    if (closeTimer) clearTimeout(closeTimer);
    closeTimer = null;
    want = next;
    if (next === open) return;
    if (next) {
      // Grow the window first, then animate the pill into the new room.
      host.setExpanded(true).then((v) => {
        if (v) view = v;
        if (!want) {
          host.setExpanded(false);
          return;
        }
        open = true;
        paint();
      });
    } else {
      // Shrink the pill inside the big window, then give the room back.
      open = false;
      paint();
      closeTimer = setTimeout(() => {
        if (!open) host.setExpanded(false);
      }, MOVE_MS);
    }
  };

  pill.addEventListener("mouseenter", () => setOpen(true));
  document.documentElement.addEventListener("mouseleave", () => {
    if (closeTimer) clearTimeout(closeTimer);
    closeTimer = setTimeout(() => setOpen(false), 150);
  });
  // No hover (touch, some Wayland setups): a click opens, a click on the top band closes.
  pill.addEventListener("click", (e) => {
    const t = e.target as Node;
    if (!open) setOpen(true);
    else if (t === pill || t === big || slots.xl.contains(t) || slots.xt.contains(t)) setOpen(false);
  });

  return (next) => {
    view = next;
    // The host's cursor poller (macOS: no hover events in an inactive app) drives too:
    // it grows the window and says `expanded`, or asks to close by sending it `false`.
    if (next && next.expanded && !open && !closeTimer) {
      want = true;
      open = true;
    } else if (next && !next.expanded && open) {
      setOpen(false);
      return;
    }
    paint();
  };
}
