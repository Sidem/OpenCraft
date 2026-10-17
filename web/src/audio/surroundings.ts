// What the player stands in, as the audio systems read it: the layout of the floats in `Game.surroundings()`
// (crates/engine/src/surroundings/mod.rs, which must match `S`), the daylight curve and the derived readings
// ambience.ts and the music director both use. Pure functions, no Web Audio.
// To add a reading: a field in `S` (and in the engine), then a function here.

/** Indices of the floats in `Game.surroundings()`. */
export const S = { biome: 0, sky: 1, open: 2, height: 3, depth: 4, sea: 5, fresh: 6, trees: 7, din: 8 } as const;
/** `Biome` variants in the engine's order. */
export const DESERT = 1, HIGHLANDS = 2, BASALT = 4;

export const clamp01 = (v: number) => Math.min(1, Math.max(0, v));
export const smooth = (a: number, b: number, v: number) => {
  const t = clamp01((v - a) / (b - a));
  return t * t * (3 - 2 * t);
};

/** 0 at night to 1 by day for the time of day `tod` (0 midnight, 0.5 noon). */
export const daylight = (tod: number) => smooth(0.2, 0.3, tod) * (1 - smooth(0.7, 0.8, tod));

/** 0 in the open to 1 well below a roof of rock. */
export const caveness = (s: ArrayLike<number>) => (1 - s[S.sky]) * clamp01(s[S.depth] / 8);

/** The kinds of place music is chosen for. */
export type Context = 'day' | 'night' | 'underground' | 'desert' | 'highlands' | 'sea';

/** The one context a reading falls in: underground beats night, night beats the biome, the biome beats the sea. */
export function contextFor(s: ArrayLike<number>, tod: number): Context {
  if (caveness(s) > 0.5) return 'underground';
  if (daylight(tod) < 0.5) return 'night';
  if (s[S.biome] === DESERT) return 'desert';
  if (s[S.biome] === HIGHLANDS) return 'highlands';
  if (s[S.sea] > 0.5) return 'sea';
  return 'day';
}
