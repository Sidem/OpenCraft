// Tests for the icon check's measures (web/src/ui/icon-check.ts): outline overlap, whole-icon mean and the most
// different patch, on small synthetic icons.

import assert from 'node:assert/strict';
import test from 'node:test';
import { compareIcons, PATCH_MIN, SIZE } from '../src/ui/icon-check.ts';

/** A SIZE×SIZE icon: an opaque grey square from `lo` to `hi`, then `paint(x, y)` may recolour a pixel. */
function icon(lo, hi, paint = () => null) {
  const px = new Uint8ClampedArray(SIZE * SIZE * 4);
  for (let y = 0; y < SIZE; y++) {
    for (let x = 0; x < SIZE; x++) {
      if (x < lo || x >= hi || y < lo || y >= hi) continue;
      const rgb = paint(x, y) ?? [128, 128, 128];
      px.set([...rgb, 255], (y * SIZE + x) * 4);
    }
  }
  return px;
}

test('identical icons share the whole outline and differ by nothing', () => {
  const c = compareIcons(icon(4, 28), icon(4, 28), SIZE);
  assert.deepEqual(c, { overlap: 1, mean: 0, patch: 0 });
});

test('overlap is the intersection over the union of the silhouettes', () => {
  const c = compareIcons(icon(0, 16), icon(0, 32), SIZE);
  assert.equal(c.overlap, 0.25);
});

test('a corner chip that differs completely passes the patch measure while the whole-icon mean stays low', () => {
  const chip = (rgb) => (x, y) => (x < 12 && y >= 20 ? rgb : null);
  const c = compareIcons(icon(0, 32, chip([217, 74, 61])), icon(0, 32, chip([76, 175, 80])), SIZE);
  assert.equal(c.overlap, 1);
  assert.ok(c.patch >= PATCH_MIN, `patch ${c.patch}`);
  assert.ok(c.mean < PATCH_MIN, `mean ${c.mean}`);
});

test('see-through pixels count as the slot colour, not as black', () => {
  const clear = new Uint8ClampedArray(SIZE * SIZE * 4);
  const faint = clear.map((v, i) => (i % 4 === 3 ? 10 : 0));
  assert.ok(compareIcons(clear, faint, SIZE).mean < 2);
});
