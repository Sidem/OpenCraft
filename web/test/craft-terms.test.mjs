// Tests for the build menu's search ranking (web/src/ui/craft-terms.ts), run with `node --test web/test`: what you
// type finds the item of that name first, before the things made from it.

import assert from 'node:assert/strict';
import test from 'node:test';
import { hasWord, nameScore } from '../src/ui/craft-terms.ts';

const words = (q) => q.toLowerCase().split(/\s+/).filter(Boolean);

test('the item named by the search ranks before what is made from it', () => {
  // A play-tester typed "concrete" and saw the Blast Furnace (made from concrete) first.
  const names = ['Blast Furnace', 'Red Concrete', 'Concrete', 'Concrete Mixer Mk2'];
  const ranked = [...names].sort((a, b) => nameScore(a, words('concrete')) - nameScore(b, words('concrete')));
  assert.deepEqual(ranked, ['Concrete', 'Concrete Mixer Mk2', 'Red Concrete', 'Blast Furnace']);
});

test('scores: the name, its start, its words, anything else', () => {
  assert.equal(nameScore('Concrete', words('Concrete')), 0);
  assert.equal(nameScore('Quicklime', words('quick')), 1);
  assert.equal(nameScore('Miner Mk2', words('mk2 miner')), 2);
  assert.equal(nameScore('Belt Lift', words('belts')), 2, 'a plural finds the singular');
  assert.equal(nameScore('Blast Furnace', words('concrete')), 3);
  assert.equal(nameScore('Concrete', []), 3, 'no search ranks nothing');
});

test('short words must be whole words', () => {
  assert.ok(hasWord('ai lab', 'ai'));
  assert.ok(!hasWord('air filter', 'ai'));
});
