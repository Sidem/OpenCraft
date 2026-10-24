// Checks the machine-stopped lessons against real saves: a player who already knows their factory must see none.
//   npm run build:wasm once, then: node scripts/check-lessons.mjs <world.ocworld>...
// Each save is loaded headlessly (read-only), run five simulated minutes with the player where the save left them, and
// asked every second which lesson is on show (`lesson_now`). Older saves count every stall lesson as taught (save 58),
// so any lesson that fires is a failure; the stalled-machine count printed beside it is what would have fired without
// that. A fresh world with its opening scene is run the same way: its idle furnace must stay quiet too.
// Exit code 1 when a lesson showed anywhere. To check a new lesson: it comes through `lesson_now` like the others.
import { readFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';
import { pathToFileURL } from 'node:url';

const files = process.argv.slice(2);
if (!files.length) throw new Error('usage: node scripts/check-lessons.mjs <world.ocworld>...');
const wasm = new URL('../web/src/wasm/', import.meta.url);
const engine = await import(new URL('engine.js', wasm).href);
engine.initSync({ module: readFileSync(new URL('engine_bg.wasm', wasm)) });

const CAUSES = ['power', 'input', 'output', 'fuel', 'water', 'deposit', 'other'];
const MINUTES = 5;
let failed = false;

function run(label, g) {
  g.run_ticks(60);
  const [working, , ...held] = g.analytics_machines();
  const stalled = CAUSES.map((c, i) => (held[i] ? `${held[i]} ${c}` : '')).filter(Boolean).join(', ');
  const fired = new Set();
  for (let s = 0; s < MINUTES * 60; s++) {
    g.run_ticks(60);
    const now = g.lesson_now();
    if (now) fired.add(`${CAUSES[now - 1]}: ${g.lesson_text()}`);
  }
  console.log(`${label}: ${working} machines working, held back by ${stalled || 'nothing'}`);
  console.log(fired.size ? `  FAILED: a lesson showed in ${MINUTES} minutes:\n    ${[...fired].join('\n    ')}` : `  quiet for ${MINUTES} minutes`);
  failed ||= fired.size > 0;
  g.free();
}

for (const file of files) {
  let bytes = readFileSync(pathToFileURL(file));
  try {
    bytes = gunzipSync(bytes);
  } catch {
    // An unpacked save: use the bytes as they are.
  }
  run(file.split(/[\\/]/).pop(), engine.Game.load(new Uint8Array(bytes), 4));
}
run('a fresh world, idle', engine.Game.new_normal(1337, 4));
process.exit(failed ? 1 : 0);
