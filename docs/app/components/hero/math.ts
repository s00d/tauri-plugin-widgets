/** Hero scene helpers: every scene is a pure function of its own clock `t` (ms). */
export const clamp01 = (x: number) => Math.min(1, Math.max(0, x))
export const ease = (x: number) => 1 - (1 - clamp01(x)) ** 3
export const easeInOut = (x: number) => {
  const k = clamp01(x)
  return k < 0.5 ? 4 * k * k * k : 1 - (-2 * k + 2) ** 3 / 2
}
export const lerp = (a: number, b: number, k: number) => a + (b - a) * k
/** 0 before `a`, 1 after `b`, eased in between. */
export const span = (t: number, a: number, b: number) => ease((t - a) / (b - a))
/** Position along waypoints [{at, x, y}] (cursor paths). */
export function path(t: number, pts: { at: number; x: number; y: number }[]) {
  if (t <= pts[0]!.at) return pts[0]!
  for (let i = 1; i < pts.length; i++) {
    const a = pts[i - 1]!
    const b = pts[i]!
    if (t <= b.at) {
      const k = easeInOut((t - a.at) / (b.at - a.at))
      return { at: t, x: lerp(a.x, b.x, k), y: lerp(a.y, b.y, k) }
    }
  }
  return pts[pts.length - 1]!
}
