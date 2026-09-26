// Smoke test for the signalling Worker, local or deployed: asks for ICE servers and a room code, then
// a host and a joiner trade one message each way, and closing the host closes the joiner.
//   npm run smoke -- http://localhost:8787
//   npm run smoke -- https://opencraft-signal.<your-subdomain>.workers.dev
if (typeof WebSocket === 'undefined') {
  console.error('This needs Node 22, or Node 20 with: node --experimental-websocket smoke.mjs <url>');
  process.exit(1);
}
const base = (process.argv[2] ?? 'http://localhost:8787').replace(/\/$/, '');
const wsBase = base.replace(/^http/, 'ws');

const ice = await (await fetch(`${base}/ice`)).json();
const turn = ice.iceServers.some((s) => [s.urls].flat().some((u) => u.startsWith('turn')));
console.log(`ice: ${ice.iceServers.length} server entries, TURN ${turn ? 'included' : 'NOT set up (STUN only)'}`);

const { code } = await (await fetch(`${base}/room`, { method: 'POST' })).json();
console.log(`room: ${code}`);

const host = await open(`${wsBase}/room/${code}?role=host`);
const joiner = await open(`${wsBase}/room/${code}?role=join`);
const welcome = await next(joiner);
const joined = await next(host);
check(welcome.type === 'welcome' && joined.type === 'joined' && joined.peer === welcome.peer, 'join handshake');

host.send(JSON.stringify({ type: 'signal', to: welcome.peer, data: { offer: 'o' } }));
const offer = await next(joiner);
check(offer.from === 'host' && offer.data.offer === 'o', 'host → joiner');
joiner.send(JSON.stringify({ type: 'signal', data: { answer: 'a' } }));
const answer = await next(host);
check(answer.from === welcome.peer && answer.data.answer === 'a', 'joiner → host');

const second = await open(`${wsBase}/room/${code}?role=host`);
const taken = await closed(second);
check(taken.code === 4000, `a second host is refused (${taken.code} ${taken.reason})`);

const joinerClosed = closed(joiner);
host.close(1000, 'done');
const gone = await joinerClosed;
check(gone.code === 4001, `the joiner hears the host left (${gone.code} ${gone.reason})`);
const empty = await closed(await open(`${wsBase}/room/${code}?role=join`));
check(empty.code === 4004, `joining an empty room is refused (${empty.code} ${empty.reason})`);
console.log('smoke test passed');

function open(url) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    ws.queue = [];
    ws.waiting = [];
    ws.onmessage = (e) => {
      const msg = JSON.parse(e.data);
      const w = ws.waiting.shift();
      if (w) w(msg);
      else ws.queue.push(msg);
    };
    ws.onopen = () => resolve(ws);
    ws.onerror = () => reject(new Error(`could not open ${url}`));
  });
}

function next(ws) {
  if (ws.queue.length) return Promise.resolve(ws.queue.shift());
  return timeout(new Promise((resolve) => ws.waiting.push(resolve)), 'a message');
}

function closed(ws) {
  return timeout(new Promise((resolve) => ws.addEventListener('close', (e) => resolve(e))), 'a close');
}

function timeout(promise, what) {
  const t = new Promise((_, reject) => setTimeout(() => reject(new Error(`timed out waiting for ${what}`)), 5000));
  return Promise.race([promise, t]);
}

function check(ok, what) {
  if (!ok) {
    console.error(`FAILED: ${what}`);
    process.exit(1);
  }
  console.log(`ok: ${what}`);
}
