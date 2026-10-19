// Keeps docs/CODEMAP.md true and the core's layer clean, so the map can be trusted without re-exploring.
//   node scripts/check-map.mjs   (part of npm run check)
// 1. Every path the map names in backticks exists (a bare file name anywhere under its section's root, a path with a
//    slash as the end of some real path there).
// 2. Every module is on the map: each top-level engine module, each file in factory/ and factory/process/, each web/src
//    folder and top-level file, each ui/*.ts.
// 3. The deterministic core never names a module that holds presentation state or the local player's hands (`crate::x`
//    with x in VIEW): that is how view state would leak into the state hash. Stateless helpers (meshing, light, hit
//    boxes, textures) are fine. A core file allowed to name one goes in EXEMPT with the modules and the reason.
// Exits 1 on any failure and prints only the failures.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const ENGINE = 'crates/engine/src';
const SKIP = new Set(['node_modules', 'target', 'dist', '.git', '.claude', 'trailer']);
/** Top-level engine modules that make up the deterministic core. */
const CORE = ['sim', 'action', 'bytes', 'join', 'math', 'deposits', 'daytime', 'inventory', 'equipment', 'tools',
  'crafting', 'perks', 'mode', 'research', 'ghosts', 'drones', 'helpers', 'blueprint', 'satellites', 'pods', 'opening',
  'settlements', 'factory', 'block', 'item', 'recipes', 'chunk', 'world', 'worldgen', 'noise'];
/** Top-level engine modules the core must never name: presentation state, the local player's hands, the API. */
const VIEW = ['api', 'interaction', 'camera', 'strategy', 'avatars', 'analytics', 'minimap', 'sound', 'orbit_fx',
  'drone_view', 'surroundings', 'quarry_preview', 'footprint_preview', 'town_preview', 'ghost_mode', 'upgrade_aim',
  'belt_line', 'power_tools', 'rail_tools', 'train_tools', 'cargo_tools', 'site_hands', 'helper_hands', 'hand_waste',
  'survey', 'prospect', 'events', 'entities', 'authority'];
/** Core files allowed to name some VIEW modules, and why. */
const EXEMPT = {
  'blueprint/hands.rs': { may: VIEW, why: 'the local player blueprint hands, not core' },
  'blueprint/repeat.rs': { may: VIEW, why: 'the blueprint prompt, presentation' },
  'factory/belt.rs': { may: ['entities'], why: 'draws belt items with the loose-item model (push_item_model)' },
  'factory/router.rs': { may: ['entities'], why: 'draws the held item with the loose-item model (push_item_model)' },
};

const failures = [];
const rel = (p) => path.relative(root, p).split(path.sep).join('/');
function* walk(dir) {
  if (!fs.existsSync(dir)) return;
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    if (SKIP.has(e.name)) continue;
    const p = path.join(dir, e.name);
    yield { p, dir: e.isDirectory() };
    if (e.isDirectory()) yield* walk(p);
  }
}

/** Every file and folder under `dir` as '/repo/relative/path' (folders end in '/'). */
function paths(dir) {
  return [...walk(dir)].map(({ p, dir: d }) => '/' + rel(p) + (d ? '/' : ''));
}
let all;
const everywhere = () => (all ??= paths(root));

// 1. Paths named on the map exist.
const map = fs.readFileSync(path.join(root, 'docs/CODEMAP.md'), 'utf8');
const sections = [
  { title: '## Engine', base: ENGINE },
  { title: '## Web host', base: 'web/src' },
  { title: '## Outside the game', base: '.' },
  { title: '## Where the tuning knobs live', base: ENGINE },
];
const textOf = {};
for (const [i, s] of sections.entries()) {
  const start = map.indexOf(s.title);
  if (start < 0) {
    failures.push(`map: section "${s.title}" missing`);
    continue;
  }
  const next = sections.slice(i + 1).map((n) => map.indexOf(n.title)).filter((n) => n > start);
  const text = map.slice(start, next.length ? Math.min(...next) : map.length);
  textOf[s.base] = (textOf[s.base] ?? '') + text;
  const here = paths(path.join(root, s.base));
  for (const [, token] of text.matchAll(/`([^`\s*<>]+)`/g)) {
    if (!/(\.(rs|ts|css|md|mjs|py|yml|json|jsonc|ogg)|\/)$/.test(token) || token.startsWith('http')) continue;
    const want = token.replace(/^\.\//, '');
    const found = here.some((p) => p.endsWith('/' + want)) || everywhere().some((p) => p.endsWith('/' + want));
    if (!found) failures.push(`map: \`${token}\` (under ${s.base}) does not exist`);
  }
}

// 2. Every module is on the map.
const mentions = (base, name) => (textOf[base] ?? '').includes(name);
const list = (dir) => (fs.existsSync(path.join(root, dir)) ? fs.readdirSync(path.join(root, dir), { withFileTypes: true }) : []);
for (const e of list(ENGINE)) {
  const name = e.isDirectory() ? `${e.name}/` : e.name;
  if (e.name === 'tests' || e.name === 'tests.rs') continue;
  const stem = e.name.replace(/\.rs$/, '');
  if (!mentions(ENGINE, `\`${name}\``) && !mentions(ENGINE, `\`${stem}.rs\``) && !mentions(ENGINE, `\`${stem}/`))
    failures.push(`map: engine module ${name} is not on the map`);
}
for (const dir of ['factory', 'factory/process']) {
  for (const e of list(`${ENGINE}/${dir}`)) {
    if (!e.isFile() || e.name === 'tests.rs' || e.name === 'mod.rs') continue;
    if (!mentions(ENGINE, e.name)) failures.push(`map: ${dir}/${e.name} is not on the map`);
  }
}
for (const e of list('web/src')) {
  if (e.name === 'CLAUDE.md') continue;
  if (!mentions('web/src', e.isDirectory() ? `\`${e.name}/` : `\`${e.name}\``)) failures.push(`map: web/src/${e.name} is not on the map`);
}
for (const e of list('web/src/ui')) {
  if (e.name.endsWith('.ts') && !mentions('web/src', e.name)) failures.push(`map: web/src/ui/${e.name} is not on the map`);
}

// 3. The core names no presentation module.
const view = new Set(VIEW);
for (const top of CORE) {
  const file = path.join(root, ENGINE, `${top}.rs`);
  const inside = [...walk(path.join(root, ENGINE, top))].map(({ p }) => p).filter((p) => p.endsWith('.rs'));
  const files = [...(fs.existsSync(file) ? [file] : []), ...inside];
  for (const f of files) {
    const r = rel(f).slice(ENGINE.length + 1);
    if (r.endsWith('tests.rs') || r.includes('/tests/')) continue;
    const src = fs.readFileSync(f, 'utf8');
    const named = [...src.matchAll(/crate::(\w+)/g)].map((m) => m[1]);
    for (const [, group] of src.matchAll(/crate::\{([^}]*)\}/g)) named.push(...group.split(',').map((g) => g.trim().split('::')[0]));
    for (const n of new Set(named)) if (view.has(n) && !EXEMPT[r]?.may.includes(n)) failures.push(`layer: core file ${r} names crate::${n}`);
  }
}

for (const line of failures) console.log(line);
console.log(failures.length ? `${failures.length} map or layer problem(s)` : 'code map and core layer ok');
process.exit(failures.length ? 1 : 0);
