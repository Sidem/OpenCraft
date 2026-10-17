// Tests for how the ambient beds add up (web/src/audio/ambience-mix.ts), run with `node --test web/test`.

import assert from 'node:assert/strict';
import test from 'node:test';
import { BEDS, bedGains, CEILING } from '../src/audio/ambience-mix.ts';

const NAMES = Object.keys(BEDS);
const loudness = (g) => Math.sqrt(NAMES.reduce((sum, n) => sum + (g[n] * BEDS[n].loud) ** 2, 0));
const levels = (over) => ({ wind: 0, surf: 0, river: 0, rain: 0, hum: 0, ...over });

test('a bed alone is never louder than the ceiling and silence stays silent', () => {
  for (const n of NAMES) assert.ok(loudness(bedGains(levels({ [n]: 1 }))) <= CEILING + 1e-9, n);
  assert.deepEqual(bedGains(levels({})), { wind: 0, surf: 0, river: 0, rain: 0, hum: 0 });
});

test('every combination of levels stays under the ceiling', () => {
  const steps = [0, 0.25, 0.5, 0.75, 1];
  for (const wind of steps) for (const surf of steps) for (const river of steps) for (const hum of steps) for (const rain of [0, 1]) {
    assert.ok(loudness(bedGains(levels({ wind, surf, river, rain, hum }))) <= CEILING + 1e-9, `${wind} ${surf} ${river} ${rain} ${hum}`);
  }
});

test('beds piled up are turned down together and keep their proportions', () => {
  const alone = bedGains(levels({ wind: 1 }));
  const piled = bedGains(levels({ wind: 1, surf: 1, river: 1 }));
  assert.ok(piled.wind < alone.wind, 'the wind gives way to the water');
  assert.ok(Math.abs(piled.river / piled.wind - BEDS.river.peak / BEDS.wind.peak) < 1e-9);
});

test('quiet readings are left alone', () => {
  const g = bedGains(levels({ wind: 0.3, river: 0.2 }));
  assert.equal(g.wind, 0.3 * BEDS.wind.peak);
  assert.equal(g.river, 0.2 * BEDS.river.peak);
});
