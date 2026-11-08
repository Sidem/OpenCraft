// What each graphics knob buys, measured as real frame intervals in a real browser (Chrome, Edge or Firefox) on a real save, with
// the frame-rate cap off so a fast GPU's numbers mean something. Uses a throwaway profile and no dependencies.
//   npx vite build --outDir <dir> --emptyOutDir
//   node scripts/frame-bench.mjs <dir> <chrome|edge|firefox> <gpu|heavy|software> <world.ocworld> [out.json]
// `software` forces the browser's software renderer, the worst case for a player whose graphics acceleration is off; `heavy`
// (Chromium only) draws so many pixels that a fast GPU is the limit. Neither is an integrated GPU. Serve from a build, not the dev server, or editing a file reloads the page mid-run. Knobs that have no
// setting are applied from outside: a replaced `renderer.resize` (scale), a blur style, `?rd=`, and a
// preload script that turns antialiasing off. To add a row: a line in `SAME_PAGE` or `RELOADED`.
import { spawn, spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import http from 'node:http';
import { tmpdir } from 'node:os';
import path from 'node:path';

const [dist, browser, mode, save, out] = process.argv.slice(2);
if (!save) throw new Error('usage: node scripts/frame-bench.mjs <dist> <chrome|edge|firefox> <gpu|software> <world.ocworld> [out.json]');
const SECONDS = mode === 'gpu' ? 8 : 10;
const WARM_MS = 1500;
const WIDTH = 1920, HEIGHT = 1080;
const PROGRAMS = {
  chrome: 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
  edge: 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe',
  firefox: 'C:\\Program Files\\Mozilla Firefox\\firefox.exe',
};
const wait = (ms) => new Promise((r) => setTimeout(r, ms));

// ---- the build, served on a free port
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.png': 'image/png', '.opus': 'audio/ogg', '.ogg': 'audio/ogg', '.json': 'application/json' };
const server = http.createServer((req, res) => {
  const rel = decodeURIComponent(new URL(req.url, 'http://x').pathname).replace(/^\/+/, '') || 'index.html';
  try {
    const body = readFileSync(path.join(dist, rel));
    res.writeHead(200, { 'content-type': TYPES[path.extname(rel)] ?? 'application/octet-stream' }).end(body);
  } catch {
    res.writeHead(404).end();
  }
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const base = `http://127.0.0.1:${server.address().port}/`;

// ---- the browser
const PRELOAD = `() => {
  try { localStorage.setItem('opencraft.quality', '{"auto":false}'); } catch {} // the bench sets the quality itself
  if (!location.search.includes('benchaa=off')) return;
  const get = HTMLCanvasElement.prototype.getContext;
  HTMLCanvasElement.prototype.getContext = function (type, attrs) { return get.call(this, type, type === 'webgl2' ? { ...attrs, antialias: false } : attrs); };
}`;

async function connect(url) {
  const ws = new WebSocket(url);
  await new Promise((ok, no) => ((ws.onopen = ok), (ws.onerror = no)));
  let id = 0;
  const pending = new Map();
  ws.onmessage = (m) => {
    const d = JSON.parse(m.data);
    if (d.id && pending.has(d.id)) (pending.get(d.id)(d), pending.delete(d.id));
  };
  return { ws, send: (method, params = {}) => new Promise((r) => { const n = ++id; pending.set(n, r); ws.send(JSON.stringify({ id: n, method, params })); }) };
}

async function retry(fn, tries = 80) {
  for (let i = 0; ; i++) {
    try {
      return await fn();
    } catch (err) {
      if (i >= tries) throw err;
      await wait(500);
    }
  }
}

async function chromium(profile, port) {
  const flags = mode === 'software' ? ['--use-angle=swiftshader', '--enable-unsafe-swiftshader'] : ['--use-angle=d3d11', '--ignore-gpu-blocklist'];
  if (mode === 'heavy') flags.push('--force-device-scale-factor=2'); // four times the pixels twice over: a fast GPU made pixel-bound, like a weak one
  const proc = spawn(PROGRAMS[browser], [`--remote-debugging-port=${port}`, `--user-data-dir=${profile}`, '--headless=new', '--no-first-run', '--no-default-browser-check',
    mode === 'heavy' ? '--window-size=3840,2160' : `--window-size=${WIDTH},${HEIGHT}`,
    // Uncapped, a software renderer queues frames faster than it can draw them and stalls for seconds; it never reaches the cap anyway.
    ...(mode === 'software' ? [] : ['--disable-gpu-vsync', '--disable-frame-rate-limit']), '--disable-background-timer-throttling', '--disable-renderer-backgrounding', '--disable-backgrounding-occluded-windows', ...flags, 'about:blank'], { stdio: 'ignore' });
  const target = await retry(async () => {
    const t = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((p) => p.type === 'page');
    if (!t) throw new Error('no page');
    return t;
  });
  const { ws, send } = await connect(target.webSocketDebuggerUrl);
  await send('Page.enable');
  await send('Page.addScriptToEvaluateOnNewDocument', { source: `(${PRELOAD})()` });
  return {
    proc, ws,
    nav: (url) => send('Page.navigate', { url }),
    run: async (expression) => {
      const r = await send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true });
      if (r.result?.exceptionDetails || r.error) throw new Error(JSON.stringify(r.result?.exceptionDetails ?? r.error).slice(0, 400));
      return r.result.result.value;
    },
  };
}

async function firefox(profile, port) {
  const prefs = { 'layout.frame_rate': 0, 'datareporting.policy.dataSubmissionEnabled': false, 'toolkit.telemetry.reportingpolicy.firstRun': false, 'browser.shell.checkDefaultBrowser': false };
  if (mode === 'software') Object.assign(prefs, { 'webgl.angle.force-warp': true, 'gfx.webrender.software': true });
  writeFileSync(path.join(profile, 'user.js'), Object.entries(prefs).map(([k, v]) => `user_pref(${JSON.stringify(k)}, ${JSON.stringify(v)});`).join('\n'));
  // A window, not --headless: headless Firefox composites in software, which made one blur look like 80 ms.
  const proc = spawn(PROGRAMS.firefox, ['--no-remote', '--new-instance', '--profile', profile, `--remote-debugging-port=${port}`, '--width', String(WIDTH), '--height', String(HEIGHT), 'about:blank'], { stdio: 'ignore' });
  const { ws, send } = await retry(() => connect(`ws://127.0.0.1:${port}/session`));
  await send('session.new', { capabilities: {} });
  const context = (await send('browsingContext.getTree', {})).result.contexts[0].context;
  await send('script.addPreloadScript', { functionDeclaration: PRELOAD });
  return {
    proc, ws,
    nav: (url) => send('browsingContext.navigate', { context, url, wait: 'none' }),
    run: async (expression) => {
      const r = await send('script.evaluate', { expression, target: { context }, awaitPromise: true });
      if (r.type === 'error' || r.result?.type === 'exception') throw new Error(JSON.stringify(r).slice(0, 400));
      return r.result.result.value;
    },
  };
}

const profile = mkdtempSync(path.join(tmpdir(), 'frame-bench-'));
const port = 9400 + Math.floor(Math.random() * 400);
const page = await (browser === 'firefox' ? firefox : chromium)(profile, port);
const run = page.run;

// ---- steps
const until = async (expression, seconds, what) => {
  const end = Date.now() + seconds * 1000;
  while (Date.now() < end) {
    if (await run(expression).catch(() => false)) return;
    await wait(500);
  }
  throw new Error(`timed out waiting for ${what}`);
};

/** Opens the page and finishes the world's streaming by hand: at a software renderer's frame rate it would take many minutes. */
async function load(query = '') {
  await page.nav(base + query);
  await wait(1500);
  await until(`!!(window.opencraft && opencraft.game.ready())`, 600, 'the world');
  await run(`opencraft.menu.el.classList.add('hidden'); undefined`);
  // Finished meshes reach the renderer a few a frame, too slowly at a software frame rate: hand them over here (as light-bench.js).
  await run(`(async () => {
    const { game: g, renderer: r, wasm } = opencraft, end = performance.now() + 120000;
    const drain = () => { for (let k; (k = g.next_event()) !== 0; ) {
      const [x, y, z] = [g.event_x(), g.event_y(), g.event_z()];
      if (k === 1) r.upsertChunk(x, y, z, new Uint32Array(wasm.memory.buffer, g.mesh_ptr(), g.mesh_len()), g.mesh_opaque_quads(), g.mesh_cutout_quads(), g.mesh_liquid_quads(),
        new Uint32Array(wasm.memory.buffer, g.mesh_shaped_ptr(), g.mesh_shaped_len()), g.mesh_shaped_opaque(), g.mesh_shaped_cutout());
      else r.removeChunk(x, y, z);
    } };
    do { g.update(1 / 60); g.begin_work(); while (g.work_step()); drain(); await new Promise((s) => setTimeout(s, 50)); } while (g.chunks_pending() > 0 && performance.now() < end);
    drain();
  })()`);
  await wait(1000);
}

async function importSave() {
  await page.nav(base);
  await wait(1500);
  await until(`!!(window.opencraft && document.querySelector('#worlds input[type=file]'))`, 120, 'the worlds panel');
  const b64 = readFileSync(save).toString('base64');
  await run(`(() => {
    const dt = new DataTransfer();
    dt.items.add(new File([Uint8Array.from(atob('${b64}'), (c) => c.charCodeAt(0))], 'bench.ocworld'));
    const input = document.querySelector('#worlds input[type=file]');
    input.files = dt.files;
    input.dispatchEvent(new Event('change'));
  })()`);
  await until(`[...document.querySelectorAll('#worlds .world')].some((w) => w.textContent.includes('bench'))`, 60, 'the import');
  await run(`[...document.querySelectorAll('#worlds .world')].find((w) => w.textContent.includes('bench')).querySelector('button').click(); undefined`);
  await until(`document.getElementById('world-name')?.textContent === 'bench' && !!window.opencraft`, 120, 'the imported world');
}

async function measure(name, rows, traces) {
  await wait(WARM_MS);
  const json = await run(`new Promise((done) => {
    const a = []; let last = null; const start = performance.now();
    const f = (t) => { if (last !== null) a.push(+(t - last).toFixed(2)); last = t; if (performance.now() - start < ${SECONDS * 1000}) requestAnimationFrame(f); else done(JSON.stringify(a)); };
    requestAnimationFrame(f);
  })`);
  const a = JSON.parse(json);
  const s = [...a].sort((x, y) => x - y);
  const at = (q) => s[Math.min(s.length - 1, Math.ceil(q * s.length) - 1)];
  const row = { name, frames: a.length, p50: at(0.5), p95: at(0.95), p99: at(0.99) };
  rows.push(row);
  traces[name] = a.slice(0, 600);
  console.log(`${browser} ${mode}  ${name.padEnd(24)} ${String(row.frames).padStart(6)} frames  p50 ${row.p50.toFixed(1).padStart(7)}  p95 ${row.p95.toFixed(1).padStart(7)}  p99 ${row.p99.toFixed(1).padStart(7)} ms`);
}

const SCALE = (s) => `(() => {
  const r = opencraft.renderer;
  r.resize = function () { const c = this.canvas, d = Math.min(devicePixelRatio, 2) * ${s}; const w = Math.round(c.clientWidth * d), h = Math.round(c.clientHeight * d); if (c.width !== w || c.height !== h) { c.width = w; c.height = h; } };
})(); undefined`;
const MENU_ON = `opencraft.menu.el.classList.remove('hidden'); undefined`;
const MENU_OFF = `opencraft.menu.el.classList.add('hidden'); undefined`;
/** Each is measured after running its code, in this order, on one loaded page. */
const SAME_PAGE = [
  ['playing', 'undefined'],
  ['launcher, blurred', MENU_ON],
  ['launcher, no blur', `opencraft.menu.el.style.backdropFilter = 'none'; undefined`],
  ['launcher, world not drawn', `opencraft.menu.el.style.backdropFilter = ''; opencraft.renderer.render = () => {}; undefined`],
  ['inventory open', `delete opencraft.renderer.render; ${MENU_OFF.replace('undefined', '')} opencraft.inventory.open(); undefined`],
  ['render scale 75%', `opencraft.inventory.close?.(); ${SCALE(0.75)}`],
  ['render scale 50%', SCALE(0.5)],
  ['playing again', `delete opencraft.renderer.resize; undefined`],
];
const RELOADED = [
  ['render distance 6', '?rd=6'],
  ['render distance 4', '?rd=4'],
  ['antialiasing off', '?benchaa=off'],
];

const rows = [], traces = {};
let gpu = null, stats = null;
try {
  await importSave();
  await load();
  gpu = JSON.parse(await run(`JSON.stringify(opencraft.perf.gpu)`));
  stats = JSON.parse(await run(`JSON.stringify({ ...opencraft.renderer.stats, drawn: [document.getElementById('game').width, document.getElementById('game').height], chunks: opencraft.game.chunks_loaded(), belts: opencraft.game.belts() })`));
  console.log(browser, mode, JSON.stringify(gpu), JSON.stringify(stats));
  for (const [name, code] of SAME_PAGE) {
    await run(code);
    await measure(name, rows, traces);
  }
  for (const [name, query] of RELOADED) {
    await load(query);
    await measure(name, rows, traces);
  }
} finally {
  if (out) writeFileSync(out, JSON.stringify({ browser, mode, gpu, stats, seconds: SECONDS, rows, traces }));
  page.ws.close();
  // By the profile's name, not the pid: Firefox's first process hands over to another, and a survivor spoils every later run.
  spawnSync('powershell', ['-NoProfile', '-Command', `Get-CimInstance Win32_Process | ? { $_.CommandLine -like '*${path.basename(profile)}*' -and $_.Name -ne 'powershell.exe' } | % { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }`]);
  server.close();
  await wait(1500);
  try { rmSync(profile, { recursive: true, force: true, maxRetries: 3 }); } catch { /* the OS will clear its temp folder */ }
}
process.exit(0);
