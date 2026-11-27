// Tests for the drawn weather (web/src/render/sky/weather.ts), run with `node --test web/test`: the engine's numbers read
// right, the kinds and forced weathers, and the ease. The weather itself is the engine's (crates/engine/src/weather/tests.rs).

import assert from 'node:assert/strict';
import test from 'node:test';
import { CLEAR, HELD_SNOW_TEMP, heldIn, kindOf, weatherFrom, weatherOf, WeatherEase, WeatherSky } from '../src/render/sky/weather.ts';

test('the engine\'s numbers read as a weather', () => {
  assert.deepEqual(weatherFrom([0.9, 0.8, 0.7, 1, -3.5]), { cover: 0.9, gloom: 0.8, precip: 0.7, snow: true, temp: -3.5 });
  assert.equal(weatherFrom([0.4, 0, 0, 0, 20]).snow, false);
});

test('kinds: a clear sky keeps fair clouds, rain or snow by the flag', () => {
  assert.equal(kindOf(CLEAR), 'clear');
  for (const k of ['clear', 'cloudy', 'gloomy', 'rain', 'snow']) assert.equal(kindOf(weatherOf(k)), k);
  assert.ok(weatherOf('snow').temp < 0 && weatherOf('rain').temp > 0, 'a forced snow is cold, a forced rain is not');
});

test('Off draws no look at all (today\'s plain sky); red dusk alone draws a clear one', () => {
  const sky = new WeatherSky();
  const model = weatherOf('cloudy');
  assert.equal(sky.look({ weather: 0, dusk: false }, model, 1000), undefined);
  assert.deepEqual(sky.look({ weather: 0, dusk: true }, model, 1100), { cover: 0, gloom: 0, precip: 0, dusk: true, clouds: 0 });
  const full = sky.look({ weather: 2, dusk: false }, model, 1200);
  assert.equal(full.clouds, 2);
  assert.equal(full.cover, model.cover, 'snaps to the model on the first frame after being off');
  assert.equal(sky.look({ weather: 1, dusk: true }, model, 1300).clouds, 1);
  sky.forced = weatherOf('snow');
  for (let t = 1400; t < 60_000; t += 500) sky.look({ weather: 2, dusk: false }, model, t);
  assert.ok(sky.ease.value.snow && sky.ease.value.precip > 0.9, 'a forced weather wins over the engine\'s');
});

test('the ease snaps the first time, then follows the target over a few seconds', () => {
  const e = new WeatherEase();
  const storm = { cover: 1, gloom: 1, precip: 1, snow: true, temp: -5 };
  e.update(storm, 0.016);
  assert.equal(e.value.gloom, 1);
  assert.equal(e.value.snow, true);
  const clear = { cover: 0.12, gloom: 0, precip: 0, snow: false, temp: 15 };
  e.update(clear, 1);
  assert.ok(e.value.gloom > 0.5 && e.value.gloom < 0.8, `after 1 s: ${e.value.gloom}`);
  for (let i = 0; i < 20; i++) e.update(clear, 1);
  assert.ok(e.value.gloom < 0.01 && Math.abs(e.value.temp - 15) < 0.1);
  e.reset();
  e.update(storm, 0.016);
  assert.equal(e.value.gloom, 1, 'snaps again after a reset');
});

test('a held weather falls as snow below 0 °C and rain above it, and held snow brings the cold', () => {
  const freezing = { ...CLEAR, temp: -6 }, mild = { ...CLEAR, temp: 14 };
  assert.deepEqual([heldIn(weatherOf('rain'), freezing).snow, heldIn(weatherOf('rain'), mild).snow], [true, false]);
  assert.equal(heldIn(weatherOf('rain'), mild).temp, 14, 'rain keeps the real temperature');
  const snow = heldIn(weatherOf('snow'), mild);
  assert.deepEqual([snow.snow, snow.temp], [true, HELD_SNOW_TEMP]);
  assert.equal(heldIn(weatherOf('snow'), freezing).temp, -6, 'colder than the held cold stays colder');
  assert.equal(heldIn(weatherOf('clear'), freezing).temp, -6, 'a held clear sky does not warm a winter');
});

test('holding a weather snaps the sky to it at once', () => {
  const sky = new WeatherSky(), settings = { weather: 2, dusk: false };
  sky.look(settings, { ...CLEAR, temp: 10 }, 0);
  sky.look(settings, { ...CLEAR, temp: 10 }, 1000);
  sky.hold(weatherOf('snow'));
  const look = sky.look(settings, { ...CLEAR, temp: 10 }, 1016);
  assert.equal(look.precip, 1, 'full snow on the next frame, not after an ease');
  assert.equal(sky.ease.value.snow, true);
});