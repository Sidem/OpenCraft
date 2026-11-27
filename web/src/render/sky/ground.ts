// How wet and how snowed under the open ground looks, as pure maths (no GL; tested in web/test/ground.test.mjs).
// `Ground` follows the drawn weather (`WeatherEase.value`, the engine's weather and temperature): rain darkens the ground
// quickly and lets it dry slowly; snowfall builds a cover, and the cold alone lays a thin one (frost, old snow) that
// deepens with the frost. Snow never melts below zero; above it, it melts the faster the warmer it is, and faster still in
// rain. Presentation only and local: a player who joins mid-winter starts with the cold's cover. To tune: the rates below,
// in "full changes per second", and `FROST_DEGREES`.

import type { Weather } from './weather';

/** Precipitation (0 to 1) above which the ground wets or the snow builds. */
const FALLING = 0.1;
const WET_RISE = 1 / 8;
const WET_DRY = 1 / 120;
const SNOW_RISE = 1 / 60;
/** How fast the cold alone lays its cover, and the degrees of frost that give the most of it (`FROST_COVER`). */
const FROST_RISE = 1 / 240;
const FROST_DEGREES = 8;
const FROST_COVER = 0.6;
/** Melting at 4 °C (scales with the warmth), and in rain. */
const SNOW_MELT = 1 / 240;
const SNOW_MELT_IN_RAIN = 1 / 40;

/** The cover the cold alone keeps at `temp` degrees. */
export function frostCover(temp: number): number {
  return Math.min(1, Math.max(0, -temp / FROST_DEGREES)) * FROST_COVER;
}

export class Ground {
  /** How wet the open ground is and how deep the snow lies on it, 0 to 1. */
  wet = 0;
  snow = 0;
  private primed = false;

  /** Moves both towards what the weather `w` does to them over `dt` seconds; `undefined` (weather off) clears them. */
  update(w: Weather | undefined, dt: number): void {
    if (!w) {
      this.reset();
      return;
    }
    dt = Math.min(Math.max(dt, 0), 1);
    if (!this.primed) {
      this.primed = true;
      this.wet = w.snow ? 0 : w.precip;
      this.snow = Math.max(w.snow ? 0.7 * w.precip : 0, frostCover(w.temp));
      return;
    }
    const raining = w.precip > FALLING && !w.snow, snowing = w.precip > FALLING && w.snow;
    this.wet = raining ? Math.min(1, this.wet + WET_RISE * dt) : Math.max(0, this.wet - WET_DRY * dt);
    if (snowing) this.snow = Math.min(1, this.snow + SNOW_RISE * dt);
    else if (w.temp < 0) this.snow = Math.max(this.snow, Math.min(frostCover(w.temp), this.snow + FROST_RISE * dt));
    else this.snow = Math.max(0, this.snow - (raining ? SNOW_MELT_IN_RAIN : SNOW_MELT * Math.max(0.25, w.temp / 4)) * dt);
  }

  reset(): void {
    this.wet = this.snow = 0;
    this.primed = false;
  }
}
