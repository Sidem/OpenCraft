// Frame-time benchmark: loads a save headlessly, flies a straight line and prints where the frame time goes.
//   npm run build:wasm once, then: node scripts/bench-flight.mjs <world.ocworld> [seconds=60] [yaw=0] [rd=8] [sprint=1]
// It runs what main.ts runs each frame without drawing: `update(dt)`, then streaming work under the host's budget
// (WORK_BUDGET_MS), then draining mesh events (reading the mesh pointer like the renderer does). Times are the engine's
// side of the frame; the browser adds drawing and the HUD. Budget: every frame under 16.6 ms, the engine's share ideally under 8 ms.
import { readFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';
import { pathToFileURL } from 'node:url';

const [file, secs = '60', yaw = '0', rd = '8', sprint = '1'] = process.argv.slice(2);
if (!file) throw new Error('usage: node scripts/bench-flight.mjs <world.ocworld> [seconds] [yaw] [rd] [sprint]');
const WORK_BUDGET_MS = 6;
const wasmDir = new URL('../web/src/wasm/', import.meta.url);
const engine = await import(new URL('engine.js', wasmDir).href);
const wasm = engine.initSync({ module: readFileSync(new URL('engine_bg.wasm', wasmDir)) });
let bytes = readFileSync(pathToFileURL(file));
try { bytes = gunzipSync(bytes); } catch { /* an unpacked save */ }
const g = engine.Game.load(new Uint8Array(bytes), Number(rd));

const now = () => performance.now();
// Let the first ring load like a menu "Play" would.
let guard = 0;
while (!g.ready() && guard++ < 2000) { g.update(1 / 60); g.begin_work(); while (g.work_step()); drain(); }
function drain() {
  let meshes = 0, quads = 0;
  for (;;) {
    const kind = g.next_event();
    if (kind === 0) break;
    if (kind === 1) {
      const v = new Uint32Array(wasm.memory.buffer, g.mesh_ptr(), g.mesh_len());
      quads += v.length;
      meshes++;
    }
  }
  return [meshes, quads];
}
console.log(`start (${g.player_x().toFixed(0)}, ${g.player_y().toFixed(0)}, ${g.player_z().toFixed(0)}) chunks ${g.chunks_loaded()} instances ${g.instance_count()}`);

if (!g.flying()) g.toggle_fly();
g.set_look(Number(yaw), 0);
g.set_move(1, 0, false, sprint === '1', false);
const frames = [];
const frameCount = Math.round(Number(secs) * 60);
for (let i = 0; i < frameCount; i++) {
  const t0 = now();
  g.update(1 / 60);
  const t1 = now();
  g.begin_work();
  while (g.work_step() && now() - t1 < WORK_BUDGET_MS);
  const t2 = now();
  const [meshes] = drain();
  const t3 = now();
  g.clear_sounds();
  frames.push({ pos: [g.player_x(), g.player_z()], update: t1 - t0, work: t2 - t1, drain: t3 - t2, total: t3 - t0, meshes, instances: g.instance_count() });
}
const pct = (key, p) => frames.map((f) => f[key]).sort((a, b) => a - b)[Math.min(frames.length - 1, Math.floor(frames.length * p))];
const row = (key) => `${key.padEnd(7)} avg ${(frames.reduce((s, f) => s + f[key], 0) / frames.length).toFixed(2)}  p50 ${pct(key, 0.5).toFixed(2)}  p99 ${pct(key, 0.99).toFixed(2)}  max ${pct(key, 1).toFixed(2)} ms`;
for (const k of ['update', 'work', 'drain', 'total']) console.log(row(k));
const over = frames.filter((f) => f.total > 16.6).length;
console.log(`frames over 16.6 ms: ${over}/${frames.length}; over 8 ms: ${frames.filter((f) => f.total > 8).length}; meshes ${frames.reduce((s, f) => s + f.meshes, 0)}; instances avg ${Math.round(frames.reduce((s, f) => s + f.instances, 0) / frames.length)}`);
console.log(`end (${g.player_x().toFixed(0)}, ${g.player_y().toFixed(0)}, ${g.player_z().toFixed(0)}) chunks ${g.chunks_loaded()} pending ${g.chunks_pending()}`);
const worst = frames.map((f, i) => [f.total, i, f]).sort((a, b) => b[0] - a[0]).slice(0, 5);
if (process.env.SPIKES) frames.forEach((f, i) => f.update > 6 && console.log(`  update spike #${i}: ${f.update.toFixed(1)} ms at (${f.pos.map(Math.round)})`));
for (const [t, i, f] of worst) console.log(`  worst #${i}: ${t.toFixed(1)} ms (update ${f.update.toFixed(1)}, work ${f.work.toFixed(1)}, drain ${f.drain.toFixed(1)}, meshes ${f.meshes})`);
