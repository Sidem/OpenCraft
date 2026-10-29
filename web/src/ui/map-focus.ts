// "Show on map" from the analytics hub: the machines of one kind the world map rings (`analytics_machines_of`) and the
// view that shows them all. Pure functions (tested in web/test/map-focus.test.mjs: no DOM, no enums in here); the rings,
// the chip and the hover lines are drawn by map-focus-layer.ts.

/** One machine of the kind shown: where it stands, its average pace (-1 idle) and what costs it most (-1 none). */
export interface Focused {
  x: number;
  y: number;
  z: number;
  percent: number;
  cause: number;
}

/** Reads `analytics_machines_of`: `fields` numbers per machine, `[x, y, z, percent, cause]` first. */
export function parseMachines(raw: ArrayLike<number>, fields: number): Focused[] {
  const out: Focused[] = [];
  for (let i = 0; i + 4 < raw.length; i += fields) {
    out.push({ x: raw[i], y: raw[i + 1], z: raw[i + 2], percent: raw[i + 3], cause: raw[i + 4] });
  }
  return out;
}

/** The centre (world columns) and zoom step (an index into `zooms`, blocks per screen pixel, closest first) that show
 * every point at least `pad` screen pixels inside a `w` × `h` view: the closest step that fits, but never closer than
 * `closest`. Points too far apart for the widest step get the widest step around their middle. Null for no points. */
export function fitView(
  points: readonly { x: number; z: number }[],
  w: number,
  h: number,
  zooms: readonly number[],
  closest: number,
  pad: number,
): { x: number; z: number; zoom: number } | null {
  if (points.length === 0) return null;
  let x0 = Infinity, x1 = -Infinity, z0 = Infinity, z1 = -Infinity;
  for (const p of points) {
    x0 = Math.min(x0, p.x);
    x1 = Math.max(x1, p.x + 1);
    z0 = Math.min(z0, p.z);
    z1 = Math.max(z1, p.z + 1);
  }
  const room = (side: number) => Math.max(1, side - 2 * pad);
  const need = Math.max((x1 - x0) / room(w), (z1 - z0) / room(h));
  let zoom = Math.max(0, Math.min(closest, zooms.length - 1));
  while (zoom < zooms.length - 1 && zooms[zoom] < need) zoom++;
  return { x: (x0 + x1) / 2, z: (z0 + z1) / 2, zoom };
}

/** Counts for the chip: how many run at full speed, how many each cause holds back (by cause index), how many idle. */
export function tally(machines: readonly Focused[]): { full: number; held: number[]; idle: number } {
  let full = 0, idle = 0;
  const held: number[] = [];
  for (const m of machines) {
    if (m.percent < 0) idle++;
    else if (m.cause < 0) full++;
    else held[m.cause] = (held[m.cause] ?? 0) + 1;
  }
  return { full, held, idle };
}
