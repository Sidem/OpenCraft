// How wet and how snowed under the open ground looks, as pure maths (no GL; tested in web/test/ground.test.mjs).
// `Ground` follows the drawn weather (`WeatherEase.value`): rain darkens the ground quickly and lets it dry slowly, snow builds a
// cover slowly and melts it more slowly still (faster in rain). Presentation only and local: it is not a function of the world
// like the weather is, so a player who joins in mid-snow sees the cover already part-grown (the first update starts from the
// weather of that moment). To tune: the rates below, in "full changes per second".

import type { Weather } from './weather';

/** Precipitation (0 to 1) above which the ground wets or the snow builds. */
const FALLING = 0.1;
const WET_RISE = 1 / 8;
const WET_DRY = 1 / 120;
const SNOW_RISE = 1 / 60;
const SNOW_MELT = 1 / 240;
const SNOW_MELT_IN_RAIN = 1 / 40;

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
      this.snow = w.snow ? 0.7 * w.precip : 0;
      return;
    }
    const raining = w.precip > FALLING && !w.snow, snowing = w.precip > FALLING && w.snow;
    this.wet = raining ? Math.min(1, this.wet + WET_RISE * dt) : Math.max(0, this.wet - WET_DRY * dt);
    if (snowing) this.snow = Math.min(1, this.snow + SNOW_RISE * dt);
    else this.snow = Math.max(0, this.snow - (raining ? SNOW_MELT_IN_RAIN : SNOW_MELT) * dt);
  }

  reset(): void {
    this.wet = this.snow = 0;
    this.primed = false;
  }
}
