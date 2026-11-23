// Lists the recipe graph's loose ends: what is used but made nowhere, what is made but used nowhere, and what is neither.
//   npm run build:wasm once, then: node scripts/recipe-graph.mjs [--all] [--pull]
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

if (process.argv.includes('--pull')) {
  pull();
  process.exit(0);
}

// --pull: what still wants each material once the research is done and the base is built. A material is anything a
// recipe, a town or a fuel slot takes; it reaches wants through the recipes that use it. Lasting wants: a town's order or
// upkeep, a fuel slot, a block the player builds with. Finite ones: a science pack (the tree ends), a machine, worn gear,
// a launch (a recipe that gives back only empty barrels). A material with no lasting want runs dry after the build-out.
function pull() {
  const town = new Set(g.town_goods());
  const fuel = new Set();
  const placed = new Set();
  for (let b = 0; b < g.block_count(); b++) placed.add(g.block_item(b));
  const byName = new Map(ids.map((id) => [name(id), id]));
  for (const id of ids) {
    for (const row of g.item_facts(id).split('\n')) {
      const [k, , v] = row.split('\t');
      if (k === 'fuel') for (const n of v.split(', ')) if (byName.has(n)) fuel.add(byName.get(n));
    }
  }
  const empty = byName.get('Empty Barrel');
  // What uses an item, as the guide says (it knows that every wood works as a log and charcoal as coal).
  const hand = recipes.slice(0, g.recipe_count());
  const inMachine = new Map([...machines.keys()].map((i, k) => [i, recipes[g.recipe_count() + k]]));
  const usesOf = (id) => pairs(g.item_uses(id)).map(([m, r]) => (m ? inMachine.get(r) : hand[r]));
  const isMachine = (id) => g.item_facts(id) !== '' || g.item_tier(id) >= 0;
  // Spent by a building outside every recipe and fuel slot: a pod launcher uses up a drop pod a send (`factory/process/pods.rs`).
  const spent = new Set([byName.get('Drop Pod')]);
  const own = (id) => {
    const w = [];
    if (spent.has(id)) w.push('spent');
    if (town.has(id)) w.push('town');
    if (fuel.has(id)) w.push('fuel');
    if (packs.has(id)) w.push('lab');
    if (g.gear_blurb(id) !== '') w.push('gear');
    if (isMachine(id)) w.push('machine');
    else if (placed.has(id)) w.push('build');
    return w;
  };
  const wants = (id, seen = new Set()) => {
    const out = new Set(own(id));
    seen.add(id);
    for (const r of usesOf(id)) {
      if (r.outs.every(([o]) => o === empty)) out.add('launch');
      // A barrel given back is the container, not a product: never follow it.
      for (const [o] of r.outs) if (o !== empty && !seen.has(o)) for (const w of wants(o, seen)) out.add(w);
    }
    return out;
  };
  const LASTING = ['town', 'fuel', 'build', 'spent'];
  const materials = ids.filter((id) => usesOf(id).length > 0 || town.has(id) || fuel.has(id));
  // Machines and gear are ends in themselves (an upgrade recipe consumes them, but that is not why they are made).
  const parts = materials.filter((id) => own(id).length === 0 || town.has(id) || fuel.has(id));
  const dry = parts.filter((id) => ![...wants(id)].some((w) => LASTING.includes(w)));
  console.log(`\n${parts.length} materials (not machines, gear or packs); lasting wants: ${LASTING.join(', ')}`);
  section('NO LASTING WANT (runs dry once the tree and the base are done)', dry, (id) => `[${[...wants(id)].join(', ')}]`);
  const known = (id) => made.has(id) || g.item_ways(id) !== '';
  const loose = ids.filter((id) => known(id) && !materials.includes(id) && own(id).length === 0 && g.tool_uses(id) <= 0);
  section('MADE, AND WANTED BY NOTHING THE TABLES KNOW (vehicles and the like, or a dead end)', loose);
}

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
