// The real sky above the game world, as pure maths (no Web Audio, no GL; tested in web/test/astro.test.mjs).
// The game's sun climbs and sets like the sun seen from the equator at a declination of about -19 degrees
// (sky.ts `skyAt`: rising east at 0.25, high in the south at noon), so the stars use the same observer: the celestial
// poles sit on the north and south horizons and every star rises straight up in the east and sets in the west.
// World axes as in sky.ts: +x east, +y up, +z south.
// `skyFrame(days)` gives the turn that carries the catalogue's equatorial directions into the world, and where the
// moon is and how much of it is lit. `days` counts game days since the world began, as a fraction (day 1 at 00:00 is
// 0); the stars drift one turn through the year (`YEAR_DAYS`) and the moon goes through its phases every
// `MOON_DAYS`. A game day is the sun's turn; the sun's right ascension grows with the year. To change the look of the
// calendar: the two constants and `SUN_RA0`.

export type Vec3 = [number, number, number];

/** Game days for the stars to come round once (the seasons of the night sky). */
export const YEAR_DAYS = 96;
/** Game days from one full moon to the next (twelve moons a year). */
export const MOON_DAYS = 8;
/** The sun's right ascension on the first day: 12 hours, so the first nights have Cassiopeia and Cygnus overhead. */
const SUN_RA0 = Math.PI;
/** The moon's angle east of the sun at the start: a first quarter, so the first evening has half a moon in the sky. */
const MOON_E0 = Math.PI / 2;
/** The sun's declination: the sun's path in `skyAt` is the circle at height 0.35 / hypot(1, 0.35) off the east-west plane. */
export const SUN_DEC = -Math.asin(0.35 / Math.hypot(1, 0.35));

export interface SkyFrame {
  /** Row-major 3x3 turning an equatorial unit vector (x towards right ascension 0, z to the north pole) into the world. */
  rotation: number[];
  /** Unit vector towards the moon. */
  moon: Vec3;
  /** 0 new moon, 1 full moon. */
  illumination: number;
  /** The sun's unit vector from the same turn (equals `skyAt(t).sun`). */
  sun: Vec3;
}

const TAU = 2 * Math.PI;

/** Equatorial unit vector of right ascension `ra` and declination `dec` (radians). */
export function equatorial(ra: number, dec: number): Vec3 {
  return [Math.cos(dec) * Math.cos(ra), Math.cos(dec) * Math.sin(ra), Math.sin(dec)];
}

/** Applies a row-major 3x3. */
export function apply(m: ArrayLike<number>, v: Vec3): Vec3 {
  return [
    m[0] * v[0] + m[1] * v[1] + m[2] * v[2],
    m[3] * v[0] + m[4] * v[1] + m[5] * v[2],
    m[6] * v[0] + m[7] * v[1] + m[8] * v[2],
  ];
}

/** The same turn the other way (the matrix is a rotation, so its transpose). */
export function applyInverse(m: ArrayLike<number>, v: Vec3): Vec3 {
  return [
    m[0] * v[0] + m[3] * v[1] + m[6] * v[2],
    m[1] * v[0] + m[4] * v[1] + m[7] * v[2],
    m[2] * v[0] + m[5] * v[1] + m[8] * v[2],
  ];
}

export function skyFrame(days: number): SkyFrame {
  const t = days - Math.floor(days);
  const sunRa = SUN_RA0 + (TAU * days) / YEAR_DAYS;
  const lst = sunRa + TAU * (t - 0.5); // the sun's hour angle is 0 at noon, so the stars' clock is this
  const s = Math.sin(lst), c = Math.cos(lst);
  // x = -cos(dec) sin(H), y = cos(dec) cos(H), z = -sin(dec) with H = lst - ra, written on the equatorial vector.
  const rotation = [-s, c, 0, c, s, 0, 0, 0, -1];
  const elong = MOON_E0 + (TAU * days) / MOON_DAYS;
  const moon = apply(rotation, equatorial(sunRa + elong, SUN_DEC * Math.cos(elong)));
  return {
    rotation,
    moon,
    illumination: (1 - Math.cos(elong)) / 2,
    sun: apply(rotation, equatorial(sunRa, SUN_DEC)),
  };
}

/** Where a direction lies on the Milky Way picture (u, v from the top left): right ascension 0 in the middle, growing to
 * the left, declination +90 on top. `eq` is an equatorial unit vector. */
export function skyPicture(eq: Vec3): [number, number] {
  const ra = Math.atan2(eq[1], eq[0]);
  const dec = Math.asin(Math.max(-1, Math.min(1, eq[2])));
  return [(((0.5 - ra / TAU) % 1) + 1) % 1, 0.5 - dec / Math.PI];
}

/** The star file (`stars.bin`, written by scripts/bake-sky/bake-sky.py) read into drawable arrays. */
export interface StarCatalogue {
  count: number;
  /** x, y, z of the equatorial unit vector, then the magnitude, per star. */
  data: Float32Array;
  /** Red, green, blue and unused bytes per star. */
  colour: Uint8Array;
  /** Pairs of star indices. */
  lines: Uint16Array;
}

export function readStars(buf: ArrayBuffer): StarCatalogue {
  const view = new DataView(buf);
  const count = view.getUint16(0, true), segments = view.getUint16(2, true);
  const data = new Float32Array(count * 4);
  const colour = new Uint8Array(count * 4);
  for (let i = 0; i < count; i++) {
    const at = 4 + i * 6;
    const ra = (view.getUint16(at, true) / 65536) * TAU;
    const dec = (view.getInt16(at + 2, true) / 32767) * (Math.PI / 2);
    const v = equatorial(ra, dec);
    data.set([v[0], v[1], v[2], view.getUint8(at + 4) / 25 - 2], i * 4);
    colour.set([...starColour(view.getUint8(at + 5) / 100 - 0.5), 255], i * 4);
  }
  const lines = new Uint16Array(segments * 2);
  for (let i = 0; i < lines.length; i++) lines[i] = view.getUint16(4 + count * 6 + i * 2, true);
  return { count, data, colour, lines };
}

/** A star's colour from its B-V colour index: blue-white for hot stars, white, yellow, orange for cool ones. */
export function starColour(ci: number): [number, number, number] {
  const stops: [number, number[]][] = [
    [-0.3, [155, 176, 255]], [0.0, [215, 225, 255]], [0.4, [255, 244, 234]], [0.8, [255, 222, 168]], [1.4, [255, 170, 100]], [2.0, [255, 130, 80]],
  ];
  if (ci <= stops[0][0]) return stops[0][1] as [number, number, number];
  for (let i = 1; i < stops.length; i++) {
    if (ci <= stops[i][0]) {
      const [a, ca] = stops[i - 1], [b, cb] = stops[i];
      const k = (ci - a) / (b - a);
      return [0, 1, 2].map((j) => Math.round(ca[j] + (cb[j] - ca[j]) * k)) as [number, number, number];
    }
  }
  return stops[stops.length - 1][1] as [number, number, number];
}
