// Tests for the wet and snowy ground (web/src/render/sky/ground.ts), run with `node --test web/test`: rain wets the ground
// fast and it dries slowly, snow builds slowly, lies while it is below zero and melts with the warmth (faster in rain); the cold alone lays a thin cover, and with the weather off it is all clear.

import assert from 'node:assert/strict';
import test from 'node:test';
import { frostCover, Ground } from '../src/render/sky/ground.ts';

const RAIN = { cover: 1, gloom: 1, precip: 1, snow: false, temp: 8 };
const SNOW = { cover: 1, gloom: 1, precip: 1, snow: true, temp: -4 };
const CLEAR = { cover: 0.1, gloom: 0, precip: 0, snow: false, temp: 4 };
const FROST = { ...CLEAR, temp: -6 };

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

test('snow never melts below zero, and melts faster the warmer it is', () => {
  const cold = new Ground(), mild = new Ground(), warm = new Ground();
  for (const x of [cold, mild, warm]) { x.update(CLEAR, 0.1); run(x, SNOW, 80); }
  run(cold, FROST, 120);
  run(mild, CLEAR, 60);
  run(warm, { ...CLEAR, temp: 16 }, 60);
  assert.equal(cold.snow, 1, 'a frosty day keeps the cover');
  assert.ok(warm.snow < mild.snow - 0.3, `16 degrees melts it faster: ${warm.snow} against ${mild.snow}`);
});

test('the cold alone lays a thin cover that deepens with the frost', () => {
  assert.equal(frostCover(2), 0);
  assert.ok(frostCover(-2) > 0 && frostCover(-2) < frostCover(-6) && frostCover(-30) <= 0.6);
  const g = new Ground();
  g.update(FROST, 0.1);
  assert.equal(g.snow, frostCover(-6), 'joined on a frosty day: the cold cover at once');
  const late = new Ground();
  late.update(CLEAR, 0.1);
  run(late, FROST, 30);
  assert.ok(late.snow > 0 && late.snow < frostCover(-6), `builds slowly: ${late.snow}`);
  run(late, FROST, 300);
  assert.equal(late.snow, frostCover(-6), 'up to the cold cover, no further');
});
