// Tests for trimming a flat icon's picture to its visible pixels (web/src/ui/icon-sprite.ts).

import assert from 'node:assert/strict';
import test from 'node:test';
import { opaqueBounds } from '../src/ui/icon-sprite.ts';

/** An RGBA image `w` by `h`, see-through except the listed pixels at the given alpha. */
function image(w, h, pixels) {
  const data = new Uint8ClampedArray(w * h * 4);
  for (const [x, y, a] of pixels) data[(y * w + x) * 4 + 3] = a;
  return data;
}

test('the bounds hold every visible pixel and nothing more', () => {
  assert.deepEqual(opaqueBounds(image(8, 8, [[2, 1, 255], [5, 6, 200], [3, 3, 255]]), 8, 8), [2, 1, 4, 6]);
});

test('faint pixels do not count, and an image with none visible has no bounds', () => {
  assert.deepEqual(opaqueBounds(image(8, 8, [[0, 0, 10], [4, 4, 255]]), 8, 8), [4, 4, 1, 1]);
  assert.equal(opaqueBounds(image(4, 4, [[1, 1, 5]]), 4, 4), null);
});
