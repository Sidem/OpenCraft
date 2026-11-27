// Tests for the weather's numbers as the UI shows them (web/src/ui/weather-scale.ts): parsing the engine's map samples,
// the grid step per zoom, the layers' colours and wind as a compass word.

import assert from 'node:assert/strict';
import test from 'node:test';
import { compassPoint, gridStep, LAYERS, parseSamples, sampleFill, windFrom } from '../src/ui/weather-scale.ts';

const sample = (over = {}) => ({
  x: 0, z: 0, temp: 12, precip: 0, cover: 0, snow: false, pressure: 1013, east: 0, south: 0, humidity: 60, ...over,
});

test('samples parse in the engine order, whole points only', () => {
  const raw = [64, -128, 12.5, 0.4, 0.9, 1, 1004, 3, -4, 88, 0, 0];
  const [s, ...rest] = parseSamples(raw, 10);
  assert.equal(rest.length, 0, 'a broken tail is dropped');
  assert.deepEqual(s, { x: 64, z: -128, temp: 12.5, precip: 0.4, cover: 0.9, snow: true, pressure: 1004, east: 3, south: -4, humidity: 88 });
});

test('the grid step is a power of two of at least 8 blocks, about the asked pixels apart', () => {
  assert.equal(gridStep(0.25, 14), 8);
  assert.equal(gridStep(1, 14), 16);
  assert.equal(gridStep(8, 14), 128);
  for (const bpp of [0.5, 2, 4, 16]) {
    const s = gridStep(bpp, 14);
    assert.ok(s / bpp >= 14 && s / bpp < 28 || s === 8, `${bpp}: ${s}`);
  }
});

test('every layer but off colours what it measures; clear dry air leaves the rain layer empty', () => {
  for (const l of LAYERS) assert.equal(sampleFill(l.key, sample()) === null, l.key === 'off' || l.key === 'rain', l.key);
  const rain = sampleFill('rain', sample({ precip: 0.5 })), snow = sampleFill('rain', sample({ precip: 0.5, snow: true }));
  assert.notEqual(rain.colour, snow.colour, 'snow looks unlike rain');
  assert.ok(sampleFill('rain', sample({ cover: 0.8 })).alpha > sampleFill('rain', sample({ cover: 0.2 })).alpha);
  assert.notEqual(sampleFill('pressure', sample({ pressure: 990 })).colour, sampleFill('pressure', sample({ pressure: 1035 })).colour);
});

test('wind says where it comes from, opposite where it blows; north is -z', () => {
  assert.equal(windFrom(0, 5), 'north', 'blowing south comes from the north');
  assert.equal(windFrom(5, 0), 'west');
  assert.equal(windFrom(-3, -3), 'south-east');
  assert.equal(windFrom(0.1, 0.1), '', 'calm');
  assert.equal(compassPoint(0), 'north');
  assert.equal(compassPoint(-90), 'west');
  assert.equal(compassPoint(337), 'north-west');
});
