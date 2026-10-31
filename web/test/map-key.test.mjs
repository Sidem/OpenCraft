// Tests for the map key (web/src/ui/map-key-rows.ts): the host's mark numbers are the engine's, and every mark the
// engine can send has a row that says what it means.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { KEY_ROWS, keyCovers } from '../src/ui/map-key-rows.ts';
import * as pins from '../src/ui/pins.ts';

/** `MARK_*` shape constants of the engine (Rust cannot list them either; marks.rs is read as text). */
function engineShapes() {
  const src = readFileSync(new URL('../../crates/engine/src/minimap/marks.rs', import.meta.url), 'utf8');
  return [...src.matchAll(/^pub const (MARK_\w+): i32 = (\d+);/gm)].map((m) => [m[1], Number(m[2])]);
}

test('the host mirrors every engine mark number', () => {
  const shapes = engineShapes();
  assert.ok(shapes.length >= 12, `read from the source: ${shapes}`);
  for (const [name, value] of shapes) assert.equal(pins[name], value, `${name} in pins.ts`);
});

test('every engine mark shape has a row in the key', () => {
  for (const [name, value] of engineShapes()) assert.ok(keyCovers(value), `${name} (${value}) has no row`);
  for (const row of KEY_ROWS) assert.ok(row.label.length > 0 && row.span >= 1, JSON.stringify(row));
});
