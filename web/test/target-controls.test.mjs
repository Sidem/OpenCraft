// Tests for structured HUD control chip parsing (web/src/ui/target-controls.ts), run with `node --test`.

import assert from 'node:assert/strict';
import test from 'node:test';
import { presetLabel } from '../src/ui/keyboard-layout.ts';
import { keyCaption, parseHudText } from '../src/ui/target-controls.ts';

test('parseHudText separates status lines from tab-prefixed control rows, preserving order', () => {
  const raw = 'Auto-wires · 32 / 32 blocks\n\tRMB=Place\tHold Shift=Full reach\tHold Ctrl=Unconnected';
  assert.deepEqual(parseHudText(raw), {
    detail: 'Auto-wires · 32 / 32 blocks',
    controls: [
      { spec: 'RMB', label: 'Place' },
      { spec: 'Hold Shift', label: 'Full reach' },
      { spec: 'Hold Ctrl', label: 'Unconnected' },
    ],
  });
});

test('parseHudText collects control rows even when extra status lines follow them', () => {
  const raw = 'Running · 60/min\n\tRMB=Open\nEfficiency: 94%';
  assert.deepEqual(parseHudText(raw), {
    detail: 'Running · 60/min\nEfficiency: 94%',
    controls: [{ spec: 'RMB', label: 'Open' }],
  });
});

test('parseHudText handles control-only cards and empty input cleanly', () => {
  assert.deepEqual(parseHudText(''), { detail: '', controls: [] });
  assert.deepEqual(parseHudText('\tRMB=Deselect\tCtrl+RMB/R=Disconnect all'), {
    detail: '',
    controls: [
      { spec: 'RMB', label: 'Deselect' },
      { spec: 'Ctrl+RMB/R', label: 'Disconnect all' },
    ],
  });
});

test('keyCaption maps letter keys through the keyboard layout and keeps named keys verbatim', () => {
  const qwertz = (code) => presetLabel('qwertz', code) ?? '';
  assert.equal(keyCaption('Z', qwertz), 'Y');
  assert.equal(keyCaption('R', qwertz), 'R');
  assert.equal(keyCaption('Shift', qwertz), 'Shift');
  assert.equal(keyCaption('Enter', qwertz), 'Enter');
});
