// Tests for automatic quality (web/src/render/auto-quality.ts) and the settings store (render/quality.ts). Each `replay` plays
// frames through the controller, the frame times depending on the level it has chosen, as they would on a machine.

import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import test from 'node:test';
import { AutoQuality } from '../src/render/auto-quality.ts';
import { DEFAULTS, QualityStore } from '../src/render/quality.ts';

/** `levels`: for each level, a list of frame times to cycle through. */
function replay(levels, { start = 0, seconds = 120, script = () => 2 } = {}) {
  const auto = new AutoQuality(start, levels.length - 1);
  const steps = [];
  let now = 0;
  for (let i = 0; now < seconds * 1000; i++) {
    const t = levels[auto.level];
    const ms = t[i % t.length];
    now += ms;
    const step = auto.frame(now, ms, script(ms));
    if (step) steps.push([step, Math.round(now / 1000)]);
  }
  return { names: steps.map((s) => s[0]), steps, level: auto.level };
}

const flat = (...ms) => ms.map((m) => [m]);

test('a fast machine is never touched, on any display', () => {
  for (const ms of [6.9, 8.3, 16.7]) assert.deepEqual(replay(flat(ms, ms, ms), { seconds: 600 }).names, [], `${ms} ms`);
});

test('single hitches and a short bad patch change nothing', () => {
  const hitches = Array.from({ length: 60 }, (_, i) => (i === 0 ? 300 : 16.7));
  assert.deepEqual(replay([hitches, hitches, hitches], { seconds: 600 }).names, []);
  const badStart = [...Array(150).fill(60), ...Array(99999).fill(16.7)]; // 9 s bad, of which 5 s are warm-up: one bad window at most
  assert.deepEqual(replay([badStart, badStart, badStart], { seconds: 40 }).names, []);
});

test('too many pixels: lowers step by step within half a minute and stops when it is fine', () => {
  const r = replay(flat(60, 34, 16), { seconds: 45 });
  assert.deepEqual(r.names, ['lower', 'lower']);
  assert.equal(r.level, 2);
  assert.ok(r.steps[1][1] <= 30, `second step at ${r.steps[1][1]} s`);
});

test('a lowering that helps is kept', () => {
  const r = replay(flat(40, 20, 12), { seconds: 45 });
  assert.deepEqual(r.names, ['lower']);
  assert.equal(r.level, 1);
});

test('a frame-rate lock: the step does nothing, so it is undone and Auto stops', () => {
  const r = replay(flat(33.3, 33.3, 33.3), { seconds: 300 });
  assert.deepEqual(r.names, ['lower', 'undo']);
  assert.equal(r.level, 0);
});

test('a small but real gain is kept, through a noisy pair of windows', () => {
  // As Firefox drawing in software: 17% better at 75%, with frames that vary from window to window.
  const noisy = (ms) => Array.from({ length: 400 }, (_, i) => ms * (Math.floor(i / 25) % 2 ? 1.12 : 0.88));
  const r = replay([noisy(65), noisy(54), noisy(48)], { seconds: 45 });
  assert.deepEqual(r.names.slice(0, 2), ['lower', 'lower']);
  assert.equal(r.level, 2);
});

test('very slow and sharpness does nothing: undone, but the player still hears about it', () => {
  assert.deepEqual(replay(flat(65, 65, 65), { seconds: 120 }).names, ['lower', 'undo', 'suggest-drawing']);
});

test('still slow at the lowest level: one suggestion, not a stream of them', () => {
  const r = replay(flat(200, 120, 70), { seconds: 300 });
  assert.deepEqual(r.names, ['lower', 'lower', 'suggest-drawing']);
});

test('steady stutter counts, though most frames are quick', () => {
  const stutter = [11, 11, 11, 11, 149];
  assert.equal(replay([stutter, [8], [8]], { seconds: 40 }).names[0], 'lower', 'a median would read 11 ms');
});

test('one long frame after a resize does not make a step look useless', () => {
  const r = replay([[1600], [3700, 1250, 1250, 1250, 1250, 1250, 1250], [950]], { seconds: 60 });
  assert.deepEqual(r.names.slice(0, 2), ['lower', 'lower']);
});

test('limited by the processor: fewer pixels would not help, so it goes straight to the suggestion', () => {
  const r = replay(flat(50, 50, 50), { seconds: 120, script: (ms) => ms * 0.8 });
  assert.deepEqual(r.names, ['suggest-script']);
  assert.equal(r.level, 0);
});

test('raises once after a long good stretch, and at most once', () => {
  assert.deepEqual(replay(flat(16.7, 16.7, 16.7), { start: 2, seconds: 600 }).names, ['raise']);
  const bounced = replay(flat(60, 34, 16), { seconds: 900 });
  assert.deepEqual(bounced.names, ['lower', 'lower', 'raise', 'lower'], 'one reversal, then it stays');
  assert.equal(bounced.level, 2);
});

test('frames that were not fed (a menu, a hidden tab) do not count towards a window', () => {
  const auto = new AutoQuality(0, 2);
  let now = 0;
  const feed = (ms, n) => { for (let i = 0; i < n; i++) { now += ms; assert.equal(auto.frame(now, ms, 2), null); } };
  feed(16.7, 400); // warm
  for (let i = 0; i < 6; i++) {
    feed(40, 40); // 1.6 s slow: never a whole window
    now += 30000; // away in a menu
  }
});

const TRACES = new URL('./fixtures/frame-traces.json', import.meta.url);
test('recorded frames from real browsers', { skip: !existsSync(TRACES) }, () => {
  const traces = JSON.parse(readFileSync(TRACES, 'utf8'));
  for (const t of traces.fast) assert.deepEqual(replay(t.levels, { seconds: 600 }).names, [], t.name);
  for (const t of traces.pixels) {
    const r = replay(t.levels, { seconds: 60 });
    assert.equal(r.names[0], 'lower', t.name);
    assert.ok(r.level > 0, t.name);
    const undone = r.names.filter((n) => n === 'undo').length;
    assert.ok(undone <= 1, `${t.name}: ${r.names}`);
    const last = r.steps.filter((s) => s[0] === 'lower' || s[0] === 'undo').at(-1);
    assert.ok(last[1] <= 30, `${t.name}: settled at ${last[1]} s`);
  }
});

function withStorage(initial, fn) {
  const data = new Map(Object.entries(initial));
  globalThis.localStorage = { getItem: (k) => data.get(k) ?? null, setItem: (k, v) => data.set(k, v) };
  try {
    fn(data);
  } finally {
    delete globalThis.localStorage;
  }
}

test('the settings keep valid values and drop bad ones', () => {
  withStorage({ 'opencraft.quality': '{"scale":75,"radius":7,"antialias":false}' }, () => {
    assert.deepEqual(new QualityStore().settings, { ...DEFAULTS, scale: 75, antialias: false });
  });
  withStorage({ 'opencraft.quality': 'not json' }, () => assert.deepEqual(new QualityStore().settings, DEFAULTS));
  assert.deepEqual(new QualityStore().settings, DEFAULTS, 'no storage at all');
});

test('choosing a setting turns Auto off; Auto changing one does not', () => {
  withStorage({}, () => {
    const s = new QualityStore();
    s.set('scale', 75);
    assert.equal(s.settings.auto, true);
    s.choose('scale', 50);
    assert.equal(s.settings.auto, false);
  });
});

test('a render distance from the address wins and stores nothing', () => {
  withStorage({}, (data) => {
    const s = new QualityStore(12);
    assert.equal(s.settings.radius, 12);
    s.set('scale', 75);
    assert.equal(data.size, 0);
  });
});
