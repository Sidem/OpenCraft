// What the player stands in, as the audio systems read it: the layout of the floats in `Game.surroundings()`
// (crates/engine/src/surroundings/mod.rs, which must match `S`), the daylight curve and the derived readings
// ambience.ts and the music director both use. Pure functions, no Web Audio.
// To add a reading: a field in `S` (and in the engine), then a function here.

/** Indices of the floats in `Game.surroundings()`. */
export const S = { biome: 0, sky: 1, open: 2, height: 3, depth: 4, sea: 5, fresh: 6, trees: 7, din: 8, town: 9 } as const;
/** `Biome` variants in the engine's order. */
export const DESERT = 1, HIGHLANDS = 2, LOWLANDS = 3, BASALT = 4, SAVANNA = 5, BOREAL = 6, WOODLAND = 7, BAMBOO = 8, REDWOODS = 9;
/** Machine din threshold (0..1) above which a working factory gets its own music. */
export const FACTORY_DIN = 0.6;
/** Height above sea level where Orbit-era players hear space music by day as well as night. */
const SPACE_HEIGHT = 60;

export const clamp01 = (v: number) => Math.min(1, Math.max(0, v));
export const smooth = (a: number, b: number, v: number) => {
  const t = clamp01((v - a) / (b - a));
  return t * t * (3 - 2 * t);
};

/** 0 at night to 1 by day for the time of day `tod` (0 midnight, 0.5 noon). */
export const daylight = (tod: number) => smooth(0.2, 0.3, tod) * (1 - smooth(0.7, 0.8, tod));

/** 0 in the open to 1 well below a roof of rock. */
export const caveness = (s: ArrayLike<number>) => (1 - s[S.sky]) * clamp01(s[S.depth] / 8);

/** The kinds of place and era music is chosen for. */
export type Context =
  | 'day'
  | 'night'
  | 'underground'
  | 'desert'
  | 'highlands'
  | 'lowlands'
  | 'basalt'
  | 'sea'
  | 'town'
  | 'factory_early'
  | 'factory_mid'
  | 'factory_high'
  | 'space';

/** The one context a reading falls in: caves, then a working factory (by science era), then a town, then night/space, then biome or sea. */
export function contextFor(s: ArrayLike<number>, tod: number, era = 0): Context {
  if (caveness(s) > 0.5) return 'underground';
  if (s[S.din] >= FACTORY_DIN) return era >= 4 ? 'factory_high' : era >= 3 ? 'factory_mid' : 'factory_early';
  if ((s[S.town] ?? 0) > 0) return 'town';
  if (daylight(tod) < 0.5) return era >= 5 ? 'space' : 'night';
  if (era >= 5 && s[S.height] > SPACE_HEIGHT) return 'space';
  if (s[S.biome] === DESERT) return 'desert';
  if (s[S.biome] === HIGHLANDS) return 'highlands';
  if (s[S.biome] === BASALT) return 'basalt';
  if (s[S.biome] === LOWLANDS) return 'lowlands';
  if (s[S.sea] > 0.5) return 'sea';
  return 'day';
}
