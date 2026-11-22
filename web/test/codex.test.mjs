import assert from 'node:assert/strict';
import test from 'node:test';
import { CATEGORY_LABELS, ELEMENT_BY_Z, FORM_LABELS, PERIODIC_ELEMENTS } from '../src/ui/codex-elements.ts';

test('periodic table holds all 118 elements with unique positions and valid categories', () => {
  assert.equal(PERIODIC_ELEMENTS.length, 118);
  const seenPos = new Set();
  for (let z = 1; z <= 118; z++) {
    const el = ELEMENT_BY_Z.get(z);
    assert.ok(el, `missing Z=${z}`);
    assert.ok(el.col >= 1 && el.col <= 18, `bad col for Z=${z}`);
    assert.ok((el.row >= 1 && el.row <= 7) || el.row === 9 || el.row === 10, `bad row for Z=${z}`);
    assert.ok(el.cat in CATEGORY_LABELS, `bad category for Z=${z}`);
    const key = `${el.col},${el.row}`;
    assert.ok(!seenPos.has(key), `duplicate cell ${key} at Z=${z}`);
    seenPos.add(key);
  }
  assert.equal(FORM_LABELS.length, 8);
});
