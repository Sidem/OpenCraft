// Tests for how the ambient beds add up (web/src/audio/ambience-mix.ts), run with `node --test web/test`.

import assert from 'node:assert/strict';
import test from 'node:test';
import { BEDS, bedGains, CEILING, WATER, WATER_CEILING } from '../src/audio/ambience-mix.ts';

const NAMES = Object.keys(BEDS);
const loudness = (g, names = NAMES) => Math.sqrt(names.reduce((sum, n) => sum + (g[n] * BEDS[n].loud) ** 2, 0));
const levels = (over) => ({ wind: 0, surf: 0, river: 0, rain: 0, hum: 0, ...over });

test('a bed alone is never louder than the ceiling and silence stays silent', () => {
  for (const n of NAMES) assert.ok(loudness(bedGains(levels({ [n]: 1 }))) <= CEILING + 1e-9, n);
  assert.deepEqual(bedGains(levels({})), { wind: 0, surf: 0, river: 0, rain: 0, hum: 0 });
});

test('every combination of levels stays under the ceiling, and the water under its own', () => {
  const steps = [0, 0.25, 0.5, 0.75, 1];
  for (const wind of steps) for (const surf of steps) for (const river of steps) for (const hum of steps) for (const rain of [0, 1]) {
    const g = bedGains(levels({ wind, surf, river, rain, hum }));
    assert.ok(loudness(g) <= CEILING + 1e-9, `${wind} ${surf} ${river} ${rain} ${hum}`);
    assert.ok(loudness(g, WATER) <= WATER_CEILING + 1e-9, `water ${surf} ${river}`);
  }
});

test('a river or the sea alone is far quieter than a full wind bed', () => {
  assert.ok(loudness(bedGains(levels({ river: 1 }))) <= WATER_CEILING + 1e-9);
  assert.ok(loudness(bedGains(levels({ surf: 1 }))) <= WATER_CEILING + 1e-9);
  assert.ok(WATER_CEILING < CEILING / 2);
});

test('water piled up is turned down together and keeps its proportions', () => {
  const alone = bedGains(levels({ river: 1 }));
  const piled = bedGains(levels({ surf: 1, river: 1 }));
  assert.ok(piled.river < alone.river, 'the river gives way to the sea');
  assert.ok(Math.abs(piled.river / piled.surf - BEDS.river.peak / BEDS.surf.peak) < 1e-9);
});

test('beds piled up are turned down together past the ceiling', () => {
  const alone = bedGains(levels({ wind: 1 }));
  const piled = bedGains(levels({ wind: 1, rain: 1, hum: 1 }));
  assert.ok(piled.wind < alone.wind, 'the wind gives way to the rain');
  assert.ok(Math.abs(piled.rain / piled.wind - BEDS.rain.peak / BEDS.wind.peak) < 1e-9);
});

test('quiet readings are left alone', () => {
  const g = bedGains(levels({ wind: 0.3, river: 0.1 }));
  assert.equal(g.wind, 0.3 * BEDS.wind.peak);
  assert.equal(g.river, 0.1 * BEDS.river.peak);
});
