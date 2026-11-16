// Lists the recipe graph's loose ends: what is used but made nowhere, what is made but used nowhere, and what is neither.
//   npm run build:wasm once, then: node scripts/recipe-graph.mjs [--all]
// Reads hand recipes and machine recipes through the web bindings (`web/src/wasm`). Only recipe inputs count as a use, plus
// the science packs labs consume; fuels, placing a building, town orders and drone fuel are outside the graph, so a "made,
// never used" line is a question to answer, not a verdict (buildings and packs are labelled and left out unless --all).
// Use it before adding chemistry: new recipes should land on these gaps, not pile on more stock.
import { readFileSync } from 'node:fs';

const wasm = new URL('../web/src/wasm/', import.meta.url);
const engine = await import(new URL('engine.js', wasm).href);
engine.initSync({ module: readFileSync(new URL('engine_bg.wasm', wasm)) });
const g = engine.Game.new_normal(2024, 2);
const all = process.argv.includes('--all');

const pairs = (flat) => Array.from({ length: flat.length >> 1 }, (_, i) => [flat[2 * i], flat[2 * i + 1]]);
const name = (id) => g.item_name(id);

// Every recipe as { by, ins, outs }: `by` is "hand" or the machine blocks that make it.
const recipes = [];
for (let r = 0; r < g.recipe_count(); r++) {
  recipes.push({ by: 'hand', ins: pairs(g.recipe_inputs(r)), outs: [[g.recipe_output(r), g.recipe_output_count(r)]] });
}
const machines = new Map();
for (let b = 0; b < g.block_count(); b++) {
  for (const i of g.machine_recipes(b)) machines.set(i, [...(machines.get(i) ?? []), g.block_name(b)]);
}
for (const [i, by] of machines) {
  recipes.push({ by: by.join('/'), ins: pairs(g.machine_recipe_inputs(i)), outs: pairs(g.machine_recipe_output(i)) });
}

const made = new Map(); // item -> [recipe]
const used = new Map();
const note = (map, id, r) => map.set(id, [...(map.get(id) ?? []), r]);
for (const r of recipes) {
  for (const [id] of r.outs) note(made, id, r);
  for (const [id] of r.ins) note(used, id, r);
}
const packs = new Set();
for (let t = 0; t < g.tech_count(); t++) for (const p of g.tech_packs(t)) packs.add(p);

const ids = [];
for (let id = 1; id < 4096; id++) if (name(id) !== '') ids.push(id);
const label = (id) => `${name(id)} (${id})`;
const kind = (id) => (packs.has(id) ? 'pack' : g.item_facts(id) !== '' ? 'machine' : id < 256 ? 'block' : 'item');

const section = (title, list, show) => {
  console.log(`\n${title}: ${list.length}`);
  for (const id of list) console.log(`  ${label(id)} [${kind(id)}]${show ? ' ' + show(id) : ''}`);
};

console.log(`${ids.length} named items, ${recipes.length} recipes (${recipes.filter((r) => r.by === 'hand').length} by hand, ${machines.size} in machines), ${packs.size} science packs`);
const byName = (id) => (made.get(id) ?? []).map((r) => r.by).filter((v, i, a) => a.indexOf(v) === i).join(', ');
section(
  'USED BUT MADE BY NO RECIPE (raw: mined, felled, pumped, found)',
  ids.filter((id) => used.has(id) && !made.has(id)),
  (id) => `used by ${[...new Set(used.get(id).map((r) => r.outs.map(([o]) => name(o)).join('+')))].slice(0, 4).join(', ')}`,
);
const dead = ids.filter((id) => made.has(id) && !used.has(id) && (all || (kind(id) !== 'machine' && kind(id) !== 'pack' && kind(id) !== 'block')));
section('MADE BUT USED IN NO RECIPE (a question: fuel, building, goods, or a dead end?)', dead, (id) => `made by ${byName(id)}`);
section('NEITHER MADE NOR USED IN ANY RECIPE', ids.filter((id) => !made.has(id) && !used.has(id) && (all || kind(id) === 'item')));

// Leaves of each kind of machine: what chemistry-like machines take and give, for a quick read of a family.
console.log('\nMACHINE RECIPES BY MACHINE');
const group = new Map();
for (const r of recipes.filter((x) => x.by !== 'hand')) group.set(r.by, [...(group.get(r.by) ?? []), r]);
for (const [by, list] of group) {
  console.log(`  ${by}: ${list.length}`);
  for (const r of list) {
    const f = (l) => l.map(([id, n]) => `${n > 1 ? n + ' ' : ''}${name(id)}`).join(' + ');
    console.log(`    ${f(r.ins)} -> ${f(r.outs)}`);
  }
}
