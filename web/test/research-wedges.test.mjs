import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { KINDS } from '../src/ui/research-kinds.ts';
import { WEDGE_COUNT, WEDGES } from '../src/ui/research-wedges.ts';

const ringsRs = readFileSync(new URL('../../crates/engine/src/research/rings.rs', import.meta.url), 'utf8');

test('WEDGES matches the engine Wedge enum count and indices (0..11)', () => {
  const m = ringsRs.match(/pub enum Wedge \{([\s\S]*?)\}/);
  assert.ok(m, 'found Wedge enum in rings.rs');
  const variants = [...m[1].matchAll(/(\w+)\s*=\s*(\d+)/g)].map(([, name, idx]) => ({ name, idx: Number(idx) }));
  assert.equal(variants.length, WEDGE_COUNT);
  assert.equal(WEDGES.length, WEDGE_COUNT);
  for (let i = 0; i < WEDGE_COUNT; i++) {
    assert.equal(variants[i].idx, i);
    assert.equal(WEDGES[i].id, i);
    assert.ok(WEDGES[i].name.length > 0);
    assert.ok(WEDGES[i].blurb.length > 0);
    assert.ok(KINDS[WEDGES[i].kind], `valid kind for wedge ${i}`);
  }
});
