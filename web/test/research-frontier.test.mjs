import test from 'node:test';
import assert from 'node:assert/strict';
import { ancestors, autoWide, bucket, chainLinks, dependents, discover, MAX_WIDE, visibleRings } from '../src/ui/research-frontier.ts';

// A small tree: 0 and 1 in ring 0; 2 (needs 0) and 3 (needs 2) and 4 (needs 1) in ring 1; 5 (needs 3) in ring 2.
const needs = [[], [], [0], [2], [1], [3]];
const rings = [0, 0, 1, 1, 1, 2];
const fact = (t, done, available, planned = false) => ({ done, available, planned, ring: rings[t], needs: needs[t] });

test('bucket puts each tech in its cell, prerequisites first', () => {
  const cells = bucket(needs, rings, [0, 1, 0, 0, 1, 0], 3, 2);
  assert.deepEqual(cells[0][1], [2, 3]);
  assert.deepEqual(cells[1][1], [4]);
  assert.deepEqual(cells[0][2], [5]);
  const reversed = bucket([[], [], [3], [], [], []], [1, 1, 1, 1, 1, 1], [0, 0, 0, 0, 0, 0], 2, 1);
  assert.ok(reversed[0][1].indexOf(3) < reversed[0][1].indexOf(2), 'a need sorts before what needs it');
});

test('discover shows done and available, teases one step ahead, hides the rest', () => {
  const facts = [fact(0, true, false), fact(1, false, true), fact(2, false, true), fact(3, false, false), fact(4, false, false), fact(5, false, false)];
  const seen = discover(facts, [true, true, false]);
  assert.deepEqual(seen, ['shown', 'shown', 'shown', 'next', 'next', 'hidden']);
});

test('discover hides a closed ring, even its techs with no needs', () => {
  const facts = [fact(0, false, true), fact(1, false, true), fact(2, false, false), fact(3, false, false), fact(4, false, false), fact(5, false, false)];
  assert.deepEqual(discover(facts, [true, false, false]).slice(2), ['hidden', 'hidden', 'hidden', 'hidden']);
});

test('a queued tech shows, and so does the step after it', () => {
  const facts = [fact(0, true, false), fact(1, true, false), fact(2, false, true), fact(3, false, false, true), fact(4, true, false), fact(5, false, false)];
  const seen = discover(facts, [true, true, true]);
  assert.equal(seen[3], 'shown');
  assert.equal(seen[5], 'next');
});

test('visibleRings stops at the first closed ring', () => {
  assert.deepEqual(visibleRings([true, false, false, false]), [0, 1]);
  assert.deepEqual(visibleRings([true, true, true]), [0, 1, 2]);
});

test('autoWide opens busy rings newest first, then the gate, else the newest ring', () => {
  assert.deepEqual([...autoWide([true, false, false], [2, 15, 16])].sort(), [0, 1]);
  assert.deepEqual([...autoWide([true, true, false], [0, 3, 16])].sort(), [1, 2]);
  assert.deepEqual([...autoWide([true, true, true, false], [0, 3, 2, 16])].sort(), [1, 2], 'busy rings before the gate');
  assert.deepEqual([...autoWide([true, true, true, true], [0, 0, 0, 0])], [3]);
  const many = autoWide([true, true, true, true, true, false], [1, 1, 1, 1, 1, 9]);
  assert.deepEqual([...many], [4, 3, 2, 1, 0].slice(0, MAX_WIDE), 'the newest busy rings, no room left for the gate');
});

test('ancestors, dependents and chain links', () => {
  assert.deepEqual([...ancestors(needs, 5)].sort(), [0, 2, 3]);
  assert.deepEqual(dependents(needs, 2), [3]);
  const links = chainLinks(needs, 3);
  assert.deepEqual(links.filter((l) => !l.ahead).map((l) => `${l.from}>${l.to}`).sort(), ['0>2', '2>3']);
  assert.deepEqual(links.filter((l) => l.ahead).map((l) => `${l.from}>${l.to}`), ['3>5']);
});
