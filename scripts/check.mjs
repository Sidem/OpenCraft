// Every check to run before a commit, with quiet output: one line per step, plus the relevant
// output of any step that fails. Exits 1 if anything failed.
//   npm run check
// Steps: rustfmt, clippy (warnings are errors), engine tests, TypeScript, size budgets.
// TypeScript needs the generated bindings in web/src/wasm (npm run build:wasm).
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const MAX_FAILURE_LINES = 60;
/** Cargo progress chatter that never explains a failure. */
const NOISE = /^\s*(Compiling|Checking|Finished|Running|Doc-tests|Blocking waiting|Updating|Downloaded|Downloading)\b/;

const steps = [
  { name: 'format', cmd: 'cargo', args: ['fmt', '--all', '--check'], hint: 'run `cargo fmt --all`' },
  { name: 'clippy', cmd: 'cargo', args: ['clippy', '--workspace', '--all-targets', '-q', '--', '-D', 'warnings'] },
  { name: 'tests', cmd: 'cargo', args: ['test', '--workspace', '-q'], summary: testSummary },
  { name: 'types', cmd: 'npx', args: ['tsc', '--noEmit'], before: needsBindings },
  { name: 'sizes', cmd: 'node', args: ['scripts/check-size.mjs'], summary: (out) => out.trim().split('\n') },
];

let failed = 0;
for (const step of steps) {
  const blocked = step.before?.();
  if (blocked) {
    failed++;
    console.log(`✗ ${step.name}: ${blocked}`);
    continue;
  }
  const t0 = Date.now();
  const r = spawnSync(step.cmd, step.args, {
    cwd: root,
    encoding: 'utf8',
    shell: process.platform === 'win32',
    maxBuffer: 64 * 1024 * 1024,
  });
  const out = `${r.stdout ?? ''}${r.stderr ?? ''}`;
  const secs = ((Date.now() - t0) / 1000).toFixed(1);
  if (r.status === 0) {
    const extra = step.summary?.(out) ?? [];
    const first = extra.length === 1 ? `: ${extra[0]}` : '';
    console.log(`✓ ${step.name} (${secs} s)${first}`);
    if (extra.length > 1) for (const line of extra) console.log(`    ${line}`);
  } else {
    failed++;
    console.log(`✗ ${step.name} (${secs} s)${step.hint ? `: ${step.hint}` : ''}`);
    const lines = out
      .replace(/\x1b\[[0-9;]*m/g, '')
      .split(/\r?\n/)
      .filter((l) => l.trim() && !NOISE.test(l));
    for (const line of lines.slice(0, MAX_FAILURE_LINES)) console.log(`    ${line}`);
    if (lines.length > MAX_FAILURE_LINES) console.log(`    … ${lines.length - MAX_FAILURE_LINES} more lines`);
  }
}
console.log(failed ? `${failed} check(s) failed` : 'all checks passed');
process.exit(failed ? 1 : 0);

function testSummary(out) {
  let passed = 0;
  for (const m of out.matchAll(/test result: ok\. (\d+) passed/g)) passed += Number(m[1]);
  return [`${passed} passed`];
}

function needsBindings() {
  return fs.existsSync(path.join(root, 'web/src/wasm/engine.d.ts'))
    ? null
    : 'web/src/wasm is missing; run `npm run build:wasm` first';
}
