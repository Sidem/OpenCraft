// Tests for the words on the Home page (web/src/ui/home-text.ts, greetings.ts) and the tip split (tip-text.ts), run with
// `node --test`: time played, research progress, the greeting lines, and which part of a tip is its title.

import assert from 'node:assert/strict';
import test from 'node:test';
import { GREETINGS, greeting } from '../src/ui/greetings.ts';
import { playTimeText, researchProgress, waitingText } from '../src/ui/home-text.ts';
import { tipParts } from '../src/ui/tip-text.ts';

test('time played reads in the two units that matter', () => {
  assert.equal(playTimeText(0), 'Under a minute');
  assert.equal(playTimeText(59), 'Under a minute');
  assert.equal(playTimeText(60), '1 min');
  assert.equal(playTimeText(42 * 60 + 30), '42 min');
  assert.equal(playTimeText(3600), '1 h');
  assert.equal(playTimeText(5 * 3600 + 12 * 60), '5 h 12 min');
  assert.equal(playTimeText(31 * 3600 + 20 * 60), '31 h', 'ten hours or more drop the minutes');
  assert.equal(playTimeText(-5), 'Under a minute');
});

test('research progress counts units, or says the level of an endless tech', () => {
  assert.equal(researchProgress(3, 5, false), '3 of 5');
  assert.equal(researchProgress(9, 5, false), '5 of 5', 'never past the total');
  assert.equal(researchProgress(3, 0, true), 'level 4');
  assert.equal(waitingText(0), '');
  assert.equal(waitingText(4), '4 more waiting');
});

test('each greeting has a title and lead lines; the plain line comes first and a random number picks one', () => {
  for (const [kind, { title, leads }] of Object.entries(GREETINGS)) {
    assert.ok(title, kind);
    assert.equal(new Set(leads).size, leads.length, `${kind}: no repeated line`);
  }
  assert.deepEqual(greeting('first'), ['Welcome to OpenCraft', '']);
  assert.equal(greeting('paused', 0)[1], 'Take your time. Press Esc or Resume when you are ready.');
  const last = GREETINGS.back.leads.at(-1);
  assert.equal(greeting('back', 0.999999)[1], last);
  assert.equal(greeting('back', 1)[1], last, 'never past the end');
  const seen = new Set(Array.from({ length: 40 }, (_, i) => greeting('paused', i / 40)[1]));
  assert.equal(seen.size, GREETINGS.paused.leads.length, 'every line can come up');
});

test('a short lead-in before a colon is the title; anything else is all body', () => {
  assert.deepEqual(tipParts('Mine: break a block'), { title: 'Mine', body: 'Break a block' });
  assert.deepEqual(tipParts('Just a sentence with no lead-in'), { title: '', body: 'Just a sentence with no lead-in' });
  assert.equal(tipParts('Press *E*: opens things').title, '', 'a lead-in with markup is not a title');
  assert.equal(tipParts(`${'x'.repeat(41)}: too long to be a title`).title, '');
});
