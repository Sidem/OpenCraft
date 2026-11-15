import test from 'node:test';
import assert from 'node:assert/strict';
import { baseName, bestState, foldFamilies, mkOf, mkRange } from '../src/ui/craft-tiers.ts';

const tile = (name, group, rank, tier, state = 'ready') => ({ name, group, order: rank * 8 + tier, state });

test('the Mks of one building fold into one list, lowest Mk first, where the first one stood', () => {
  const tiles = [
    tile('Belt', 0, 1, 0),
    tile('Miner Mk2', 0, 2, 1),
    tile('Miner', 0, 2, 0),
    tile('Splitter', 0, 3, 0),
    tile('Miner Mk3', 0, 2, 2),
  ];
  const lists = foldFamilies(tiles);
  assert.deepEqual(lists.map((l) => l.map((t) => t.name)), [['Belt'], ['Miner', 'Miner Mk2', 'Miner Mk3'], ['Splitter']]);
});

test('the same rank in another category, and guide tiles, never fold', () => {
  const lists = foldFamilies([tile('A', 0, 1, 0), tile('B', 1, 1, 1), tile('C', 2, 4, 0, 'guide'), tile('D', 2, 4, 1, 'guide')]);
  assert.equal(lists.length, 4);
});

test('names, Mk ranges and the best state', () => {
  assert.equal(baseName('Smelter Mk2'), 'Smelter');
  assert.equal(baseName('Hauler Pack'), 'Hauler Pack');
  assert.equal(baseName('Mk5'), 'Mk5');
  assert.equal(mkOf(tile('x', 3, 2, 4)), 5);
  assert.equal(mkRange([1, 2, 5]), 'Mk1–5');
  assert.equal(mkRange([3]), 'Mk3');
  assert.equal(bestState(['locked', 'short']), 'short');
  assert.equal(bestState(['locked', 'ready', 'short']), 'ready');
  assert.equal(bestState(['locked']), 'locked');
});
