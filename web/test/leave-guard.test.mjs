// Tests for the "ask before closing" rule (web/src/save/leave-guard.ts): when closing is held back, and that the
// setting is off until turned on and survives unavailable storage.

import assert from 'node:assert/strict';
import test from 'node:test';
import { holdsClosing, loadAskBeforeClosing, saveAskBeforeClosing } from '../src/save/leave-guard.ts';

test('closing is held back only with the setting on and the world changed since it was covered', () => {
  assert.equal(holdsClosing(false, 500, 0), false, 'off by default behaviour: never holds');
  assert.equal(holdsClosing(true, 500, 500), false, 'nothing new since the export or "not now"');
  assert.equal(holdsClosing(true, 501, 500), true, 'played on since');
  assert.equal(holdsClosing(true, 0, 0), false, 'a fresh, untouched world');
});

test('the setting is off until turned on, and storage that throws means off', () => {
  const data = new Map();
  globalThis.localStorage = { getItem: (k) => data.get(k) ?? null, setItem: (k, v) => data.set(k, v) };
  assert.equal(loadAskBeforeClosing(), false);
  saveAskBeforeClosing(true);
  assert.equal(loadAskBeforeClosing(), true);
  saveAskBeforeClosing(false);
  assert.equal(loadAskBeforeClosing(), false);
  globalThis.localStorage = {
    getItem() {
      throw new Error('blocked');
    },
    setItem() {
      throw new Error('blocked');
    },
  };
  assert.equal(loadAskBeforeClosing(), false);
  assert.doesNotThrow(() => saveAskBeforeClosing(true));
});
