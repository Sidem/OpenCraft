// Tests for the wet and snowy ground (web/src/render/sky/ground.ts), run with `node --test web/test`: rain wets the ground
// fast and it dries slowly, snow builds slowly and melts slower still (faster in rain), and with the weather off it is all clear.

import assert from 'node:assert/strict';
import test from 'node:test';
import { Ground } from '../src/render/sky/ground.ts';

const RAIN = { cover: 1, gloom: 1, precip: 1, snow: false };
const SNOW = { cover: 1, gloom: 1, precip: 1, snow: true };
const CLEAR = { cover: 0.1, gloom: 0, precip: 0, snow: false };

/** Steps `g` through `seconds` of weather `w` at 10 steps a second. */
function run(g, w, seconds) {
  for (let i = 0; i < seconds * 10; i++) g.update(w, 0.1);
}

test('the first update starts from the weather of that moment', () => {
  const g = new Ground();
  g.update(SNOW, 0.1);
  assert.ok(g.snow > 0.5 && g.wet === 0, `joined in snow: ${g.snow}`);
  const dry = new Ground();
  dry.update(CLEAR, 0.1);
  assert.deepEqual([dry.wet, dry.snow], [0, 0]);
});

test('rain wets the ground within seconds and it dries over minutes', () => {
  const g = new Ground();
  g.update(CLEAR, 0.1);
  run(g, RAIN, 10);
  assert.ok(g.wet > 0.7, `10 s of rain: ${g.wet}`);
  run(g, CLEAR, 20);
  assert.ok(g.wet > 0.5, `still damp 20 s after: ${g.wet}`);
  run(g, CLEAR, 130);
  assert.equal(g.wet, 0, 'dry after minutes');
  assert.equal(g.snow, 0, 'rain makes no snow');
});

test('snow builds slowly, lingers, and melts faster in rain', () => {
  const g = new Ground();
  g.update(CLEAR, 0.1);
  run(g, SNOW, 30);
  assert.ok(g.snow > 0.3 && g.snow < 0.7, `30 s of snow: ${g.snow}`);
  run(g, SNOW, 40);
  assert.equal(g.snow, 1, 'a full cover');
  assert.equal(g.wet, 0, 'snow does not wet the ground');
  const keep = new Ground(), rainy = new Ground();
  for (const x of [keep, rainy]) { x.update(CLEAR, 0.1); run(x, SNOW, 80); }
  run(keep, CLEAR, 60);
  run(rainy, RAIN, 60);
  assert.ok(keep.snow > 0.7, `melts slowly in dry weather: ${keep.snow}`);
  assert.ok(rainy.snow < keep.snow - 0.5, `rain melts it: ${rainy.snow} against ${keep.snow}`);
});

test('weather off clears everything, and a long frame cannot jump the ground', () => {
  const g = new Ground();
  g.update(CLEAR, 0.1);
  run(g, RAIN, 20);
  g.update(undefined, 0.1);
  assert.deepEqual([g.wet, g.snow], [0, 0]);
  g.update(CLEAR, 0.1);
  g.update(RAIN, 1000);
  assert.ok(g.wet <= 0.2, `one 1000 s frame counts as at most a second: ${g.wet}`);
});
