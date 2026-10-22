// Tests for the map filters' rules (web/src/ui/map-filter.ts): which category each engine mark shape belongs to, what a
// set of hidden categories lets through, and reading a saved filter back.

import assert from 'node:assert/strict';
import test from 'node:test';
import { MapFilter, markCategory, oreCategory, parseHidden, parseInfo, pinCategory, visibleMarks } from '../src/ui/map-filter.ts';
import {
  MARK_CHANNEL, MARK_COVER, MARK_DEPOSIT, MARK_GUESS, MARK_LAUNCH, MARK_MACHINE, MARK_ORE, MARK_RANGE, MARK_SITE, MARK_TOWN,
} from '../src/ui/pins.ts';

const IRON = 0xd0703a;
const COAL = 0x2a2a2e;

test('every engine mark shape has a category, and ore marks follow their ore', () => {
  assert.equal(markCategory(MARK_DEPOSIT, IRON), oreCategory(IRON));
  assert.equal(markCategory(MARK_ORE, COAL), oreCategory(COAL));
  for (const strength of [0, 1, 2]) assert.equal(markCategory(MARK_GUESS + strength, IRON), oreCategory(IRON));
  for (const shape of [MARK_MACHINE, MARK_CHANNEL, MARK_CHANNEL + 1, MARK_LAUNCH]) {
    assert.equal(markCategory(shape, 0x123456), 'machines', `shape ${shape}`);
  }
  for (const state of [0, 1, 2]) assert.equal(markCategory(MARK_TOWN + state, 0x4aa3ff), 'towns');
  assert.equal(markCategory(MARK_SITE, 0xd55e00), 'sites');
  for (let kind = 0; kind < 4; kind++) assert.equal(markCategory(MARK_COVER + kind, 768), 'cover');
  assert.equal(markCategory(MARK_RANGE, 64), null, 'a pole\'s reach is a building aid and always shows');
});

test('hiding a category hides only its marks', () => {
  // x, z, colour, shape
  const marks = [1, 1, IRON, MARK_DEPOSIT, 2, 2, COAL, MARK_ORE, 3, 3, 0xf2c230, MARK_MACHINE, 4, 4, 64, MARK_RANGE];
  assert.deepEqual(visibleMarks(marks, 4, new Set()), marks);
  assert.deepEqual(visibleMarks(marks, 4, new Set([oreCategory(IRON)])), marks.slice(4));
  assert.deepEqual(visibleMarks(marks, 4, new Set(['machines', oreCategory(COAL)])), [...marks.slice(0, 4), ...marks.slice(12)]);
  assert.deepEqual(visibleMarks(marks, 4, new Set(['cover', 'towns', 'sites'])), marks, 'other groups change nothing');
  assert.deepEqual(visibleMarks([1, 2, 3], 4, new Set()), [], 'a cut-short record is dropped');
});

test('a saved filter reads back, and bad text hides nothing', () => {
  assert.deepEqual([...parseHidden('["machines","pin:home"]')], ['machines', 'pin:home']);
  for (const bad of [null, '', 'machines', '{"a":1}', '[1,2]', '[null]']) assert.equal(parseHidden(bad).size, 0, String(bad));
  assert.deepEqual([...parseHidden('["towns",3,"cover"]')], ['towns', 'cover'], 'only strings are kept');
});

test('the filter object toggles categories and tells its listeners (no storage needed)', () => {
  const filter = new MapFilter();
  let calls = 0;
  filter.subscribe(() => calls++);
  assert.ok(filter.showsPin('home') && filter.showsMark(MARK_SITE, 0));
  filter.toggle(pinCategory('home'));
  filter.set(['sites', 'cover'], false);
  assert.ok(!filter.showsPin('home') && filter.showsPin('note'));
  assert.ok(!filter.showsMark(MARK_SITE, 0) && !filter.showsMark(MARK_COVER, 500) && filter.showsMark(MARK_MACHINE, 0));
  filter.set(['sites'], true);
  filter.toggle(pinCategory('home'));
  assert.ok(filter.showsMark(MARK_SITE, 0) && filter.showsPin('home'));
  assert.equal(calls, 4);
});

test('the engine\'s hover lines are read nearest first, skipping hidden marks, with reaches told from colours', () => {
  const text = [
    `${MARK_DEPOSIT}\t${IRON}\tIron Ore vein\tA deposit.`,
    `${MARK_MACHINE}\t${0xf2c230}\tMiner\tOne of your machines.`,
    `${MARK_COVER}\t768\tSurvey satellite cover\tReaches 768 blocks.`,
    'cut short\t',
    '',
  ].join('\n');
  const all = parseInfo(text, new MapFilter());
  assert.deepEqual(all.map((i) => i.title), ['Iron Ore vein', 'Miner', 'Survey satellite cover']);
  assert.equal(all[0].colour, '#d0703a');
  assert.equal(all[2].colour, null, 'a footprint\'s third number is its reach, not a colour');
  const filter = new MapFilter();
  filter.set(['machines', oreCategory(IRON)], false);
  assert.deepEqual(parseInfo(text, filter).map((i) => i.title), ['Survey satellite cover']);
});
