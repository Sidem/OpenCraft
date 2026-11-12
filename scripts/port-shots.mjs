// Pictures for a blind reader: every multi-block machine with item ports at each of its four turns, seen from where the
// player stands to place it, the page's HUD hidden, so only the 3D view says where items go in and come out.
//   a running game server (npm run dev), then: node scripts/port-shots.mjs <url> <out dir> <ghost|placed|belt>
// `ghost`: the machine held and aimed at the ground (the placement preview); `placed`: built there, hand empty; `belt`:
// built there, a belt held. Writes <block>-<turn>.jpg and truth.json, per shot the machine's name and the roles of the
// ports on each side as the player sees it (`placement_ports`: the same port data the simulation uses). Headless Chrome
// on the GPU, a throwaway profile, no dependencies (as scripts/frame-bench.mjs).
import { spawn } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';

const [url, outDir, mode = 'ghost'] = process.argv.slice(2);
if (!outDir || !['ghost', 'placed', 'belt'].includes(mode)) throw new Error('usage: node scripts/port-shots.mjs <url> <out dir> <ghost|placed|belt>');
const CHROME = 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe';
const [WIDTH, HEIGHT] = [1920, 1080];
// The middle of a 1080p screen, where the machine stands: a player sees it this big.
const CLIP = { x: 320, y: 180, width: 1280, height: 720, scale: 1 };
const wait = (ms) => new Promise((r) => setTimeout(r, ms));
mkdirSync(outDir, { recursive: true });

// ---- the browser (Chrome DevTools Protocol over the built-in WebSocket)
const profile = mkdtempSync(path.join(tmpdir(), 'port-shots-'));
const port = 9800 + Math.floor(Math.random() * 100);
const proc = spawn(CHROME, [`--remote-debugging-port=${port}`, `--user-data-dir=${profile}`, '--headless=new', '--no-first-run',
  `--window-size=${WIDTH},${HEIGHT}`, '--use-angle=d3d11', '--ignore-gpu-blocklist', 'about:blank'], { stdio: 'ignore' });
let target;
for (let i = 0; !target; i++) {
  try {
    target = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((p) => p.type === 'page');
  } catch {
    if (i > 60) throw new Error('no browser');
    await wait(500);
  }
}
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((ok, no) => ((ws.onopen = ok), (ws.onerror = no)));
let nextId = 0;
const pending = new Map();
ws.onmessage = (m) => {
  const d = JSON.parse(m.data);
  if (d.id && pending.has(d.id)) (pending.get(d.id)(d), pending.delete(d.id));
};
const send = (method, params = {}) => new Promise((r) => { const n = ++nextId; pending.set(n, r); ws.send(JSON.stringify({ id: n, method, params })); });
const run = async (expression) => {
  const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
  if (r.result?.exceptionDetails || r.error) throw new Error(JSON.stringify(r.result?.exceptionDetails ?? r.error).slice(0, 400));
  return r.result.result.value;
};

// ---- the scene: a stone floor in the sky over spawn, the player standing south of the anchor looking north at it
await send('Page.enable');
await send('Page.navigate', { url: `${url}?preview&seed=1337&rd=4` });
for (let i = 0; !(await run('!!window.opencraft?.game').catch(() => false)); i++) {
  if (i > 120) throw new Error('the game did not start');
  await wait(500);
}
const SETUP = `(() => {
  const g = opencraft.game;
  document.getElementById('menu')?.classList.add('hidden');
  for (const el of document.body.children) if (el.id !== 'game') el.style.visibility = 'hidden';
  for (let i = 0, n = 0; i < 600 && !g.ready(); i++) { g.update(1 / 60); g.begin_work(); while (g.work_step() && n++ < 200); }
  const [x, y, z] = [Math.floor(g.player_x()), 160, Math.floor(g.player_z())];
  g.fill_blocks(x - 12, y - 1, z - 12, x + 12, y - 1, z + 12, 1);
  g.run_ticks(2);
  return [x, y, z];
})()`;
const [X, Y, Z] = await run(SETUP);
const blocks = await run('Array.from(opencraft.game.footprint_blocks())');

// Puts `item` in hotbar slot 0 and selects it (or empties slot 0 for -1).
const hold = (item) => `(() => {
  const g = opencraft.game;
  const find = () => { for (let s = 0; s < g.inventory_size(); s++) if (g.slot_item(s) === ${item}) return s; return -1; };
  if (${item} >= 0 && find() < 0) { g.give(${item}, 1); g.run_ticks(1); }
  const s = ${item} >= 0 ? find() : -1;
  if (s > 0) { g.click_slot(s); g.run_ticks(1); g.click_slot(0); g.run_ticks(1); g.click_slot(s); g.run_ticks(1); }
  if (${item} < 0 && g.slot_item(0)) { g.click_slot(0); g.run_ticks(1); for (let k = 9; k < g.inventory_size(); k++) if (!g.slot_item(k)) { g.click_slot(k); break; } g.run_ticks(1); }
  g.select_slot(0); g.run_ticks(1);
})()`;
const BELT = await run(`opencraft.game.find_item('Conveyor Belt')[0] ?? opencraft.game.find_item('belt')[0]`);

const truth = [];
// R's turn stays with the player between shots (`place_turn`): count what this page has turned so far.
let turned = 0;
const name = (b) => run(`opencraft.game.item_name(${b})`);
for (const block of blocks) {
  for (let turn = 0; turn < 4; turn++) {
    // Clear the air above the floor, stand south of the anchor, aim at the floor cell under it.
    await run(`opencraft.game.fill_blocks(${X - 12}, ${Y}, ${Z - 12}, ${X + 12}, ${Y + 8}, ${Z + 12}, 0)`);
    if (mode !== 'ghost') await run(`opencraft.game.place_at(${X}, ${Y}, ${Z}, ${block}, ${turn})`);
    await run(hold(mode === 'ghost' ? block : mode === 'belt' ? BELT : -1));
    const ports = await run(`(() => {
      const g = opencraft.game;
      g.teleport(${X + 0.5}, ${Y}, ${Z + 3.5});
      const eye = [${X + 0.5}, ${Y} + 1.62, ${Z + 3.5}], aim = [${X + 0.5}, ${Y}, ${Z + 0.5}];
      const d = aim.map((a, i) => a - eye[i]), len = Math.hypot(...d);
      g.set_look(Math.atan2(d[0], -d[2]), Math.asin(d[1] / len));
      for (let k = 0; k < 4; k++) g.update(1 / 60);
      if (${mode === 'ghost'}) { for (let k = 0; k < ${(turn - turned + 4) % 4}; k++) g.rotate_target(); g.update(1 / 60); }
      return [g.placement_ports ? Array.from(g.placement_ports(${block}, ${turn})) : [], g.line_label()];
    })()`).then(([p, label]) => {
      // The ghost really is at this turn: the HUD's side line (from R's turn) names the same sides as the truth.
      const words = ['in', 'out', 'byproduct'].map((w, f) => [w, [0, 1, 2, 3].filter((s) => p.some((v, i) => i % 2 === 0 && v === s && p[i + 1] === f))]);
      const want = words.filter(([, s]) => s.length).map(([w, s]) => `${w}: ${s.map((k) => ['far', 'right', 'near', 'left'][k]).join(', ')}`).join(' · ');
      // (A town depot's label says where a town may stand instead.)
      if (mode === 'ghost' && want && /(in|out|byproduct): /.test(label) && !label.includes(want)) throw new Error(`block ${block} turn ${turn}: label "${label}" but truth "${want}"`);
      return p;
    });
    if (mode === 'ghost') turned = turn;
    await run('new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(r, 150))))');
    const shot = await send('Page.captureScreenshot', { format: 'jpeg', quality: 82, clip: CLIP });
    const file = `${block}-${turn}.jpg`;
    writeFileSync(path.join(outDir, file), Buffer.from(shot.result.data, 'base64'));
    // `placement_ports`: pairs of (side as seen: 0 far, 1 right, 2 near, 3 left; role: 0 in, 1 out, 2 byproduct).
    const sides = { far: [], right: [], near: [], left: [] };
    const SIDE = ['far', 'right', 'near', 'left'], ROLE = ['input', 'output', 'byproduct'];
    for (let i = 0; i < ports.length; i += 2) if (!sides[SIDE[ports[i]]].includes(ROLE[ports[i + 1]])) sides[SIDE[ports[i]]].push(ROLE[ports[i + 1]]);
    truth.push({ file, block, turn, name: await name(block), sides });
  }
  process.stdout.write('.');
}
writeFileSync(path.join(outDir, 'truth.json'), JSON.stringify(truth, null, 1));
console.log(`\n${truth.length} shots in ${outDir}`);
ws.close();
proc.kill();
await wait(500);
rmSync(profile, { recursive: true, force: true });
