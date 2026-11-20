// Tests for the visual target inspector card parser (web/src/ui/target-card.ts), run with `node --test`.

import assert from 'node:assert/strict';
import test from 'node:test';
import { formatSlotCount, formatTenthsRate, parseTargetCard, stripMkSuffix } from '../src/ui/target-card.ts';

test('parseTargetCard extracts tier badge, status, recipe flow, item buffers, efficiency bar, and controls', () => {
  const raw = [
    '@tier|1|42|274|8',
    '@status|warn|Output full',
    '@recipe|12:300,15:150|44:600,18:300',
    '@items|In|15:64,12:16',
    '@items|Out|44:63',
    '@eff|11|waiting for input|89',
    '\tRMB=Open',
  ].join('\n');
  const card = parseTargetCard(raw);
  assert.deepEqual(card.tier, { mk: 1, itemId: 42, kitId: 274, kits: 8 });
  assert.equal(card.grid, null);
  assert.deepEqual(card.rows, [
    { kind: 'status', tone: 'warn', text: 'Output full' },
    {
      kind: 'recipe',
      inputs: [{ itemId: 12, amount: 300 }, { itemId: 15, amount: 150 }],
      outputs: [{ itemId: 44, amount: 600 }, { itemId: 18, amount: 300 }],
    },
    { kind: 'items', group: 'In', items: [{ itemId: 15, amount: 64 }, { itemId: 12, amount: 16 }] },
    { kind: 'items', group: 'Out', items: [{ itemId: 44, amount: 63 }] },
    { kind: 'eff', idle: false, pct: 11, cause: 'waiting for input', lossPct: 89 },
  ]);
  assert.deepEqual(card.controls, [{ spec: 'RMB', label: 'Open' }]);
});

test('parseTargetCard extracts grid badge, power bar, connection pips, and stat chips', () => {
  const raw = [
    '@grid|Main grid|#0072b2',
    '@bar|Power|3320|10000|3,320 / 10,000 kW|ok',
    '@pips|Connections|2|6',
    '@stats|179 poles|136 machines|Reach 160 long · 16 pole · 10 machine',
  ].join('\n');
  const card = parseTargetCard(raw);
  assert.deepEqual(card.grid, { label: 'Main grid', colour: '#0072b2' });
  assert.deepEqual(card.rows, [
    { kind: 'bar', label: 'Power', value: 3320, max: 10000, right: '3,320 / 10,000 kW', tone: 'ok' },
    { kind: 'pips', label: 'Connections', used: 2, total: 6 },
    { kind: 'stats', chips: ['179 poles', '136 machines', 'Reach 160 long · 16 pole · 10 machine'] },
  ]);
});

test('stripMkSuffix removes duplicate Mk suffix when tier badge shows it', () => {
  assert.equal(stripMkSuffix('Assembler Mk1', 1), 'Assembler');
  assert.equal(stripMkSuffix('Conveyor Belt Mk3', 3), 'Conveyor Belt');
  assert.equal(stripMkSuffix('Chemical Plant', 1), 'Chemical Plant');
});

test('rate and slot count formatters keep numbers compact', () => {
  assert.equal(formatTenthsRate(300), '30/m');
  assert.equal(formatTenthsRate(125), '12.5/m');
  assert.equal(formatSlotCount(64), '64');
  assert.equal(formatSlotCount(1500), '1.5k');
  assert.equal(formatSlotCount(12400), '12k');
});
