import test from 'node:test';
import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { glyphSvg, kindOf, KINDS } from '../src/ui/research-kinds.ts';

const dir = new URL('../../crates/engine/src/research/', import.meta.url);

/** Every tech name written as `name: "..."` in the engine's research tables. */
function techNames() {
  const names = [];
  for (const file of readdirSync(dir).filter((f) => f.endsWith('.rs'))) {
    for (const m of readFileSync(new URL(file, dir), 'utf8').matchAll(/^\s+name: "([^"]+)"/gm)) names.push(m[1]);
  }
  return names;
}

test('every kind has a label, a colour and a drawing', () => {
  for (const [kind, style] of Object.entries(KINDS)) {
    assert.ok(style.label, kind);
    assert.match(style.colour, /^\d{1,3}, \d{1,3}, \d{1,3}$/, kind);
    assert.match(style.path, /^M/, kind);
    assert.ok(glyphSvg(kind).includes(style.path));
  }
});

test('every tech in the engine has a kind of its own, not the general one', () => {
  const names = techNames();
  assert.ok(names.length > 60, `found ${names.length} techs`);
  const general = names.filter((n) => kindOf(n) === 'general');
  assert.deepEqual(general, []);
});

test('a tech that is not listed is guessed from its name, else general', () => {
  assert.equal(kindOf('Mk9 Widgets'), 'upgrade');
  assert.equal(kindOf('Plasma Rockets'), 'space');
  assert.equal(kindOf('Quux'), 'general');
});
