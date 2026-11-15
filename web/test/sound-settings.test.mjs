// Tests for the sound settings (web/src/audio/settings.ts), run with `node --test web/test`: the master level's gain curve,
// and that sound settings saved before the master existed load unchanged with the master at full and unmuted.

import assert from 'node:assert/strict';
import test from 'node:test';
import { DEFAULT_LEVELS, loadSettings, masterGain, saveSettings } from '../src/audio/settings.ts';

/** A stand-in for the browser's storage, holding `saved` under the sound settings' key (or nothing). */
function storage(saved) {
  const data = new Map(saved === undefined ? [] : [['opencraft.sound', JSON.stringify(saved)]]);
  globalThis.localStorage = {
    getItem: (k) => data.get(k) ?? null,
    setItem: (k, v) => void data.set(k, String(v)),
  };
  return data;
}

test('the master gain is 1 at full, silent at 0 or muted, and follows the channels\' squared taper', () => {
  assert.equal(masterGain(100, false), 1);
  assert.equal(masterGain(0, false), 0);
  assert.equal(masterGain(50, false), 0.25);
  assert.equal(masterGain(100, true), 0);
  // Out-of-range dial values (a hand-edited save) are clamped, never boosted.
  assert.equal(masterGain(500, false), 1);
  assert.equal(masterGain(-5, false), 0);
});

test('every step of the dial is quieter than the next (no flat spots, no jumps back)', () => {
  let last = -1;
  for (let v = 0; v <= 100; v++) {
    const g = masterGain(v, false);
    assert.ok(g > last, `master ${v} should be louder than ${v - 1}`);
    last = g;
  }
});

test('settings saved before the master load with it at full and unmuted, the channel levels untouched', () => {
  storage({ volume: 35, muted: true, muteInBackground: true, design: {} });
  const s = loadSettings();
  assert.equal(s.master, DEFAULT_LEVELS.master);
  assert.equal(s.masterMuted, false);
  assert.equal(s.volume, 35);
  assert.equal(s.muted, true);
  assert.equal(s.muteInBackground, true);
});

test('with nothing saved, or storage that throws, the defaults apply', () => {
  storage(undefined);
  assert.deepEqual([loadSettings().master, loadSettings().volume], [DEFAULT_LEVELS.master, DEFAULT_LEVELS.volume]);
  globalThis.localStorage = { getItem: () => { throw new Error('blocked'); }, setItem: () => { throw new Error('blocked'); } };
  assert.equal(loadSettings().master, 100);
  assert.doesNotThrow(() => saveSettings(loadSettings()));
});

test('the master and its mute survive a save and a load; a corrupt master falls back to full', () => {
  const data = storage(undefined);
  const s = loadSettings();
  s.master = 40;
  s.masterMuted = true;
  saveSettings(s);
  const back = loadSettings();
  assert.deepEqual([back.master, back.masterMuted], [40, true]);
  data.set('opencraft.sound', JSON.stringify({ master: 'loud', masterMuted: 'yes' }));
  const bad = loadSettings();
  assert.deepEqual([bad.master, bad.masterMuted], [100, false]);
});
