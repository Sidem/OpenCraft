// Tests for the performance check: naming software rendering without false alarms (render/gpu-probe.ts), honest frame
// figures (render/frame-stats.ts) and the report and browser steps (ui/perf-text.ts).

import assert from 'node:assert/strict';
import test from 'node:test';
import { FrameStats, FrameTimes } from '../src/render/frame-stats.ts';
import { classifyGpu } from '../src/render/gpu-probe.ts';
import { accelerationSteps, buildDate, formatReport } from '../src/ui/perf-text.ts';

test('software renderers are named as software whatever the second signal says', () => {
  for (const name of [
    'ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (Subzero) (0x0000C0DE)), SwiftShader driver)',
    'Google SwiftShader',
    'llvmpipe (LLVM 15.0.7, 256 bits)',
    'ANGLE (Microsoft, Microsoft Basic Render Driver (0x0000008C) Direct3D11 vs_5_0 ps_5_0, D3D11)',
    'Microsoft Corporation GDI Generic',
  ]) {
    for (const caveat of [true, false, null]) assert.equal(classifyGpu(name, caveat), 'software', name);
  }
});

test('a real graphics card never brings a warning', () => {
  for (const name of [
    'ANGLE (NVIDIA, NVIDIA GeForce RTX 3080 (0x00002206) Direct3D11 vs_5_0 ps_5_0, D3D11)',
    'ANGLE (Intel, Intel(R) UHD Graphics 620 (0x00005917) Direct3D11 vs_5_0 ps_5_0, D3D11)',
    'Apple M2',
  ]) {
    assert.equal(classifyGpu(name, true), 'hardware', name);
    assert.equal(classifyGpu(name, null), 'hardware', name);
    assert.equal(classifyGpu(name, false), 'unknown', `${name}: the signals disagree, so no warning`);
  }
});

test('a hidden name is judged only by what the browser says', () => {
  for (const name of ['', '  ', 'Mozilla', 'WebKit WebGL']) {
    assert.equal(classifyGpu(name, true), 'hardware', `"${name}" passed`);
    assert.equal(classifyGpu(name, null), 'unknown', `"${name}" untried`);
    assert.equal(classifyGpu(name, false), 'software', `"${name}" refused`);
  }
});

test('the median and the slowest 5% are of the frames kept, and hitches are counted', () => {
  const s = new FrameStats();
  assert.deepEqual(s.summary(), { frames: 0, medianMs: 0, p95Ms: 0, long: 0 });
  for (let i = 0; i < 95; i++) s.add(16);
  for (let i = 0; i < 5; i++) s.add(100);
  assert.deepEqual(s.summary(), { frames: 100, medianMs: 16, p95Ms: 16, long: 5 });
  s.add(100);
  assert.equal(s.summary().p95Ms, 100, 'more than one in twenty now');
});

test('old frames fall out of the window', () => {
  const s = new FrameStats();
  for (let i = 0; i < 600; i++) s.add(200);
  for (let i = 0; i < 600; i++) s.add(16);
  assert.deepEqual(s.summary(), { frames: 600, medianMs: 16, p95Ms: 16, long: 0 });
});

test('fps keeps falling below 10, where a capped time step stops', () => {
  const t = new FrameTimes();
  for (let now = 0, i = 0; i < 50; i++, now += 250) t.tick(now, false);
  assert.ok(Math.abs(t.fps - 4) < 0.01, `read ${t.fps}`);
});

test('menu frames do not mix into play, and a hidden tab is not a frame', () => {
  const t = new FrameTimes();
  t.tick(0, false);
  t.tick(20, false);
  t.tick(60, true);
  t.tick(9000, true);
  t.tick(9010, false);
  assert.equal(t.playing.summary().frames, 2);
  assert.equal(t.menu.summary().frames, 1);
  assert.equal(t.menu.summary().medianMs, 40);
});

const FACTS = {
  notes: '# Patch notes\n\n## 2026-11-07 · Round trees\n- x\n\n## 2026-11-01 · Older\n',
  gpu: 'Some GPU',
  kind: 'software',
  browser: 'UA',
  window: [1920, 1080],
  screen: [2560, 1440],
  drawn: [2400, 1350],
  ratio: 1.25,
  cores: 8,
  memoryGb: undefined,
  quality: { scale: 75, radius: 8 },
  sky: { quality: 2 },
  playing: { frames: 600, medianMs: 40, p95Ms: 90, long: 12 },
  menu: { frames: 0, medianMs: 0, p95Ms: 0, long: 0 },
  frameMs: 3.21,
  workMs: 1.5,
  meshes: 400,
  visible: 120,
  drawCalls: 300,
  quads: 250000,
  chunks: [500, 3, 1],
  factory: [10, 2, 4],
  items: 7,
  session: 'solo',
  lost: 0,
};

test('the report says what is needed on one screen', () => {
  const text = formatReport(FACTS);
  assert.ok(text.split('\n').length < 30);
  for (const part of ['Build: 2026-11-07', 'Using the graphics card: no', '25 fps typical, slowest 5% over 90 ms, 12 long of 600', 'In the menu: no frames yet', 'Quality: scale=75 radius=8', 'memory not reported', 'drawing 2400x1350', '250k quads']) {
    assert.ok(text.includes(part), part);
  }
});

test('the build date falls back when the notes have none', () => {
  assert.equal(buildDate('nothing here'), 'unknown');
});

test('each browser gets its own way to the switch', () => {
  const chrome = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36';
  assert.match(accelerationSteps(chrome), /chrome:\/\/settings\/system/);
  assert.match(accelerationSteps(`${chrome} Edg/130.0.0.0`), /edge:\/\/settings\/system/, 'Edge also says Chrome');
  assert.match(accelerationSteps(`${chrome} OPR/115.0.0.0`), /opera:\/\/settings\/system/);
  assert.match(accelerationSteps('Mozilla/5.0 (Windows NT 10.0; rv:132.0) Gecko/20100101 Firefox/132.0'), /about:preferences/);
  assert.match(accelerationSteps('Mozilla/5.0 (Macintosh) AppleWebKit/605.1.15 Version/18.0 Safari/605.1.15'), /no switch/);
  assert.match(accelerationSteps('Something else'), /browser's settings/);
});
