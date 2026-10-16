// Prints what a player's save says about how they play: time, research, the factory and the inventory.
//   npm run build:wasm once, then: node scripts/inspect-save.mjs <world.ocworld>
// Loads the save headlessly through the web bindings (`web/src/wasm`) and only reads it: nothing is written back.
// Machines are counted by the analytics after one simulated second, with what holds each kind back. To add a line:
// call another `Game` getter (`web/src/wasm/engine.d.ts` lists them).
import { readFileSync } from 'node:fs';
import { gunzipSync } from 'node:zlib';
import { pathToFileURL } from 'node:url';

const file = process.argv[2];
if (!file) throw new Error('usage: node scripts/inspect-save.mjs <world.ocworld>');
const wasm = new URL('../web/src/wasm/', import.meta.url);
const engine = await import(new URL('engine.js', wasm).href);
engine.initSync({ module: readFileSync(new URL('engine_bg.wasm', wasm)) });
let bytes = readFileSync(pathToFileURL(file));
try {
  bytes = gunzipSync(bytes);
} catch {
  // An unpacked save: use the bytes as they are.
}
const g = engine.Game.load(new Uint8Array(bytes), 4);
const name = (id) => g.item_name(id);
const hours = (s) => (s / 3600).toFixed(1);

console.log(`seed ${g.seed()} · ${g.is_creative() ? 'creative' : 'normal'} · day ${g.day_number()} · ${hours(g.play_seconds())} h played`);

const done = [];
const open = [];
const demand = new Map();
let labSeconds = 0;
for (let t = 0; t < g.tech_count(); t++) {
  if (g.tech_done(t)) done.push(g.tech_name(t));
  else if (g.tech_available(t)) open.push(`${g.tech_name(t)} ${Math.round(g.tech_progress(t) * 100)}%`);
  if (g.tech_endless(t)) continue;
  labSeconds += g.tech_units(t) * g.tech_seconds(t);
  for (const pack of g.tech_packs(t)) demand.set(pack, (demand.get(pack) ?? 0) + g.tech_units(t));
}
console.log(`\ntechs done ${done.length}/${g.tech_count()}: ${done.join(', ')}`);
console.log(`available: ${open.join(', ') || 'none'}`);
const queue = Array.from({ length: g.research_queue_len() }, (_, i) => g.tech_name(g.research_queue_at(i)));
console.log(`queue: ${queue.join(', ') || 'empty'}`);
const packs = [...demand].map(([pack, n]) => `${n} ${name(pack)}`);
console.log(`the whole tree (no endless techs): ${packs.join(', ')}; ${hours(labSeconds)} lab-hours at Mk1 speed`);

g.run_ticks(60);
const causes = ['power', 'input', 'output', 'fuel', 'water', 'deposit', 'other'];
const [working, full, ...held] = g.analytics_machines();
const heldText = causes.map((c, i) => (held[i] ? `${held[i]} ${c}` : '')).filter(Boolean).join(', ');
console.log(`\nmachines: ${working} working, ${full} at full speed; held back by ${heldText || 'nothing'}`);
const fields = g.analytics_type_fields();
const rows = g.analytics_machine_types();
const kinds = [];
for (let i = 0; i < rows.length; i += fields) kinds.push(`${name(rows[i])} ${rows[i + 1] + rows[i + 4]} (${rows[i + 3]}%)`);
console.log(`by kind (count, average speed): ${kinds.join('; ')}`);
console.log(`deposits touched: ${g.deposits_tracked()}`);

const carried = new Map();
for (let s = 0; s < g.inventory_size(); s++) {
  const item = g.slot_item(s);
  if (item) carried.set(item, (carried.get(item) ?? 0) + g.slot_count(s));
}
const used = [...carried.keys()].length;
const list = [...carried].sort((a, b) => b[1] - a[1]).map(([item, n]) => `${name(item)} ×${n}`);
console.log(`\ninventory (${g.inventory_size()} slots, ${used} kinds; tools count uses left): ${list.join(', ')}`);
g.free();
