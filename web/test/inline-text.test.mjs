// Tests for the patch notes' inline markup (web/src/ui/inline-text.ts).

import assert from 'node:assert/strict';
import test from 'node:test';
import { inlinePieces } from '../src/ui/inline-text.ts';

test('bold and code spans become pieces, the text between stays plain', () => {
  assert.deepEqual(inlinePieces('**Wall poles** hold on `.ocworld` files.'), [
    { text: 'Wall poles', kind: 'bold' },
    { text: ' hold on ', kind: 'plain' },
    { text: '.ocworld', kind: 'code' },
    { text: ' files.', kind: 'plain' },
  ]);
});

test('plain text, an unclosed marker and an empty line are left as typed', () => {
  assert.deepEqual(inlinePieces('no markup'), [{ text: 'no markup', kind: 'plain' }]);
  assert.deepEqual(inlinePieces('half **open'), [{ text: 'half **open', kind: 'plain' }]);
  assert.deepEqual(inlinePieces(''), []);
});

test('two bold spans in a row keep their own text', () => {
  const kinds = inlinePieces('**a** and **b**').map((p) => p.kind);
  assert.deepEqual(kinds, ['bold', 'plain', 'bold']);
});
