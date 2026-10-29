// Tests for "Show on map" (web/src/ui/map-focus.ts): reading the engine's machine list and fitting the world map's view
// so every machine of the kind is on screen.

import assert from 'node:assert/strict';
import test from 'node:test';
import { fitView, parseMachines, tally } from '../src/ui/map-focus.ts';

const ZOOMS = [0.25, 0.5, 1, 2, 4, 8, 16];
const W = 800, H = 500, PAD = 40;

/** Where a column lands on screen for a fitted view. */
function onScreen(view, p) {
  const bpp = ZOOMS[view.zoom];
  return { sx: (p.x + 0.5 - view.x) / bpp + W / 2, sy: (p.z + 0.5 - view.z) / bpp + H / 2 };
}

test('parses five numbers a machine and ignores extra fields', () => {
  assert.deepEqual(parseMachines(Int32Array.from([1, 2, 3, -1, -1, 9, 8, 7, 55, 2]), 5), [
    { x: 1, y: 2, z: 3, percent: -1, cause: -1 },
    { x: 9, y: 8, z: 7, percent: 55, cause: 2 },
  ]);
  assert.equal(parseMachines([1, 2, 3, 4, 5, 0, 6, 7, 8, 9, 10, 0], 6).length, 2);
  assert.deepEqual(parseMachines([], 5), []);
});

test('counts full-speed, idle and held-back machines by cause', () => {
  const ms = parseMachines([0, 0, 0, -1, -1, 1, 0, 0, 100, -1, 2, 0, 0, 40, 1, 3, 0, 0, 0, 2, 4, 0, 0, 0, 2], 5);
  const t = tally(ms);
  assert.equal(t.full, 1);
  assert.equal(t.idle, 1);
  assert.deepEqual([t.held[1], t.held[2], t.held[0]], [1, 2, undefined]);
});

test('no machines: no view', () => {
  assert.equal(fitView([], W, H, ZOOMS, 1, PAD), null);
});

test('one machine is centred at the closest step allowed', () => {
  assert.deepEqual(fitView([{ x: 10, z: -4 }], W, H, ZOOMS, 1, PAD), { x: 10.5, z: -3.5, zoom: 1 });
});

test('every machine lands inside the padded view, at the closest step that fits', () => {
  const spreads = [5, 60, 300, 700, 1500, 4000];
  for (const spread of spreads) {
    for (let seed = 1; seed <= 20; seed++) {
      let s = seed * 7919 + spread;
      const rnd = () => ((s = (s * 48271) % 2147483647) / 2147483647);
      const ox = Math.floor(rnd() * 20000 - 10000), oz = Math.floor(rnd() * 20000 - 10000);
      const pts = Array.from({ length: 2 + (seed % 9) }, () => ({ x: ox + Math.floor(rnd() * spread), z: oz + Math.floor(rnd() * spread * 0.6) }));
      const v = fitView(pts, W, H, ZOOMS, 1, PAD);
      for (const p of pts) {
        const { sx, sy } = onScreen(v, p);
        assert.ok(sx >= PAD - 1 && sx <= W - PAD + 1 && sy >= PAD - 1 && sy <= H - PAD + 1, `spread ${spread}: ${sx}, ${sy}`);
      }
      // One step closer would not fit (unless already at the closest allowed).
      if (v.zoom > 1) {
        const tighter = { ...v, zoom: v.zoom - 1 };
        // Judged by the columns' edges, which is what the fit keeps inside the padding.
        const bpp = ZOOMS[tighter.zoom];
        const sx = (x) => (x - v.x) / bpp + W / 2, sy = (z) => (z - v.z) / bpp + H / 2;
        assert.ok(pts.some((p) => sx(p.x) < PAD || sx(p.x + 1) > W - PAD || sy(p.z) < PAD || sy(p.z + 1) > H - PAD),
          `spread ${spread}: a closer step would also fit`);
      }
    }
  }
});

test('points too far apart get the widest step around their middle', () => {
  const v = fitView([{ x: 0, z: 0 }, { x: 100000, z: 0 }], W, H, ZOOMS, 1, PAD);
  assert.deepEqual(v, { x: 50000.5, z: 0.5, zoom: ZOOMS.length - 1 });
});
