// The weather as drawn (no GL, no DOM; tested in web/test/weather.test.mjs). The engine owns the weather (weather/ in the
// engine: fronts drifting over the world, the temperature of the place, rain turning to snow below zero); `weatherFrom`
// reads its `game.weather_here()` numbers, which main.ts polls a few times a second into `FrameInfo.weather`.
// `WeatherEase` smooths what is drawn (a front arriving, a time skip, the menu switch) over a few seconds, and
// `WeatherSky` (one per renderer) turns the sky settings and the eased weather into the `SkyLook` that the sky shader and
// `skyAt` (render/sky.ts) take. The console's `weather` command holds a kind (`weatherOf`, `WeatherSky.hold`), drawn in
// the place's real temperature (`heldIn`: rain turns to snow below 0 °C, held snow brings the cold). To tune what is drawn: the
// ease's time constant and the kinds in `weatherOf`; the weather itself is tuned in the engine.

export type WeatherKind = 'clear' | 'cloudy' | 'gloomy' | 'rain' | 'snow';

export interface Weather {
  /** Share of the sky under cloud, 0 to 1. */
  cover: number;
  /** How dark and heavy the cloud is, 0 to 1 (dims the sun, the stars and the fog). */
  gloom: number;
  /** How hard it rains or snows, 0 to 1 (the particles read it). */
  precip: number;
  /** Whether precipitation here falls as snow. */
  snow: boolean;
  /** The temperature here in degrees Celsius (snow lies while it is below zero). */
  temp: number;
}

/** What the sky is drawn with: the eased weather, whether the red dusk is on, and the cloud quality (0 none, 1 Low, 2 Full). */
export interface SkyLook {
  cover: number;
  gloom: number;
  precip: number;
  dusk: boolean;
  clouds: 0 | 1 | 2;
}

/** A clear day still has scattered fair-weather clouds (the key art's sky); `kindOf` calls it cloudy above 0.4. */
export const CLEAR: Weather = { cover: 0.38, gloom: 0, precip: 0, snow: false, temp: 12 };

/** The engine's `weather_here()`: cover, gloom, precipitation, snow (0 or 1), temperature. */
export function weatherFrom(v: ArrayLike<number>): Weather {
  return { cover: v[0], gloom: v[1], precip: v[2], snow: v[3] > 0.5, temp: v[4] };
}

/** The word for a weather: what a player would call it. */
export function kindOf(w: Weather): WeatherKind {
  if (w.precip > 0.15) return w.snow ? 'snow' : 'rain';
  if (w.gloom > 0.4) return 'gloomy';
  return w.cover > 0.4 ? 'cloudy' : 'clear';
}

/** A weather of a kind at a strength (0 to 1, only rain and snow use it): what the console's `weather` command forces. */
export function weatherOf(kind: WeatherKind, amount = 1): Weather {
  const a = Math.min(1, Math.max(0, amount));
  switch (kind) {
    case 'clear': return { ...CLEAR };
    case 'cloudy': return { cover: 0.7, gloom: 0.15, precip: 0, snow: false, temp: 12 };
    case 'gloomy': return { cover: 1, gloom: 0.75, precip: 0, snow: false, temp: 10 };
    default: return { cover: 1, gloom: 0.75 + 0.25 * a, precip: Math.max(0.2, a), snow: kind === 'snow', temp: kind === 'snow' ? -4 : 9 };
  }
}

/** The coldest a held snow keeps it, in degrees Celsius: cold enough that the snow lies. */
export const HELD_SNOW_TEMP = -4;

/** A held weather in the place's real temperature (`model.temp`): a held snow holds the cold too (`HELD_SNOW_TEMP` at
 * most); any other keeps the real temperature, so what falls is snow below 0 °C and rain above it, as in the engine. */
export function heldIn(held: Weather, model: Weather): Weather {
  const temp = held.snow ? Math.min(model.temp, HELD_SNOW_TEMP) : model.temp;
  return { ...held, temp, snow: temp < 0 };
}

/** The weather as drawn: the engine's, eased, and turned into the `SkyLook` for the sky's settings (the renderer owns one). */
export class WeatherSky {
  readonly ease = new WeatherEase();
  /** When set, this weather is drawn instead of the engine's, in its temperature (`heldIn`; the console's `weather`
   * command; never saved). Set it with `hold`. */
  forced: Weather | undefined;
  private last = 0;

  /** Holds `w` (`undefined`: back to the engine's weather); the sky snaps to it at once rather than easing over. */
  hold(w: Weather | undefined): void {
    this.forced = w;
    this.ease.reset();
  }

  /** The look for this frame (`now` in milliseconds), or `undefined` when weather and red dusk are both off: today's plain sky. */
  look(settings: { weather: 0 | 1 | 2; dusk: boolean }, model: Weather, now: number): SkyLook | undefined {
    const dt = (now - this.last) / 1000;
    this.last = now;
    if (settings.weather === 0) {
      this.ease.reset();
      return settings.dusk ? { cover: 0, gloom: 0, precip: 0, dusk: true, clouds: 0 } : undefined;
    }
    const w = this.ease.update(this.forced ? heldIn(this.forced, model) : model, dt);
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
    v.temp += (target.temp - v.temp) * k;
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
