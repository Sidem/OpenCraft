// Tests for the Controls page's data (web/src/ui/control-bindings.ts, keyboard-layout.ts, keyboard-geometry.ts), run with
// `node --test web/test`: the table and input.ts agree (the drift guard), every key has a cap, the built-in layouts match what
// Windows itself prints (fixtures/windows-layouts.json), labels are learned and chosen as documented, and the category colours
// stay apart for people with the common kinds of colour blindness and carry readable text.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { BINDINGS, CATEGORIES, boundKeys, categoryOf, fill, tokensOf } from '../src/ui/control-bindings.ts';
import { KEY_PLACES } from '../src/ui/keyboard-geometry.ts';
import { KeyLabels, PRESETS, inferPreset, presetLabel } from '../src/ui/keyboard-layout.ts';
import { DEUTAN, PROTAN, TRITAN, contrast, deltaE2000, lab, mul, rgb } from './colour.mjs';

const read = (path) => readFileSync(new URL(path, import.meta.url), 'utf8');

// ---- the table and input.ts

test('every key input.ts reads is in the controls table, and every key in the table is read', () => {
  const source = read('../src/input.ts');
  const code = /['"](Key[A-Z]|Digit\d|Space|Tab|Enter|Escape|Home|Slash|Backquote|F\d+|Arrow(?:Up|Down|Left|Right)|(?:Shift|Control)(?:Left|Right))['"]/g;
  const read_ = new Set([...source.matchAll(code)].map((m) => m[1]));
  if (/Digit\(\[1-9\]\)/.test(source)) for (let d = 1; d <= 9; d++) read_.add(`Digit${d}`);
  const listed = boundKeys();
  assert.deepEqual([...read_].filter((c) => !listed.has(c)).sort(), [], 'read by input.ts but missing from control-bindings.ts');
  assert.deepEqual([...listed].filter((c) => !read_.has(c)).sort(), [], 'listed in control-bindings.ts but never read by input.ts');
});

test('the code scan really sees input.ts (it finds the keys it should)', () => {
  const source = read('../src/input.ts');
  for (const probe of ['KeyW', 'Space', 'ShiftLeft', 'Slash', 'F3']) assert.ok(source.includes(`'${probe}'`), probe);
});

test('every {Code} in a label is one of the row\'s keys, and every one in a text is a key the table lists', () => {
  const listed = boundKeys();
  for (const b of BINDINGS) {
    for (const t of tokensOf(b.label)) assert.ok(b.keys.includes(t), `"${b.label}" names ${t}, which is not in its keys`);
    for (const t of tokensOf(b.text)) assert.ok(listed.has(t), `"${b.text}" names ${t}, which no row lists`);
  }
});

test('every row says something, uses a known category, and names a key or a mouse input', () => {
  const ids = new Set(CATEGORIES.map((c) => c.id));
  for (const b of BINDINGS) {
    assert.ok(ids.has(b.cat), b.label);
    assert.ok(b.label.trim() && b.text.trim(), b.label);
    assert.ok(b.keys.length > 0 || (b.mouse?.length ?? 0) > 0, `${b.label} has neither a key nor a mouse input`);
  }
  for (const c of CATEGORIES) assert.ok(BINDINGS.some((b) => b.cat === c.id), `${c.id} has no rows`);
});

test('every key the game reads has a cap on the keyboard picture, and no cap is drawn twice', () => {
  const places = new Set(KEY_PLACES.map((p) => p.code));
  assert.equal(places.size, KEY_PLACES.length, 'a key place appears twice');
  for (const code of boundKeys()) assert.ok(places.has(code), `${code} has no cap`);
});

test('keys do not overlap on the picture', () => {
  for (const a of KEY_PLACES) {
    for (const b of KEY_PLACES) {
      if (a === b || a.y !== b.y) continue;
      assert.ok(a.x + a.w <= b.x + 1e-9 || b.x + b.w <= a.x + 1e-9, `${a.code} overlaps ${b.code}`);
    }
  }
});

test('every key takes a colour from the first row that uses it', () => {
  assert.equal(categoryOf('KeyW'), 'move');
  assert.equal(categoryOf('ShiftLeft'), 'move'); // sprint comes before Shift+R
  assert.equal(categoryOf('KeyR'), 'build');
  assert.equal(categoryOf('KeyM'), 'screens');
  assert.equal(categoryOf('Digit0'), null);
});

test('labels and texts fill in with the layout\'s caps', () => {
  const azerty = (code) => presetLabel('azerty', code) ?? code;
  assert.equal(fill('{KeyW} {KeyA} {KeyS} {KeyD}', azerty), 'Z Q S D');
  assert.equal(fill('{KeyM}', azerty), ',');
});

// ---- layouts

test('the built-in layouts print what Windows prints at every key place (US, German, French, Dvorak)', () => {
  const windows = JSON.parse(read('./fixtures/windows-layouts.json'));
  for (const [fixture, preset] of [['us', 'qwerty'], ['qwertz', 'qwertz'], ['azerty', 'azerty'], ['dvorak', 'dvorak']]) {
    for (const [code, printed] of Object.entries(windows[fixture])) {
      assert.equal(presetLabel(preset, code)?.toLowerCase(), printed.toLowerCase(), `${preset} at ${code}`);
    }
  }
});

test('every built-in layout has each letter exactly once and rows of the right length', () => {
  for (const [id, p] of Object.entries(PRESETS)) {
    assert.equal([...p.row1].length, 12, `${id} row 1`);
    assert.equal([...p.row2].length, 11, `${id} row 2`);
    assert.equal([...p.row3].length, 10, `${id} row 3`);
    const printed = [...(p.row1 + p.row2 + p.row3)].filter((c) => /[a-z]/.test(c)).sort().join('');
    assert.equal(printed, 'abcdefghijklmnopqrstuvwxyz', `${id} letters`);
  }
});

test('Colemak keeps its well-known changes (checked by hand: Windows has no Colemak)', () => {
  const at = (code) => presetLabel('colemak', code);
  assert.deepEqual(['KeyE', 'KeyR', 'KeyT', 'KeyS', 'KeyD', 'KeyF', 'KeyG', 'KeyJ', 'KeyK', 'KeyL', 'KeyP', 'Semicolon', 'KeyN'].map(at), ['F', 'P', 'G', 'R', 'S', 'T', 'D', 'N', 'E', 'I', ';', 'O', 'K']);
  assert.deepEqual(['KeyW', 'KeyA', 'KeyQ', 'KeyZ', 'KeyX', 'KeyC', 'KeyV', 'KeyB', 'KeyM'].map(at), ['W', 'A', 'Q', 'Z', 'X', 'C', 'V', 'B', 'M']);
});

/** A store that remembers in memory. */
const memory = () => {
  const data = new Map();
  return { data, get: (k) => data.get(k) ?? null, set: (k, v) => void data.set(k, String(v)) };
};
const press = (code, key, extra = {}) => ({ code, key, ...extra });

test('with nothing known, labels are QWERTY, and words and digits never change', () => {
  const labels = new KeyLabels(memory());
  assert.equal(labels.source, 'assumed');
  assert.deepEqual(['KeyW', 'KeyM', 'Slash'].map((c) => labels.label(c)), ['W', 'M', '/']);
  assert.deepEqual(['Space', 'Escape', 'ShiftLeft', 'ControlRight', 'ArrowUp', 'F3', 'Digit4'].map((c) => labels.label(c)), ['Space', 'Esc', 'Shift', 'Ctrl', '↑', 'F3', '4']);
});

test('one key press on a French keyboard is enough to relabel the whole board', () => {
  const labels = new KeyLabels(memory());
  assert.ok(labels.learn(press('KeyW', 'z')));
  assert.equal(labels.source, 'learned');
  assert.equal(labels.looksLike, 'azerty');
  assert.deepEqual(['KeyW', 'KeyA', 'KeyS', 'KeyD', 'KeyM', 'Slash'].map((c) => labels.label(c)), ['Z', 'Q', 'S', 'D', ',', '!']);
});

test('a German keyboard is told apart from a US one by Y and Z, and Dvorak by almost any letter', () => {
  const de = new KeyLabels(memory());
  de.learn(press('KeyY', 'z'));
  assert.equal(de.looksLike, 'qwertz');
  const dv = new KeyLabels(memory());
  dv.learn(press('KeyS', 'o'));
  assert.equal(dv.looksLike, 'dvorak');
  const us = new KeyLabels(memory());
  us.learn(press('KeyE', 'e'));
  us.learn(press('KeyW', 'w'));
  assert.equal(us.looksLike, 'qwerty');
});

test('presses that say nothing about the layout are not learned', () => {
  const labels = new KeyLabels(memory());
  for (const e of [
    press('KeyW', 'W', { shiftKey: true }), press('KeyW', 'w', { ctrlKey: true }), press('KeyW', 'w', { altKey: true }),
    press('KeyW', 'w', { metaKey: true }), press('Digit1', '&'), press('Space', ' '), press('Enter', 'Enter'),
    press('KeyW', 'Dead'), press('KeyW', 'Process'), press('F3', 'F3'),
  ]) assert.equal(labels.learn(e), false, `${e.code} ${e.key}`);
  assert.equal(labels.source, 'assumed');
});

test('learned labels persist, and a layout that fits no preset keeps what was learned', () => {
  const store = memory();
  new KeyLabels(store).learn(press('KeyW', 'z'));
  assert.equal(new KeyLabels(store).label('KeyW'), 'Z');
  const odd = new KeyLabels(memory());
  odd.learn(press('KeyW', 'x')); // no built-in layout puts X there
  assert.equal(odd.looksLike, 'qwerty');
  assert.equal(odd.label('KeyW'), 'X');
  assert.equal(odd.label('KeyE'), 'E');
});

test('the browser\'s layout map wins over learned labels; an empty map counts as none; a chosen layout wins over both', () => {
  const labels = new KeyLabels(memory());
  labels.learn(press('KeyW', 'z'));
  labels.setMap(new Map([['KeyW', 'w'], ['KeyM', ';']]));
  assert.equal(labels.source, 'map');
  assert.deepEqual(['KeyW', 'KeyM'].map((c) => labels.label(c)), ['W', ';']);
  labels.setMap(new Map());
  assert.equal(labels.source, 'learned');
  assert.equal(labels.label('KeyW'), 'Z');
  labels.setMap(new Map([['KeyW', 'w']]));
  labels.setChoice('dvorak');
  assert.equal(labels.source, 'chosen');
  assert.equal(labels.label('KeyW'), ',');
});

test('the chosen layout is remembered, and a stale choice in storage is ignored', () => {
  const store = memory();
  new KeyLabels(store).setChoice('azerty');
  assert.equal(new KeyLabels(store).choice, 'azerty');
  store.data.set('opencraft.keyboardLayout', 'klingon');
  assert.equal(new KeyLabels(store).choice, 'auto');
  store.data.set('opencraft.keyLabels', '{"KeyW": 5, "Space": "x", "KeyQ": "a"');
  assert.doesNotThrow(() => new KeyLabels(store));
});

test('labels tell their listeners when they change', () => {
  const labels = new KeyLabels(memory());
  let calls = 0;
  labels.subscribe(() => calls++);
  labels.learn(press('KeyW', 'z'));
  labels.learn(press('KeyW', 'z')); // nothing new
  labels.setChoice('qwertz');
  assert.equal(calls, 2);
  assert.equal(inferPreset({}), 'qwerty');
});

// ---- colours

test('the colour maths is right: CIEDE2000 matches published test pairs, and the blindness matrices keep white white', () => {
  assert.ok(Math.abs(deltaE2000([50, 2.6772, -79.7751], [50, 0, -82.7485]) - 2.0425) < 1e-3);
  assert.ok(Math.abs(deltaE2000([50, 2.5, 0], [73, 25, -18]) - 27.1492) < 1e-3);
  assert.ok(Math.abs(deltaE2000([60.2574, -34.0099, 36.2677], [60.4626, -34.1751, 39.4387]) - 1.2644) < 1e-3);
  for (const m of [PROTAN, DEUTAN, TRITAN]) for (const row of m) assert.ok(Math.abs(row.reduce((s, v) => s + v, 0) - 1) < 1e-4);
});

test('the six category colours stay clearly apart for normal sight and for protan, deutan and tritan colour blindness', () => {
  const NEEDED = 15; // CIEDE2000; 2 is barely visible side by side, 10 or more is plain at a glance (the Okabe-Ito set itself scores 10.9)
  const worst = [];
  for (const [name, sim] of [['normal', null], ['protan', PROTAN], ['deutan', DEUTAN], ['tritan', TRITAN]]) {
    const seen = CATEGORIES.map((c) => lab(sim ? mul(sim, rgb(c.colour)) : rgb(c.colour)));
    for (let i = 0; i < seen.length; i++) {
      for (let j = i + 1; j < seen.length; j++) {
        const d = deltaE2000(seen[i], seen[j]);
        worst.push({ name, pair: `${CATEGORIES[i].id}/${CATEGORIES[j].id}`, d });
      }
    }
  }
  const low = worst.filter((w) => w.d < NEEDED).map((w) => `${w.name} ${w.pair} ${w.d.toFixed(1)}`);
  assert.deepEqual(low, [], `pairs closer than ${NEEDED}`);
});

test('the key caps\' text is readable on every category colour and on an unlit cap', () => {
  const TEXT = rgb('#10131a');
  for (const c of CATEGORIES) assert.ok(contrast(TEXT, rgb(c.colour)) >= 4.5, `${c.id} ${c.colour}`);
  assert.ok(contrast(rgb('#d4d9e2'), rgb('#262b36')) >= 4.5, 'an unlit cap (light text on dark)');
});
