import assert from 'node:assert/strict';
import test from 'node:test';
import {
  canvasToCell,
  cellToCanvas,
  CLASS_ORE_METAL,
  CLASS_ORE_UNKNOWN,
  CLASS_TUNNEL,
  colorForCell,
  compassEnds,
  describeCell,
  dirAxis,
  legendEntries,
  offsetLabel,
  perpLabel,
  plotRect,
  worldCoord,
} from '../src/ui/seismic-data.ts';

test('seismic compass, world coordinates, MRI offset and labels follow facing', () => {
  assert.deepEqual(compassEnds(0), ['S', 'N']);
  assert.deepEqual(compassEnds(1), ['W', 'E']);
  assert.deepEqual(compassEnds(2), ['N', 'S']);
  assert.deepEqual(compassEnds(3), ['E', 'W']);
  assert.deepEqual(dirAxis(1), [1, 0]);
  assert.deepEqual(dirAxis(2), [0, 1]);

  const d = { anchorX: 100, anchorZ: -40, dir: 1, perpOffset: 0, width: 64 };
  assert.deepEqual(worldCoord(d, 32, 30), [100, 30, -40]);
  assert.deepEqual(worldCoord(d, 44, 20), [112, 20, -40]);
  assert.deepEqual(worldCoord(d, 20, 15), [88, 15, -40]);
  assert.deepEqual(worldCoord({ ...d, perpOffset: 8 }, 32, 24), [100, 24, -32]);
  assert.deepEqual(worldCoord({ ...d, perpOffset: -6 }, 32, 24), [100, 24, -46]);
  assert.equal(offsetLabel(d, 32), '0m');
  assert.equal(offsetLabel(d, 42), '+10m E');
  assert.equal(offsetLabel(d, 22), '−10m W');
  assert.equal(perpLabel(1, 0), 'Center (0m)');
  assert.equal(perpLabel(1, 8), '+8m S');
  assert.equal(perpLabel(1, -6), '−6m N');
});

test('seismic canvas-to-cell and cell-to-canvas round-trip accurately', () => {
  const d = { width: 64, height: 96 };
  const plot = plotRect(960, 520);
  for (const [s, y] of [[0, 0], [32, 48], [63, 95], [17, 23]]) {
    const [cx, cy] = cellToCanvas(plot, d, s, y);
    const back = canvasToCell(plot, d, cx, cy);
    assert.deepEqual(back, { s, y });
  }
  assert.equal(canvasToCell(plot, d, 0, 0), null);
});

test('seismic cell descriptions, palettes and legends respect Mk masking', () => {
  const oreName = (det) => (det === 2 ? 'Iron Ore' : '');
  assert.match(describeCell(CLASS_ORE_UNKNOWN, 0, 220, oreName).label, /dense/i);
  assert.equal(describeCell(CLASS_ORE_METAL, 0, 220, oreName).label, 'Metallic ore reflector');
  assert.equal(describeCell(CLASS_ORE_METAL, 2, 220, oreName).label, 'Iron Ore (metallic)');
  assert.equal(describeCell(CLASS_TUNNEL, 0, 20, oreName).label, 'Excavated tunnel');

  const oreColors = new Map([[2, [210, 120, 75]]]);
  const [r, g, b] = colorForCell('tomo', 220, CLASS_ORE_METAL, 2, oreColors);
  assert.ok(r > b && g > 0 && r <= 255);
  assert.equal(legendEntries(0).length, 5);
  assert.equal(legendEntries(2).length, 6);
});
