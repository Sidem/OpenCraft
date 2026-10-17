// Tests for the weather model (web/src/render/sky/weather.ts), run with `node --test web/test`: it must be the same for every
// peer (a pure function of seed, time and biome), clear most of the time, stormy every few days, and snow only where it is cold.

import assert from 'node:assert/strict';
import test from 'node:test';
import { YEAR_DAYS as ASTRO_YEAR } from '../src/render/sky/astro.ts';
import { kindOf, storminess, temperature, weatherAt, WeatherEase, WeatherSky, YEAR_DAYS } from '../src/render/sky/weather.ts';

const PLAINS = 0, DESERT = 1, HIGHLANDS = 2, LOWLANDS = 3;
const SEEDS = [1, 42, 12345, 987654321, 0xdeadbeef];
const STEP = 0.01; // a game day is 20 minutes: this samples every 12 seconds

/** Share of time each kind shows over 1,500 game days and every seed. */
function shares(biome, from = 0, to = 1500) {
  const n = { clear: 0, cloudy: 0, gloomy: 0, rain: 0, snow: 0 };
  let total = 0;
  for (const seed of SEEDS) for (let d = from; d < to; d += STEP) { n[kindOf(weatherAt(seed, d, biome))]++; total++; }
  return Object.fromEntries(Object.entries(n).map(([k, v]) => [k, v / total]));
}

test('the year is the stars\' year', () => {
  assert.equal(YEAR_DAYS, ASTRO_YEAR);
});

test('the weather is a pure function of seed, time and biome', () => {
  for (const seed of SEEDS) for (const d of [0, 0.5, 7.3, 123.456, 9999.9]) for (const b of [0, 1, 2, 3, 4]) {
    assert.deepEqual(weatherAt(seed, d, b), weatherAt(seed, d, b));
  }
  // Order of calls does not matter (no hidden state): ask for a late time first, then an early one.
  const late = weatherAt(7, 500.25, PLAINS);
  const early = weatherAt(7, 3.1, PLAINS);
  assert.deepEqual(weatherAt(7, 3.1, PLAINS), early);
  assert.deepEqual(weatherAt(7, 500.25, PLAINS), late);
});

test('different worlds have different weather', () => {
  let differ = 0;
  for (let d = 0; d < 200; d += 0.5) if (Math.abs(storminess(1, d) - storminess(2, d)) > 0.05) differ++;
  assert.ok(differ > 150, `only ${differ} of 400 samples differ`);
});

test('plains are clear most of the time, with real storms', () => {
  const s = shares(PLAINS);
  assert.ok(s.clear > 0.55 && s.clear < 0.8, `clear ${s.clear}`);
  assert.ok(s.rain + s.snow > 0.06 && s.rain + s.snow < 0.2, `precipitation ${s.rain + s.snow}`);
  assert.ok(s.gloomy + s.rain + s.snow > 0.1, `gloomy or worse ${s.gloomy + s.rain + s.snow}`);
});

test('a stormy stretch comes every few days', () => {
  // Over ten-day windows: how many hold a gloomy or wet spell, and how long the quiet gaps between rains are.
  for (const seed of SEEDS) {
    let windows = 0, stormy = 0;
    for (let from = 0; from < 1000; from += 10) {
      windows++;
      let hit = false;
      for (let d = from; d < from + 10 && !hit; d += STEP) hit = weatherAt(seed, d, PLAINS).gloom > 0.4;
      if (hit) stormy++;
    }
    assert.ok(stormy / windows > 0.6, `seed ${seed}: ${stormy} of ${windows} ten-day windows have a gloomy spell`);
    let spells = 0, wet = false;
    for (let d = 0; d < 1000; d += STEP) {
      const now = weatherAt(seed, d, PLAINS).precip > 0.15;
      if (now && !wet) spells++;
      wet = now;
    }
    const gap = 1000 / spells;
    assert.ok(gap > 2 && gap < 14, `seed ${seed}: a rainy spell every ${gap.toFixed(1)} days`);
  }
});

test('a storm builds and passes: no jumps from one moment to the next', () => {
  for (const seed of SEEDS) {
    let prev = weatherAt(seed, 0, PLAINS);
    for (let d = STEP; d < 500; d += STEP) {
      const w = weatherAt(seed, d, PLAINS);
      // At most about a tenth in 12 seconds of play (the drawn weather is eased over a further few seconds).
      assert.ok(Math.abs(w.cover - prev.cover) < 0.13 && Math.abs(w.gloom - prev.gloom) < 0.13 && Math.abs(w.precip - prev.precip) < 0.13, `jump at ${d}`);
      prev = w;
    }
  }
});

test('biomes: deserts are dry and never snowed on, lowlands are wet, highlands get the snow', () => {
  const desert = shares(DESERT), plains = shares(PLAINS), lowlands = shares(LOWLANDS), highlands = shares(HIGHLANDS);
  assert.equal(desert.snow, 0);
  assert.ok(desert.rain < plains.rain / 2, `desert rain ${desert.rain} against plains ${plains.rain}`);
  assert.ok(desert.clear > plains.clear);
  assert.ok(lowlands.rain + lowlands.snow > plains.rain + plains.snow);
  assert.ok(highlands.snow > plains.snow * 2, `highlands ${highlands.snow} snow against plains ${plains.snow}`);
});

test('precipitation is snow exactly when the place is below freezing', () => {
  for (const b of [0, 1, 2, 3, 4]) {
    for (let d = 0; d < 3 * YEAR_DAYS; d += 0.37) {
      assert.equal(weatherAt(5, d, b).snow, temperature(b, d) < 0);
    }
  }
  // Plains: warm in the first half of the year, snowy only deep in the cold half.
  assert.ok(temperature(PLAINS, YEAR_DAYS / 4) > 0.5, 'high summer');
  assert.ok(temperature(PLAINS, (3 * YEAR_DAYS) / 4) < 0, 'midwinter');
  assert.ok(temperature(PLAINS, YEAR_DAYS / 2 + 1) > 0, 'early autumn is not snowy');
  assert.ok(temperature(DESERT, (3 * YEAR_DAYS) / 4) > 0, 'deserts do not freeze');
});

test('the clear state keeps a few small clouds and no gloom', () => {
  for (let d = 0; d < 300; d += 0.05) {
    const w = weatherAt(11, d, PLAINS);
    if (kindOf(w) === 'clear') assert.ok(w.gloom < 0.4 && w.precip < 0.15 && w.cover >= 0.1);
    assert.ok(w.cover >= 0 && w.cover <= 1 && w.gloom >= 0 && w.gloom <= 1 && w.precip >= 0 && w.precip <= 1);
    if (w.precip > 0) assert.ok(w.gloom >= w.precip && w.cover >= w.gloom, 'rain comes from heavy cloud');
  }
});

test('Off draws no look at all (today\'s plain sky); red dusk alone draws a clear one', () => {
  const sky = new WeatherSky();
  assert.equal(sky.look({ weather: 0, dusk: false }, 1, 5.5, 0, 1000), undefined);
  assert.deepEqual(sky.look({ weather: 0, dusk: true }, 1, 5.5, 0, 1100), { cover: 0, gloom: 0, precip: 0, dusk: true, clouds: 0 });
  const full = sky.look({ weather: 2, dusk: false }, 1, 5.5, 0, 1200);
  const model = weatherAt(1, 5.5, 0);
  assert.equal(full.clouds, 2);
  assert.equal(full.cover, model.cover, 'snaps to the model on the first frame after being off');
  assert.equal(sky.look({ weather: 1, dusk: true }, 1, 5.5, 0, 1300).clouds, 1);
});

test('the ease snaps the first time, then follows the target over a few seconds', () => {
  const e = new WeatherEase();
  const storm = { cover: 1, gloom: 1, precip: 1, snow: true };
  e.update(storm, 0.016);
  assert.equal(e.value.gloom, 1);
  assert.equal(e.value.snow, true);
  const clear = { cover: 0.12, gloom: 0, precip: 0, snow: false };
  e.update(clear, 1);
  assert.ok(e.value.gloom > 0.5 && e.value.gloom < 0.8, `after 1 s: ${e.value.gloom}`);
  for (let i = 0; i < 20; i++) e.update(clear, 1);
  assert.ok(e.value.gloom < 0.01);
  e.reset();
  e.update(storm, 0.016);
  assert.equal(e.value.gloom, 1, 'snaps again after a reset');
});
