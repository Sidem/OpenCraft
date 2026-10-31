// Tests for the note sign's pure helpers, run with `node --test web/test`: colours round-trip through the wheel's hue,
// saturation and value; a point on the wheel and its colour agree; a line is cut to the bytes a sign holds at a whole
// character; and the face layout shrinks long lines and centres the lines in use.

import assert from 'node:assert/strict';
import test from 'node:test';
import { fitBytes, hex, hsvToRgb, rgbToHsv, wheelPick, wheelPoint } from '../src/ui/colour.ts';
import { layoutSign } from '../src/render/sign-layout.ts';

test('colours round-trip through hue, saturation and value', () => {
  for (const rgb of [0x000000, 0xffffff, 0xff0000, 0x00ff00, 0x0000ff, 0xe6d3a3, 0x2b2118, 0x123456, 0x808080]) {
    assert.equal(hsvToRgb(...rgbToHsv(rgb)), rgb, hex(rgb));
  }
  assert.equal(hex(0x0a0b0c), '#0a0b0c');
  assert.equal(hsvToRgb(1 / 3, 1, 1), 0x00ff00, 'a third of the way round is green');
});

test('a point on the wheel gives back its hue and saturation', () => {
  for (const [h, s] of [[0, 1], [0.25, 0.5], [0.6, 0.9], [0.99, 0.1]]) {
    const [x, y] = wheelPoint(h, s, 80);
    const [h2, s2] = wheelPick(x, y, 80);
    assert.ok(Math.abs(h2 - h) < 1e-9 && Math.abs(s2 - s) < 1e-9, `${h} ${s}`);
  }
  assert.equal(wheelPick(200, 0, 80)[1], 1, 'past the rim is the rim');
});

test('a line is cut to whole characters within the bytes a sign holds', () => {
  assert.equal(fitBytes('ä'.repeat(20), 32), 'ä'.repeat(16));
  assert.equal(fitBytes('abc😀', 6), 'abc', 'a four-byte character does not split');
  assert.equal(fitBytes('a\tb\nc', 32), 'abc', 'no control characters');
});

test('the face layout shrinks long lines and centres the lines in use', () => {
  const measure = (text, size) => text.length * size * 0.5;
  const short = layoutSign(['Kiln', '', '', ''], 4, 256, 128, measure);
  const long = layoutSign(['A line far too long for the board', '', '', ''], 4, 256, 128, measure);
  assert.ok(long.size < short.size, 'smaller letters for a long line');
  assert.ok(long.size * 0.5 * 33 <= 256, 'and it fits');
  assert.deepEqual(short.middles, [64], 'one line sits in the middle');
  const full = layoutSign(['a', 'b', 'c', 'd'], 4, 256, 128, measure);
  assert.equal(full.middles.length, 4);
  assert.ok(full.middles[0] < full.middles[3] && full.middles[0] > 0 && full.middles[3] < 128);
});
