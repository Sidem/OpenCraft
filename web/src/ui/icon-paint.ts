// Paints an item icon from its model parts (`item_model`, `item_model_floats()` each) on a 2D canvas: isometric, +x
// right-down, +z left-down, +y up, parts in their order (back to front: there is no depth test). A box draws its sides
// turned to the viewer and its top (a leaned one every face turned to the viewer); a round body (`icon-geometry.ts`) its outline filled with its side texture, shaded
// round from edge to edge, then the end that faces the viewer as the inscribed ellipse of that end's square. Layers are
// the texture array's images (`hud.ts` keeps them). `view` places one cube unit: `S` pixels, centred on `cx, cy`.

import {
  boxAxes, endCentre, farEndShows, iso, outline, roundBody, shadeOf, sideShades, type P2, type V3,
} from './icon-geometry';

export interface IconView {
  px: number;
  S: number;
  cx: number;
  cy: number;
}

/** Paints every part of `model` (records of `stride` floats). */
export function paintParts(ctx: CanvasRenderingContext2D, layers: HTMLCanvasElement[], view: IconView,
  model: ArrayLike<number>, stride: number): void {
  for (let i = 0; i < model.length; i += stride) {
    const p = Array.from({ length: stride }, (_, k) => model[i + k]);
    if (roundBody(p)) paintRound(ctx, layers, view, p);
    else if (p[10] === 4) paintLeaned(ctx, layers, view, p);
    else paintBox(ctx, layers, view, p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7], p[8]);
  }
}

/** A box at x, y, z of size a, h, b turned by `yaw` about its vertical axis. */
export function paintBox(ctx: CanvasRenderingContext2D, layers: HTMLCanvasElement[], view: IconView, x: number, y: number,
  z: number, a: number, h: number, b: number, yaw: number, topLayer: number, sideLayer: number): void {
  const { S } = view, k = S / 16;
  const top = layers[topLayer], side = layers[sideLayer];
  const c = Math.cos(yaw), s = Math.sin(yaw);
  // Its local x runs along (cos, sin) in the xz plane and its local z along (-sin, cos). A world vector (vx, vz) on
  // screen is ((vx - vz) S/2, (vx + vz) S/4); an image is 16 px, so a face's matrix takes its edge vectors over 16.
  const corner = (lx: number, lz: number) => screen(view, [x + c * lx - s * lz, y + h / 2, z + s * lx + c * lz]);
  const edge = (vx: number, vz: number): P2 => [((vx - vz) * S) / 32, ((vx + vz) * S) / 64];
  const down = (h * k) / 2;
  // Side faces seen from outside, left to right: start corner and edge; shaded by which way they face.
  const sides: [number, number, number, number, number, number][] = [
    [-a / 2, b / 2, c * a, s * a, -s, c], // +z face runs along +x
    [a / 2, b / 2, s * b, -c * b, c, s], // +x face runs along -z
    [a / 2, -b / 2, -c * a, -s * a, s, -c], // -z face runs along -x
    [-a / 2, -b / 2, -s * b, c * b, -c, -s], // -x face runs along +z
  ];
  for (const [lx, lz, ex, ez, nx, nz] of sides) {
    if (nx + nz <= 0.01) continue; // faces turned away from the viewer
    const [sx, sy] = edge(ex, ez), [px, py] = corner(lx, lz);
    face(ctx, side, [sx, sy, 0, down, px, py], 0.31 + 0.09 * (nx - nz));
  }
  // The top face's texture starts at its back corner, then runs along local +x and +z.
  const [ax, ay] = edge(c * a, s * a), [bx, by] = edge(-s * b, c * b), [qx, qy] = corner(-a / 2, -b / 2);
  face(ctx, top, [ax, ay, bx, by, qx, qy], 0);
}

/** A round body: its side over its outline, two texture widths across (mirrored, as the shader wraps them) as a box shows
 * two sides, then its end. */
function paintRound(ctx: CanvasRenderingContext2D, layers: HTMLCanvasElement[], view: IconView, p: number[]): void {
  const b = roundBody(p)!;
  const { S } = view;
  const shape = outline(b).map((q) => toPx(view, q));
  const near = toPx(view, iso(endCentre(b, -1))), far = toPx(view, iso(endCentre(b, 1)));
  const along: P2 = [far[0] - near[0], far[1] - near[1]];
  const len = Math.hypot(along[0], along[1]);
  // Across the side, pointing right on screen (an end-on axis has no direction: take the screen's x).
  let across: P2 = len > 1e-3 ? [-along[1] / len, along[0] / len] : [1, 0];
  if (across[0] < 0) across = [-across[0], -across[1]];
  let reach = 0;
  for (const q of shape) reach = Math.max(reach, Math.abs((q[0] - near[0]) * across[0] + (q[1] - near[1]) * across[1]));
  ctx.save();
  ctx.beginPath();
  shape.forEach((q, i) => (i ? ctx.lineTo(q[0], q[1]) : ctx.moveTo(q[0], q[1])));
  ctx.closePath();
  ctx.clip();
  const side = layers[p[8]];
  // Tiles run across from the left edge, rows from the far end to the near one; the clip trims them to the body.
  const tile: P2 = [(across[0] * reach) / 16, (across[1] * reach) / 16];
  const rows: P2 = [-along[0] / 16, -along[1] / 16];
  for (const k of [-1, 1]) {
    // From each edge in to the middle: the right copy is the left one mirrored, as the shader mirrors every other.
    const o: P2 = [far[0] + across[0] * reach * k, far[1] + across[1] * reach * k];
    const f = 16 / side.width;
    ctx.setTransform(-k * tile[0] * f, -k * tile[1] * f, rows[0] * f, rows[1] * f, o[0], o[1]);
    // Reach a little past both ends so a tapered rim is covered.
    ctx.drawImage(side, 0, -side.height * 0.5, side.width, side.height * 2);
  }
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  const g = ctx.createLinearGradient(near[0] - across[0] * reach, near[1] - across[1] * reach,
    near[0] + across[0] * reach, near[1] + across[1] * reach);
  for (const { t, shade } of sideShades(b, [across[0], across[1]])) g.addColorStop((t + 1) / 2, `rgba(0,0,0,${shade})`);
  ctx.globalCompositeOperation = 'source-atop';
  ctx.fillStyle = g;
  ctx.fillRect(0, 0, view.px, view.px);
  ctx.globalCompositeOperation = 'source-over';
  ctx.restore();
  // The end turned to the viewer: its square's matrix, clipped to the circle inscribed in it.
  const end = farEndShows(b) ? 1 : -1;
  const r = end === 1 ? b.taper : 1;
  const c = endCentre(b, end);
  const u: V3 = b.u.map((x) => x * b.ru * r * 2) as V3, v: V3 = b.v.map((x) => x * b.rv * r * 2) as V3;
  const vec = (w: V3): P2 => { const q = iso(w); return [(q[0] * S) / 16, (q[1] * S) / 16]; };
  const o = screen(view, [c[0] - u[0] / 2 - v[0] / 2, c[1] - u[1] / 2 - v[1] / 2, c[2] - u[2] / 2 - v[2] / 2]);
  const [ux, uy] = vec(u), [vx, vy] = vec(v);
  const cap = layers[end === 1 ? p[7] : p[9]];
  ctx.save();
  ctx.setTransform(ux, uy, vx, vy, o[0], o[1]);
  ctx.beginPath();
  ctx.arc(8, 8, 8, 0, 2 * Math.PI);
  ctx.clip();
  ctx.drawImage(cap, 0, 0, 16, 16);
  const shade = shadeOf(b.axis.map((x) => x * end) as V3);
  if (shade > 0) {
    ctx.fillStyle = `rgba(0,0,0,${shade})`;
    ctx.fillRect(0, 0, 16, 16);
  }
  ctx.restore();
}

/** A leaned box (form 4, its pitch in place of the taper): each face turned to the viewer from its corner and two edges,
 * the sides' texture upright along the box, shaded by which way the face looks. */
function paintLeaned(ctx: CanvasRenderingContext2D, layers: HTMLCanvasElement[], view: IconView, p: number[]): void {
  const [ax, ay, az] = boxAxes(p[6], p[11]);
  const half = [p[3] / 2, p[4] / 2, p[5] / 2];
  const at = (i: number, j: number, k: number): V3 =>
    [0, 1, 2].map((d) => p[d] + ax[d] * i * half[0] + ay[d] * j * half[1] + az[d] * k * half[2]) as V3;
  const vec = (a: V3, b: V3): P2 => {
    const q = iso([b[0] - a[0], b[1] - a[1], b[2] - a[2]]);
    return [(q[0] * view.S) / 16, (q[1] * view.S) / 16];
  };
  const neg = (v: V3): V3 => [-v[0], -v[1], -v[2]];
  // Each face: its outward normal, layer, the corner its texture starts at and the corners along its two edges.
  const faces: [V3, number, V3, V3, V3][] = [
    [az, p[8], at(-1, 1, 1), at(1, 1, 1), at(-1, -1, 1)],
    [ax, p[8], at(1, 1, 1), at(1, 1, -1), at(1, -1, 1)],
    [neg(az), p[8], at(1, 1, -1), at(-1, 1, -1), at(1, -1, -1)],
    [neg(ax), p[8], at(-1, 1, -1), at(-1, 1, 1), at(-1, -1, -1)],
    [ay, p[7], at(-1, 1, -1), at(1, 1, -1), at(-1, 1, 1)],
    [neg(ay), p[9], at(-1, -1, 1), at(1, -1, 1), at(-1, -1, -1)],
  ];
  for (const [n, layer, o, a, b] of faces) {
    if (n[0] + n[1] + n[2] <= 0.01) continue; // turned away from the viewer
    const [ex, ey] = vec(o, a), [dx, dy] = vec(o, b), [ox, oy] = screen(view, o);
    face(ctx, layers[layer], [ex, ey, dx, dy, ox, oy], shadeOf(n));
  }
}

/** A face: `img` (16 px across in the matrix's space) through matrix `m`, darkened by `shade`. */
function face(ctx: CanvasRenderingContext2D, img: HTMLCanvasElement, m: [number, number, number, number, number, number],
  shade: number): void {
  const f = 16 / img.width; // the matrices are for a 16 px image; textures are finer
  ctx.setTransform(m[0] * f, m[1] * f, m[2] * f, m[3] * f, m[4], m[5]);
  ctx.drawImage(img, 0, 0);
  if (shade > 0) {
    ctx.globalCompositeOperation = 'source-atop';
    ctx.fillStyle = `rgba(0,0,0,${shade})`;
    ctx.fillRect(0, 0, img.width, img.height);
    ctx.globalCompositeOperation = 'source-over';
  }
}

function screen(view: IconView, p: V3): P2 {
  return toPx(view, iso(p));
}

function toPx(view: IconView, q: P2): P2 {
  return [view.px / 2 + (q[0] - view.cx) * view.S, view.px / 2 + (q[1] - view.cy) * view.S];
}
