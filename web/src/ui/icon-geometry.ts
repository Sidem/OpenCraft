// The geometry of an item icon's round bodies (`icon-paint.ts` paints them): where a part's axis, ends and rims lie,
// its outline on the icon (the hull of both rims), which end faces the viewer and how its side shades from one edge to
// the other. Parts are the engine's `item_model` records (`api/content.rs`); the icon looks from +x, +z and above, so
// the viewer is along (1, 1, 1) and a point (x, y, z) lands at `iso`. Pure functions, tested in
// `web/test/icon-geometry.test.mjs`; the engine fits icons with the same outline (`item_models/icon_frame.rs`).

export type V3 = [number, number, number];
export type P2 = [number, number];

/** A round body: its centre, unit axis (towards its far end), unit across directions and their radii at the near end. */
export interface RoundBody {
  centre: V3;
  axis: V3;
  u: V3;
  v: V3;
  length: number;
  ru: number;
  rv: number;
  /** The far end's radii as a share of the near end's. */
  taper: number;
}

/** Points sampled round each rim for the outline. */
export const RIM_POINTS = 48;

/** A point on the icon, in cube units right and down from the centre of a full cube's icon. */
export function iso(p: V3): P2 {
  return [(p[0] - p[2]) / 2, (p[0] + p[2]) / 4 - p[1] / 2];
}

/**
 * The round body of a part record (centre xyz, size xyz, yaw, three layers, form, taper), or null for a box (form 0,
 * or 4 leaned). Its size is the box it fills before its yaw turns it; its form says the axis: 1 y, 2 x, 3 z.
 */
export function roundBody(p: ArrayLike<number>): RoundBody | null {
  const form = p[10];
  if (form < 1 || form > 3) return null;
  const c = Math.cos(p[6]), s = Math.sin(p[6]);
  const x: V3 = [c, 0, s], y: V3 = [0, 1, 0], z: V3 = [-s, 0, c];
  const [sx, sy, sz] = [p[3], p[4], p[5]];
  const centre: V3 = [p[0], p[1], p[2]];
  const taper = p[11];
  if (form === 1) return { centre, axis: y, u: x, v: z, length: sy, ru: sx / 2, rv: sz / 2, taper };
  if (form === 2) return { centre, axis: x, u: z, v: y, length: sx, ru: sz / 2, rv: sy / 2, taper };
  return { centre, axis: z, u: x, v: y, length: sz, ru: sx / 2, rv: sy / 2, taper };
}

const add = (a: V3, b: V3, k: number): V3 => [a[0] + b[0] * k, a[1] + b[1] * k, a[2] + b[2] * k];

/** The centre of the near (-1) or far (+1) end. */
export function endCentre(b: RoundBody, end: -1 | 1): V3 {
  return add(b.centre, b.axis, (end * b.length) / 2);
}

/** The point at angle `a` round the rim of an end. */
export function rimPoint(b: RoundBody, end: -1 | 1, a: number): V3 {
  const k = end === 1 ? b.taper : 1;
  return add(add(endCentre(b, end), b.u, b.ru * k * Math.cos(a)), b.v, b.rv * k * Math.sin(a));
}

/** The body's outline on the icon: the convex hull of both rims, counter-clockwise on screen (y down). */
export function outline(b: RoundBody): P2[] {
  const pts: P2[] = [];
  for (const end of [-1, 1] as const) {
    for (let i = 0; i < RIM_POINTS; i++) pts.push(iso(rimPoint(b, end, (2 * Math.PI * i) / RIM_POINTS)));
  }
  return hull(pts);
}

/** The convex hull of `pts` (Andrew's monotone chain). */
export function hull(pts: P2[]): P2[] {
  const p = [...pts].sort((a, b) => a[0] - b[0] || a[1] - b[1]);
  if (p.length < 3) return p;
  const cross = (o: P2, a: P2, b: P2) => (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
  const half = (list: P2[]) => {
    const h: P2[] = [];
    for (const q of list) {
      while (h.length >= 2 && cross(h[h.length - 2], h[h.length - 1], q) <= 0) h.pop();
      h.push(q);
    }
    h.pop();
    return h;
  };
  return [...half(p), ...half(p.reverse())];
}

/**
 * The frame of a box turned by `yaw` and leaned by `pitch` about its own x (the engine's `Form::Lean`, the shader's
 * yaw * tilt): where its local x, y and z point.
 */
export function boxAxes(yaw: number, pitch: number): [V3, V3, V3] {
  const c = Math.cos(yaw), s = Math.sin(yaw), cp = Math.cos(pitch), sp = Math.sin(pitch);
  const turn = (v: V3): V3 => [c * v[0] - s * v[2], v[1], s * v[0] + c * v[2]];
  return [turn([1, 0, 0]), turn([0, cp, sp]), turn([0, -sp, cp])];
}

/** Whether the far end faces the viewer (else the near one does). */
export function farEndShows(b: RoundBody): boolean {
  return b.axis[0] + b.axis[1] + b.axis[2] > 0;
}

/**
 * How dark a face turned to `n` is drawn, 0 (lit) to about 0.44: as the icon shades a box, a top face 0 and a side
 * by which way it faces (`0.31 + 0.09 (nx - nz)`), and in between by how much it looks up.
 */
export function shadeOf(n: V3): number {
  const up = Math.max(n[1], 0);
  return (1 - up) * (0.31 + 0.09 * (n[0] - n[2]));
}

/**
 * The side's shade across the outline: at `t` from -1 to 1 along `across` (a unit screen direction perpendicular to
 * the axis), how dark the side is there, for the half of the side turned to the viewer, sorted by `t`.
 */
export function sideShades(b: RoundBody, across: P2, steps = 32): { t: number; shade: number }[] {
  const out: { t: number; shade: number }[] = [];
  let reach = 0;
  for (let i = 0; i < steps; i++) {
    const a = (2 * Math.PI * i) / steps;
    const n: V3 = add(add([0, 0, 0], b.u, Math.cos(a)), b.v, Math.sin(a));
    const off = iso(add(add([0, 0, 0], b.u, b.ru * Math.cos(a)), b.v, b.rv * Math.sin(a)));
    const t = off[0] * across[0] + off[1] * across[1];
    reach = Math.max(reach, Math.abs(t));
    if (n[0] + n[1] + n[2] > 0) out.push({ t, shade: shadeOf(n) });
  }
  return out.map((s) => ({ t: s.t / (reach || 1), shade: s.shade })).sort((a, b) => a.t - b.t);
}
