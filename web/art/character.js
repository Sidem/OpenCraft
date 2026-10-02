// Save-free character inspection using real engine input, articulation and the production renderer.
// A quiet presentation stage replaces terrain here; the game link opens the ordinary playable world.
import init, { Game } from '../src/wasm/engine.js';
import { Renderer } from '../src/render/renderer.ts';

const status = document.querySelector('#status');
try {
  const wasm = await init();
  const game = new Game(2024, 2);
  const stride = game.instance_floats();
  const renderer = new Renderer(document.querySelector('#scene'), 2, stride);
  renderer.setTextures(new Uint8Array(wasm.memory.buffer, game.texture_ptr(), game.texture_byte_len()),
    game.texture_size(), game.texture_layers());
  renderer.fovY = 48 * Math.PI / 180;
  game.set_third_person(4);
  game.give(267, 600); // Iron pickaxe; selected in mining poses.
  game.give(1, 64); // Stone; selected in placement poses.
  for (let i = 0; i < 3; i++) {
    game.update(1 / 60);
    game.begin_work();
    while (game.work_step());
  }
  // Find a clear, level patch; the spawn's nearby trees can pull the real camera into the head.
  const columns = new Map();
  const top = (x, z) => {
    const key = `${x},${z}`;
    if (!columns.has(key)) {
      let y = 120;
      while (y > 0 && game.block_at(x, y, z) === 0) y--;
      columns.set(key, { y, block: game.block_at(x, y, z) });
    }
    return columns.get(key);
  };
  let patch = null, score = Infinity;
  for (let x = -28; x <= 28; x += 4) for (let z = -20; z <= 28; z += 4) {
    const ground = top(x, z);
    if (!['Grass', 'Sand', 'Stone'].includes(game.block_name(ground.block))) continue;
    let variation = 0;
    for (let dx = -3; dx <= 3; dx++) for (let dz = -8; dz <= 4; dz++) {
      const height = top(x + dx, z + dz).y;
      if (Math.abs(dx) <= 2 && dz >= -3 && height > ground.y) variation = Infinity;
      variation += Math.abs(height - ground.y);
    }
    if (variation < score) { patch = [x + 0.5, ground.y + 1, z + 0.5]; score = variation; }
  }
  if (!patch) {
    let highest = 0;
    for (let x = -12; x <= 12; x++) for (let z = -12; z <= 12; z++) {
      if (top(x, z).y > highest) { highest = top(x, z).y; patch = [x + 0.5, highest + 1, z + 0.5]; }
    }
  }
  if (patch) game.teleport(...patch);
  for (let i = 0; i < 60; i++) game.update(1 / 60);
  let pose = 'idle', view = 'front', clock = 0, last = performance.now();
  document.querySelector('#aim').addEventListener('change', (event) => game.set_look(0, Number(event.target.value)));
  const start = [game.player_x(), game.player_y(), game.player_z()];
  document.querySelectorAll('[data-pose]').forEach((button) => button.addEventListener('click', () => {
    pose = button.dataset.pose;
    clock = 0;
    game.teleport(...start);
    game.select_slot(pose === 'place' ? 1 : 0);
    document.querySelectorAll('[data-pose]').forEach((b) => b.setAttribute('aria-pressed', String(b === button)));
  }));
  document.querySelectorAll('[data-view]').forEach((button) => button.addEventListener('click', () => {
    view = button.dataset.view;
    renderer.fovY = (view === 'shoulder' ? 72 : 48) * Math.PI / 180;
    document.querySelector('#crosshair').style.display = view === 'shoulder' ? 'block' : 'none';
    document.querySelectorAll('[data-view]').forEach((b) => b.setAttribute('aria-pressed', String(b === button)));
  }));

  function frame(now) {
    const dt = Math.min((now - last) / 1000, 0.05);
    last = now;
    clock += dt;
    const moving = ['walk', 'sprint', 'sneak'].includes(pose);
    // Walk short lengths on the real spawn ground; return to the start between demonstrations.
    if (clock > 1.6 && moving) { game.teleport(...start); clock = 0; }
    game.set_move(moving ? 1 : 0, 0, pose === 'jump' && clock % 1.4 < 0.12,
      pose === 'sprint', pose === 'crouch' || pose === 'sneak');
    game.set_mining(pose === 'mine');
    game.set_using(pose === 'place');
    game.update(dt);
    game.begin_work();
    for (let i = 0; i < 8 && game.work_step(); i++);
    const source = [game.eye_x(), game.eye_y(), game.eye_z()];
    const feet = [game.player_x(), game.player_y(), game.player_z()];
    const yaw = view === 'front' ? Math.PI + 0.35 : view === 'back' ? -0.35 : game.yaw();
    const pitch = view === 'shoulder' ? game.pitch() : -0.12;
    const distance = 3.6;
    const eye = view === 'shoulder' ? source : [feet[0] - Math.sin(yaw) * distance,
      feet[1] + 0.8 - Math.sin(pitch) * distance, feet[2] + Math.cos(yaw) * distance];
    const count = game.instance_count();
    const boxes = new Float32Array((count + 2) * stride);
    boxes.set(new Float32Array(wasm.memory.buffer, game.instance_ptr(), count * stride));
    for (let i = 0; i < count * stride; i += stride) {
      for (let a = 0; a < 3; a++) boxes[i + a] += source[a] - eye[a];
    }
    const stage = (i, center, size, layer) => boxes.set([
      ...center.map((v, a) => v - eye[a]), 0, ...size, 0, layer, layer, layer, 30, 0, 0, 1, 0,
    ], i * stride);
    stage(count, [feet[0], start[1] - 0.08, feet[2]], [7, 0.12, 7], 164);
    stage(count + 1, [feet[0], start[1] - 0.013, feet[2]], [0.72, 0.007, 0.55], 162);
    renderer.render({ eye, yaw, pitch, time: 0.5, boxes, boxCount: count + 2, target: null,
      mineProgress: 0, lineCells: new Int32Array(), quarryBoxes: new Int32Array(), cracks: new Int32Array(), underwater: false });
    status.textContent = `${document.querySelector('[data-pose][aria-pressed="true"]').textContent} · Live character animation · Your worlds are untouched`;
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);
} catch (error) {
  status.textContent = String(error);
  console.error(error);
}
