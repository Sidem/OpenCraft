// Tests for the blueprint card's picture (web/src/ui/blueprint-plan.ts): fitting a blueprint seen from above into a
// square, so every piece lands inside the picture at the right place and size.

import assert from 'node:assert/strict';
import test from 'node:test';
import { layoutPlan } from '../src/ui/blueprint-plan.ts';

const FIELDS = 8;
/** A 6×2×4 box: a belt running east at the corner and a 2×2 machine at the far end. */
const PLAN = [6, 2, 4, 0, 0, 0, 0, 0, 12, 1, 1, 4, 2, 5, 3, 1, 61, 1, 0];

test('pieces sit at their columns, one cell per block', () => {
  const lay = layoutPlan(PLAN, FIELDS, 100, 100);
  const [belt, machine] = lay.pieces;
  assert.equal(lay.pieces.length, 2);
  assert.deepEqual([belt.x, belt.y, belt.w, belt.h], [lay.ox, lay.oy, lay.cell, lay.cell]);
  assert.deepEqual([machine.x - lay.ox, machine.y - lay.oy, machine.w, machine.h], [4 * lay.cell, 2 * lay.cell, 2 * lay.cell, 2 * lay.cell]);
  assert.deepEqual([belt.flow, belt.facing, machine.flow, machine.item], [true, 1, false, 61]);
});

test('the box is centred and fits, however big', () => {
  for (const [sx, sz] of [[1, 1], [6, 4], [48, 3], [48, 48]]) {
    const lay = layoutPlan([sx, 1, sz], FIELDS, 144, 96);
    assert.ok(lay.cell >= 1 && lay.ox >= 0 && lay.oy >= 0, `${sx}×${sz}`);
    assert.ok(lay.ox + lay.width <= 144 && lay.oy + lay.height <= 96, `${sx}×${sz} fits`);
    assert.ok(Math.abs(lay.ox - (144 - lay.ox - lay.width)) <= 1, `${sx}×${sz} centred across`);
    assert.ok(Math.abs(lay.oy - (96 - lay.oy - lay.height)) <= 1, `${sx}×${sz} centred down`);
  }
});

test('a line uses the wide picture: bigger cells than in a square', () => {
  const line = [9, 2, 4];
  assert.ok(layoutPlan(line, FIELDS, 144, 96).cell > layoutPlan(line, FIELDS, 96, 96).cell);
});

test('a tiny blueprint is not blown up to fill the picture', () => {
  assert.ok(layoutPlan([1, 1, 1], FIELDS, 100, 100).cell < 30);
});
