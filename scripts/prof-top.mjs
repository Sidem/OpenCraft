// Summarises a Chrome .cpuprofile: time by kind (wasm, js, GPU wait, gc) and the top functions by self time.
//   npm run build:wasm -- --profiling   (keeps function names), then:
//   npx vite build --outDir <dir> --emptyOutDir
//   PROFILE=p.cpuprofile node scripts/frame-bench.mjs <dir> chrome gpu <world.ocworld>
//   node scripts/prof-top.mjs p.cpuprofile [rows=25]
import { readFileSync } from 'node:fs';

const [file, rows = '25'] = process.argv.slice(2);
if (!file) throw new Error('usage: node scripts/prof-top.mjs <file.cpuprofile> [rows]');
const profile = JSON.parse(readFileSync(file, 'utf8'));
const nodes = new Map(profile.nodes.map((n) => [n.id, n]));
const self = new Map();
profile.samples.forEach((id, i) => self.set(id, (self.get(id) ?? 0) + (profile.timeDeltas[i] ?? 0)));
const total = [...self.values()].reduce((a, b) => a + b, 0);
const byFunction = new Map();
const byKind = new Map();
for (const [id, us] of self) {
  const { functionName: name, url, lineNumber } = nodes.get(id).callFrame;
  const label = (name || '(anonymous)').replace(/\[[0-9a-f]{16}\]/g, '').replace(/opencraft_engine::/g, '');
  const key = `${label} ${url.split('/').pop()}:${lineNumber + 1}`;
  byFunction.set(key, (byFunction.get(key) ?? 0) + us);
  const kind = name === '(idle)' ? 'idle' : name === '(program)' ? 'native / GPU wait' : name === '(garbage collector)' ? 'gc' : url.includes('wasm') ? 'wasm' : url ? 'js' : 'other';
  byKind.set(kind, (byKind.get(kind) ?? 0) + us);
}
const line = (us, label) => `  ${(us / 1000).toFixed(0).padStart(7)} ms ${((us / total) * 100).toFixed(1).padStart(5)}%  ${label}`;
console.log(`profile ${(total / 1e6).toFixed(1)} s, ${profile.samples.length} samples`);
for (const [kind, us] of [...byKind].sort((a, b) => b[1] - a[1])) console.log(line(us, kind));
console.log('top self time:');
for (const [key, us] of [...byFunction].sort((a, b) => b[1] - a[1]).slice(0, Number(rows))) console.log(line(us, key));
