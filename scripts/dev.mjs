// One-command dev loop: builds the engine, serves the web app with Vite, and rebuilds the wasm
// (followed by a full page reload) whenever anything under crates/ changes.
//   npm run dev            release wasm (recommended; the debug build is too slow to play)
//   npm run dev -- --dev   debug wasm
import { watch } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import { buildWasm } from './build-wasm.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const dev = process.argv.includes('--dev');

if (!(await buildWasm({ dev }))) {
  console.error('Initial wasm build failed; fix the errors above and run again.');
  process.exit(1);
}

const server = await createServer({ configFile: path.join(root, 'vite.config.ts') });
await server.listen();
server.printUrls();

let timer = null;
let building = false;
let queued = false;

async function rebuild() {
  if (building) {
    queued = true;
    return;
  }
  building = true;
  console.log('\n[opencraft] Rust changed, rebuilding wasm...');
  const ok = await buildWasm({ dev });
  building = false;
  if (ok) {
    console.log('[opencraft] wasm rebuilt, reloading page');
    server.ws.send({ type: 'full-reload' });
  }
  if (queued) {
    queued = false;
    rebuild();
  }
}

watch(path.join(root, 'crates'), { recursive: true }, (_event, file) => {
  if (!file || !/\.(rs|toml)$/.test(file)) return;
  clearTimeout(timer);
  timer = setTimeout(rebuild, 150);
});
