// The game's version lives in package.json (rule: docs/DECISIONS.md, "Version numbers"). The engine crate's
// version and the newest patch note's heading must carry the same one, so a bump can't be half done.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = (path) => readFileSync(new URL(`../../${path}`, import.meta.url), 'utf8');
const version = JSON.parse(read('package.json')).version;

test('the version is X.Y.Z', () => {
  assert.match(version, /^\d+\.\d+\.\d+$/);
});

test('the engine crate has the same version', () => {
  assert.equal(read('crates/engine/Cargo.toml').match(/^version = "([^"]+)"/m)?.[1], version);
});

test('the newest patch note carries the version', () => {
  const heading = read('web/src/patch-notes.md').match(/^## .*$/m)?.[0] ?? '';
  const noted = heading.split(' · ')[1]?.trim();
  assert.equal(noted, `v${version}`, `newest note "${heading}" should be "## date · v${version} · title"`);
});
