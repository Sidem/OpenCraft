// Tests for the rates printed on recipe cards (web/src/ui/recipe-rates.ts), run with `node --test`.

import assert from 'node:assert/strict';
import test from 'node:test';
import { formatRate, formatSeconds, perMinute, perSecond, rateWords } from '../src/ui/recipe-rates.ts';

test('a batch time reads as seconds or minutes', () => {
  assert.equal(formatSeconds(4), '4 s');
  assert.equal(formatSeconds(1.5), '1.5 s');
  assert.equal(formatSeconds(90), '1 min 30 s');
  assert.equal(formatSeconds(120), '2 min');
});

test('a smelter making one ingot every 1.5 s makes 40 a minute and 0.67 a second', () => {
  assert.equal(perMinute(1, 1.5), '40/min');
  assert.equal(perSecond(1, 1.5), '0.67/s');
  assert.equal(rateWords(1, 1.5), '40/min · 0.67/s');
});

test('a faster machine reads as a higher rate, not the Mk1 one', () => {
  assert.equal(perMinute(1, 0.75), '80/min');
  assert.equal(perSecond(1, 0.75), '1.33/s');
});

test('rates are short: whole when large, trimmed zeros, a floor for tiny ones', () => {
  assert.equal(formatRate(300), '300');
  assert.equal(formatRate(37.5), '37.5');
  assert.equal(formatRate(12), '12');
  assert.equal(formatRate(0.5), '0.5');
  assert.equal(formatRate(0.001), '<0.01');
  assert.equal(formatRate(0), '0');
  assert.equal(perMinute(2, 4), '30/min');
});
