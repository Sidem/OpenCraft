// The weather, as pure maths (no GL, no DOM; tested in web/test/weather.test.mjs). `weatherAt(seed, days, biome)` is a
// function of the world's seed, the game time in days (`FrameInfo.days`: the core tick, so every co-op peer agrees) and the
// biome the player stands in. Nothing is stored or saved: the same inputs give the same sky on every machine.
// A "storminess" curve, the sum of three slow noises of the seed and the time, says how stormy the day is; clear sky is
// the usual state, and every few days a front brings cloud, then gloom, then rain or snow, and passes. The biome shifts it
// (deserts are dry, lowlands wet) and decides rain or snow with the temperature: each biome has a warmth, the year
// (`YEAR_DAYS`, the same turn as the stars) swings it, and below zero the precipitation is snow.
// `WeatherEase` smooths what is drawn (crossing a biome border, a time skip, the menu switch) over a few seconds, and
// `WeatherSky` (one per renderer) turns the sky settings and the eased weather into the `SkyLook` that the sky shader and
// `skyAt` (render/sky.ts) take. To tune: `storminess`'s weights and the three smoothsteps in `weatherAt` (the test pins the
// shares of clear, cloudy and wet time and how fast things may change); the biome rows `WET` and `WARMTH`.

/** Game days in a year: the same as astro.ts `YEAR_DAYS` (a test checks; a value import would not load under `node --test`). */
export const YEAR_DAYS = 96;

export type WeatherKind = 'clear' | 'cloudy' | 'gloomy' | 'rain' | 'snow';

export interface Weather {
  /** Share of the sky under cloud, 0 to 1. */
  cover: number;
  /** How dark and heavy the cloud is, 0 to 1 (dims the sun, the stars and the fog). */
  gloom: number;
  /** How hard it rains or snows, 0 to 1 (the particles of step S3 read it). */
  precip: number;
  /** Whether precipitation here falls as snow. */
  snow: boolean;
}

/** What the sky is drawn with: the eased weather, whether the red dusk is on, and the cloud quality (0 none, 1 Low, 2 Full). */
export interface SkyLook {
  cover: number;
  gloom: number;
  precip: number;
  dusk: boolean;
  clouds: 0 | 1 | 2;
}

export const CLEAR: Weather = { cover: 0.12, gloom: 0, precip: 0, snow: false };

/** Biomes in the engine's order (audio/surroundings.ts): plains, desert, highlands, lowlands, basalt fields. */
const WET = [0, -0.12, 0.02, 0.04, 0];
const WARMTH = [0.4, 1.0, 0.05, 0.4, 0.3];
/** How far the year swings the temperature each way. */
const SWING = 0.5;

function hash(seed: number, i: number): number {
  let h = (seed ^ Math.imul(i, 0x9e3779b1)) >>> 0;
  h = Math.imul(h ^ (h >>> 16), 0x85ebca6b);
  h = Math.imul(h ^ (h >>> 13), 0xc2b2ae35);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

/** Smooth 1-D value noise in 0..1 with one lattice point per unit of `x`. */
function noise(seed: number, x: number): number {
  const i = Math.floor(x), f = x - i;
  const t = f * f * f * (f * (f * 6 - 15) + 10);
  const a = hash(seed, i), b = hash(seed, i + 1);
  return a + (b - a) * t;
}

const smooth = (a: number, b: number, x: number) => {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
};

/** How stormy the world is at `days`, 0 to 1: fronts a day or two long, shorter squalls, and wetter and drier weeks. */
export function storminess(seed: number, days: number): number {
  const s = seed >>> 0;
  const front = noise(s, days / 1.6), squall = noise(s ^ 0x5bd1e995, days / 0.9), season = noise(s ^ 0x27d4eb2f, days / 9);
  return 0.66 * front + 0.16 * squall + 0.18 * season;
}

/** The temperature at `days` in a biome, about -0.5 (cold) to 1.5 (hot); snow falls below zero. */
export function temperature(biome: number, days: number): number {
  // The first half of a year is the warm half, the second the cold one.
  return (WARMTH[biome] ?? WARMTH[0]) + SWING * Math.sin((2 * Math.PI * days) / YEAR_DAYS);
}

export function weatherAt(seed: number, days: number, biome: number): Weather {
  const s = storminess(seed, days) + (WET[biome] ?? 0);
  const precip = smooth(0.68, 0.78, s);
  const gloom = Math.max(smooth(0.6, 0.74, s) * 0.85, precip);
  const cover = Math.max(CLEAR.cover, smooth(0.48, 0.68, s), gloom);
  return { cover, gloom, precip, snow: temperature(biome, days) < 0 };
}

/** The word for a weather: what a player would call it. */
export function kindOf(w: Weather): WeatherKind {
  if (w.precip > 0.15) return w.snow ? 'snow' : 'rain';
  if (w.gloom > 0.4) return 'gloomy';
  return w.cover > 0.4 ? 'cloudy' : 'clear';
}

/** The weather as drawn: the model, eased, and turned into the `SkyLook` for the sky's settings (the renderer owns one). */
export class WeatherSky {
  readonly ease = new WeatherEase();
  private last = 0;

  /** The look for this frame (`now` in milliseconds), or `undefined` when weather and red dusk are both off: today's plain sky. */
  look(settings: { weather: 0 | 1 | 2; dusk: boolean }, seed: number, days: number, biome: number, now: number): SkyLook | undefined {
    const dt = (now - this.last) / 1000;
    this.last = now;
    if (settings.weather === 0) {
      this.ease.reset();
      return settings.dusk ? { cover: 0, gloom: 0, precip: 0, dusk: true, clouds: 0 } : undefined;
    }
    const w = this.ease.update(weatherAt(seed, days, biome), dt);
    return { cover: w.cover, gloom: w.gloom, precip: w.precip, dusk: settings.dusk, clouds: settings.weather };
  }
}

/** Moves the drawn weather towards the real one over a few seconds; the first call snaps. */
export class WeatherEase {
  value: Weather = { ...CLEAR };
  private primed = false;

  update(target: Weather, dt: number): Weather {
    const k = this.primed ? 1 - Math.exp(-Math.min(dt, 1) / 3) : 1;
    const v = this.value;
    v.cover += (target.cover - v.cover) * k;
    v.gloom += (target.gloom - v.gloom) * k;
    v.precip += (target.precip - v.precip) * k;
    v.snow = target.snow;
    this.primed = true;
    return v;
  }

  /** Forget the history: the next `update` snaps, and what is drawn (and heard) is clear sky (used while the weather is off). */
  reset(): void {
    this.primed = false;
    this.value = { ...CLEAR };
  }
}
