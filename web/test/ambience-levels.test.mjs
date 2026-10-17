// Tests for what the ambience plays (web/src/audio/ambience-levels.ts), run with `node --test web/test`: rain is heard in the
// open, muffled-quiet under a roof and not at all underground, snow is silent, and birds and crickets keep quiet in weather.

import assert from 'node:assert/strict';
import test from 'node:test';
import { mixFor } from '../src/audio/ambience-levels.ts';
import { S } from '../src/audio/surroundings.ts';

const NOON = 0.5, NIGHT = 0;
/** A reading: plains, `sky` open, `depth` blocks below the surface, trees around. */
const reading = (over = {}) => {
  const s = new Array(9).fill(0);
  Object.assign(s, { [S.sky]: 1, [S.open]: 1, [S.trees]: 0.6 }, over);
  return s;
};
const RAIN = { precip: 1, snow: false };
const SNOW = { precip: 1, snow: true };

test('no weather, no rain: the mix is what it was before', () => {
  const m = mixFor(reading(), NOON);
  assert.equal(m.rain, 0);
  assert.equal(mixFor(reading(), NOON, { precip: 0, snow: false }).birds, m.birds);
});

test('rain is heard in the open, less under a roof, and not underground', () => {
  const open = mixFor(reading(), NOON, RAIN);
  const roofed = mixFor(reading({ [S.sky]: 0 }), NOON, RAIN);
  const cave = mixFor(reading({ [S.sky]: 0, [S.depth]: 12 }), NOON, RAIN);
  assert.equal(open.rain, 1);
  assert.ok(roofed.rain > 0 && roofed.rain < 0.5 * open.rain, `a patter on the roof: ${roofed.rain}`);
  assert.equal(cave.rain, 0, 'deep underground it is silent');
  assert.equal(roofed.sky, 0, 'the host dulls the rain by the open sky');
});

test('rain scales with how hard it falls', () => {
  const light = mixFor(reading(), NOON, { precip: 0.3, snow: false }).rain;
  const heavy = mixFor(reading(), NOON, RAIN).rain;
  assert.ok(light > 0 && light < heavy, `${light} < ${heavy}`);
});

test('snow falls silent', () => {
  assert.equal(mixFor(reading(), NOON, SNOW).rain, 0);
});

test('birds and crickets keep quiet in weather, and the wind picks up', () => {
  const dry = mixFor(reading(), NOON), wet = mixFor(reading(), NOON, RAIN);
  assert.ok(dry.birds > 0.2 && wet.birds < 0.25 * dry.birds, `birds ${dry.birds} -> ${wet.birds}`);
  const night = mixFor(reading(), NIGHT), nightWet = mixFor(reading(), NIGHT, RAIN);
  assert.ok(night.insects > 0 && nightWet.insects < 0.25 * night.insects, `crickets ${night.insects} -> ${nightWet.insects}`);
  assert.ok(wet.wind > dry.wind, 'a squall has wind in it');
});
