// Builds crates/engine to web/src/wasm with wasm-pack.
//   node scripts/build-wasm.mjs          optimised release build (default, what you want to play)
//   node scripts/build-wasm.mjs --dev    faster compile, much slower at runtime
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

export function buildWasm({ dev = false } = {}) {
  const args = [
    'build',
    'crates/engine',
    '--target', 'web',
    dev ? '--dev' : '--release',
    '--out-dir', '../../web/src/wasm',
    '--out-name', 'engine',
    '--no-pack',
  ];
  return new Promise((resolve) => {
    const child = spawn('wasm-pack', args, { cwd: root, stdio: 'inherit', shell: process.platform === 'win32' });
    child.on('error', (err) => {
      console.error(`\nCould not run wasm-pack (${err.message}). Install it with: cargo install wasm-pack`);
      resolve(false);
    });
    child.on('exit', (code) => resolve(code === 0));
  });
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const ok = await buildWasm({ dev: process.argv.includes('--dev') });
  process.exit(ok ? 0 : 1);
}
