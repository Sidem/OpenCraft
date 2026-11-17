// Tests for the tier-colour palettes (crates/engine/src/tier_palette.rs, web/src/render/tier-paint.ts), run with
// `node --test web/test`: the colour-blind palette keeps Mk1 to Mk5 apart under protan, deutan and tritan sight (CIEDE2000, the
// same bar as the control categories), and recolouring a baked layer moves the tier colour and leaves steel and pips alone.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { DEUTAN, PROTAN, TRITAN, deltaE2000, hex, lab, rgb, worstPair } from './colour.mjs';
import { KIT_CASE, STRIPE, STUD_PAINT, TierPaint, parseTierLayers, recolour } from '../src/render/tier-paint.ts';

/** A palette table out of the engine's source, so the test reads the one copy of the numbers. */
function palette(file, name) {
  const source = readFileSync(new URL(`../../crates/engine/src/${file}`, import.meta.url), 'utf8');
  const row = new RegExp(`pub const ${name}: \\[u32; 5\\] = \\[([^\\]]+)\\]`).exec(source);
  assert.ok(row, `${name} in ${file}`);
  const colours = row[1].split(',').map((c) => parseInt(c.trim().replace(/^0x/, ''), 16));
  assert.equal(colours.length, 5);
  return colours;
}
const STANDARD = palette('factory/upgrades.rs', 'TIER_COLOURS');
const COLOUR_BLIND = palette('tier_palette.rs', 'TIER_COLOURS_CB');

const SIGHTS = [['normal', null], ['protan', PROTAN], ['deutan', DEUTAN], ['tritan', TRITAN]];

test('the colour-blind tier colours stay clearly apart for normal sight and for protan, deutan and tritan colour blindness', () => {
  const NEEDED = 15; // CIEDE2000, the bar the control categories use: 10 or more is plain at a glance
  const low = [];
  for (const [name, sim] of SIGHTS) {
    const w = worstPair(COLOUR_BLIND, sim);
    if (w.d < NEEDED) low.push(`${name} Mk${w.i + 1}/Mk${w.j + 1} ${w.d.toFixed(1)}`);
  }
  assert.deepEqual(low, [], `pairs closer than ${NEEDED}`);
});

test('the standard palette is why the setting exists: two of its colours are one for a deutan eye', () => {
  assert.ok(worstPair(STANDARD, DEUTAN).d < 5, 'blue and violet merge');
});

test('the colour-blind palette runs dark to light, so lightness says the Mk as well as hue', () => {
  const L = COLOUR_BLIND.map((c) => lab(rgb(hex(c)))[0]);
  for (let i = 1; i < L.length; i++) assert.ok(L[i] - L[i - 1] >= 6, `Mk${i} ${L[i - 1].toFixed(0)} then Mk${i + 1} ${L[i].toFixed(0)}`);
});

// ---- recolouring a layer

const texel = (c, k = 1) => [(c >> 16) & 255, (c >> 8) & 255, c & 255].map((v) => Math.round(v * k));
/** A 16 × 16 stripe like the engine's: steel above and below, a band of the tier colour, two white pips. */
function stripe(c) {
  const out = new Uint8Array(16 * 16 * 4);
  for (let y = 0; y < 16; y++) {
    for (let x = 0; x < 16; x++) {
      const pip = (y === 7 || y === 8) && x >= 6 && x < 8;
      const px = pip ? [245, 245, 240] : y < 4 || y > 11 ? [122, 128, 136] : texel(c, y === 4 || y === 11 ? 0.6 : 0.95);
      out.set([...px, 255], (y * 16 + x) * 4);
    }
  }
  return out;
}
const at = (layer, x, y) => Array.from(layer.slice((y * 16 + x) * 4, (y * 16 + x) * 4 + 4));
const hue = ([r, g, b]) => { const [, a, bb] = lab(rgb(hex((r << 16) | (g << 8) | b))); return Math.atan2(bb, a); };

test('recolouring a stripe moves its band to the new colour and leaves the steel and the pips alone', () => {
  const from = STANDARD[2], to = COLOUR_BLIND[0];
  const before = stripe(from), after = recolour(before, STRIPE, from, to);
  assert.deepEqual(at(after, 0, 0), at(before, 0, 0), 'steel');
  assert.deepEqual(at(after, 6, 7), at(before, 6, 7), 'a pip');
  assert.equal(after[(5 * 16 + 3) * 4 + 3], 255, 'alpha');
  const band = at(after, 3, 5);
  const wanted = deltaE2000(lab(rgb(hex((band[0] << 16) | (band[1] << 8) | band[2]))), lab(rgb(hex(to))));
  assert.ok(wanted < 25, `the band is near the new colour (ΔE ${wanted.toFixed(1)})`);
  const old = deltaE2000(lab(rgb(hex((band[0] << 16) | (band[1] << 8) | band[2]))), lab(rgb(hex(from))));
  assert.ok(old > wanted, 'and nearer to it than to the old one');
});

test('every tier and kind lands near its new colour, and recolouring to the same palette changes nothing', () => {
  for (let tier = 0; tier < 5; tier++) {
    const layer = stripe(STANDARD[tier]);
    assert.deepEqual(recolour(layer, STRIPE, STANDARD[tier], STANDARD[tier]), layer);
    const moved = at(recolour(layer, KIT_CASE, STANDARD[tier], COLOUR_BLIND[tier]), 3, 6);
    const dh = Math.abs(hue(moved) - hue(texel(COLOUR_BLIND[tier])));
    assert.ok(Math.min(dh, 2 * Math.PI - dh) < 0.35, `Mk${tier + 1} hue`);
  }
});

test('a stud\'s paint stays a pale tint of the new colour', () => {
  const pale = (c) => texel(c).map((v) => Math.round(v * 0.3 + 255 * 0.7));
  const layer = new Uint8Array([...pale(STANDARD[1]), 255]);
  const moved = Array.from(recolour(layer, STUD_PAINT, STANDARD[1], COLOUR_BLIND[1]).slice(0, 3));
  assert.deepEqual(moved, pale(COLOUR_BLIND[1]));
});

test('TierPaint paints from the standard layers every time, so switching back restores them', () => {
  const layers = parseTierLayers([0, 1, STRIPE, 1, 2, KIT_CASE]);
  const size = 16, bytes = size * size * 4;
  const pixels = new Uint8Array(2 * bytes);
  pixels.set(stripe(STANDARD[1]), 0);
  pixels.set(stripe(STANDARD[2]), bytes);
  const tp = new TierPaint(pixels, size, layers);
  const cb = tp.paint(STANDARD, COLOUR_BLIND);
  assert.notDeepEqual(cb.get(0), pixels.slice(0, bytes));
  const back = tp.paint(STANDARD, STANDARD);
  assert.deepEqual(back.get(0), pixels.slice(0, bytes));
  assert.deepEqual(back.get(1), pixels.slice(bytes, 2 * bytes));
  assert.deepEqual(parseTierLayers([7, 1, 2]), [{ layer: 7, tier: 1, kind: 2 }]);
});
