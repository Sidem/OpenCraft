// What ambience.ts should play for a reading of the surroundings: `mixFor` turns the engine's `surroundings()` floats
// (audio/surroundings.ts), the time of day and the falling weather into target levels (wind rising with height and openness,
// surf by the sea, running water, rain, a low room tone and drips underground, birds by day near trees, crickets at night).
// Pure maths, no Web Audio, tested in web/test/ambience-levels.test.mjs (hence the `.ts` in its import: `node --test` needs
// the extension, and tsconfig allows it). To add a sound: a field in `Mix` and a line in `mixFor`, then a bed or a shot in
// ambience.ts.

import { BASALT, caveness, clamp01, DESERT, daylight, HIGHLANDS, S } from './surroundings.ts';

/** Target levels, each 0 to 1 (see `mixFor`). */
export interface Mix {
  wind: number;
  /** 0 to 1: how dry and hissing the wind is (deserts, basalt fields). */
  dry: number;
  surf: number;
  river: number;
  /** Rain on the ground and leaves; indoors a muffled patter on the roof. Snow falls silent. */
  rain: number;
  /** How open the sky is above, 0 to 1: dulls the rain under a roof. */
  sky: number;
  hum: number;
  birds: number;
  insects: number;
  drips: number;
  /** How much working machinery quiets everything else. */
  duck: number;
}

/** The weather as heard: how hard it rains or snows (0 to 1, the drawn weather's `precip`) and whether it is snow. */
export interface Falling {
  precip: number;
  snow: boolean;
}

/** What should be heard for a `surroundings` reading at the time of day `tod` (0 midnight, 0.5 noon) in the weather `falling`. */
export function mixFor(s: ArrayLike<number>, tod: number, falling: Falling = { precip: 0, snow: false }): Mix {
  const sky = s[S.sky], open = s[S.open], trees = s[S.trees], biome = s[S.biome];
  const day = daylight(tod);
  const cave = caveness(s);
  const dryland = biome === DESERT ? 1 : biome === BASALT ? 0.5 : 0;
  const high = clamp01((s[S.height] - 20) / 120 + (biome === HIGHLANDS ? 0.15 : 0));
  const precip = clamp01(falling.precip);
  const wind = sky * (0.3 + 0.7 * open) * (0.1 + 0.9 * high + 0.25 * dryland) * (1 - 0.5 * trees);
  const green = biome === DESERT || biome === BASALT ? 0 : 1;
  const calm = 1 - 0.85 * precip; // birds and crickets keep quiet in weather
  return {
    wind: clamp01(wind + 0.25 * precip * sky),
    dry: dryland,
    surf: s[S.sea] * (0.3 + 0.7 * sky),
    river: s[S.fresh] * (0.3 + 0.7 * sky),
    rain: falling.snow ? 0 : precip * (1 - cave) * (0.3 + 0.7 * sky),
    sky,
    hum: cave,
    birds: day * trees * clamp01(sky * 1.5 - 0.2) * green * calm,
    insects: (1 - day) * sky * (0.4 + 0.6 * trees) * (biome === HIGHLANDS ? 0.4 : biome === DESERT ? 0.3 : green) * calm,
    drips: cave * (s[S.depth] > 6 ? 1 : 0.5),
    duck: 1 - 0.55 * s[S.din],
  };
}
