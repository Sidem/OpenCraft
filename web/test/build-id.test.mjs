// Tests for the co-op build id (web/src/net/protocol.ts): the same bundle served from different hosts or
// folders (GitHub Pages, itch.io's iframe host) must match; a different bundle must not.

import assert from 'node:assert/strict';
import test from 'node:test';
import { buildIdOf } from '../src/net/build-id.ts';

test('the same bundle on GitHub Pages and on itch.io has one build id', () => {
  const pages = buildIdOf('https://sidem.github.io/OpenCraft/assets/index-B06nHxbm.js');
  const itch = buildIdOf('https://html-classic.itch.zone/html/123456/assets/index-B06nHxbm.js');
  assert.equal(pages, 'index-B06nHxbm.js');
  assert.equal(itch, pages);
});

test('a different bundle (another content hash) has a different build id', () => {
  assert.notEqual(
    buildIdOf('https://sidem.github.io/OpenCraft/assets/index-B06nHxbm.js'),
    buildIdOf('https://sidem.github.io/OpenCraft/assets/index-Zz9yXwVu.js'),
  );
});

test('query strings and fragments are not part of the id', () => {
  assert.equal(buildIdOf('http://localhost:5173/src/net/protocol.ts?t=1700000000#x'), 'protocol.ts');
});
