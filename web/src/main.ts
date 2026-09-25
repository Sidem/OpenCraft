// Entry point: loads the wasm engine, opens the latest world (save/), builds the renderer, HUD, input,
// sound and panels, wires the pause menu, and runs the frame loop (input → `game.update` → sounds →
// streaming work → mesh events → render → HUD). Keeps no game state of its own; everything lives in
// the engine.

import './base.css';
import './ui/menu.css';
import init from './wasm/engine.js';
import { SoundSystem } from './audio/sound';
import { Input } from './input';
import { INSTANCE_FLOATS } from './render/boxes';
import { Renderer } from './render/renderer';
import { FIRST_SEED, message, openWorld, type Opened, Session } from './save/session';
import { WorldStore } from './save/store';
import { Hints } from './ui/hints';
import { Hud } from './ui/hud';
import { InventoryPanel } from './ui/inventory';
import { MachinePanel } from './ui/machine';
import { ResearchPanel } from './ui/research';
import { SoundLab } from './ui/sound-lab';
import { VolumeControl } from './ui/volume-control';
import { WorldsPanel } from './ui/worlds';

const MOUSE_SENSITIVITY = 0.0022; // radians per pixel
const WORK_BUDGET_MS = 6; // per frame, for streaming world generation and meshing
const LOADING_WORK_BUDGET_MS = 28;

const intParam = (params: URLSearchParams, name: string, fallback: number, min: number, max: number) => {
  const v = Number.parseInt(params.get(name) ?? '', 10);
  return Number.isFinite(v) ? Math.min(max, Math.max(min, v)) : fallback;
};

async function main(): Promise<void> {
  const wasm = await init();
  const params = new URLSearchParams(location.search);
  const seed = params.has('seed') ? intParam(params, 'seed', FIRST_SEED, 0, 0xffffffff) : null;
  const viewRadius = intParam(params, 'rd', 8, 2, 24);
  // `?seed=` starts a new world once; dropping it makes a reload continue that world.
  if (seed !== null) {
    params.delete('seed');
    history.replaceState(null, '', params.size ? `?${params}` : location.pathname);
  }

  // ---- world: the latest one, kept saved (save/) and managed from the menu (ui/worlds.ts)
  const worlds = document.getElementById('worlds')!;
  const store = await WorldStore.open().catch(() => null);
  let opened: Opened;
  try {
    opened = await openWorld(store, seed, viewRadius);
  } catch (err) {
    // The latest world can't be loaded: offer the others instead of starting.
    document.getElementById('loading-text')!.textContent = message(err);
    if (store) worlds.append(new WorldsPanel(store, null).el);
    return;
  }
  const { game, meta } = opened;
  const session = store && new Session(store, meta, game);
  if (store) worlds.append(new WorldsPanel(store, session, opened.notice).el);
  else worlds.textContent = opened.notice;

  const canvas = document.getElementById('game') as HTMLCanvasElement;
  const renderer = new Renderer(canvas, viewRadius);
  const texPixels = new Uint8Array(wasm.memory.buffer, game.texture_ptr(), game.texture_byte_len()).slice();
  renderer.setTextures(texPixels, game.texture_size(), game.texture_layers());
  const hud = new Hud(game, texPixels, game.texture_size());
  const input = new Input(canvas);
  const sound = new SoundSystem();
  hud.setMuted(sound.muted);
  sound.subscribe(() => hud.setMuted(sound.muted));
  // Every block except air, with the sound material it uses.
  const blocks = Array.from({ length: game.block_count() - 1 }, (_, i) => ({
    id: i + 1,
    name: game.block_name(i + 1),
    material: game.block_sound(i + 1),
  }));
  const soundLab = new SoundLab(sound, blocks, (id) => hud.itemIcon(id));
  const inventory = new InventoryPanel(game, (id) => hud.itemIcon(id));
  inventory.onCraft = () => sound.ui();
  const machine = new MachinePanel(game, (id) => hud.itemIcon(id));
  const research = new ResearchPanel(game, (id) => hud.itemIcon(id));
  research.onDone = () => sound.ui();
  const hints = new Hints(game);
  const panelOpen = () => inventory.isOpen || machine.isOpen || research.isOpen;

  // ---- menu / pointer lock
  const menu = document.getElementById('menu')!;
  const play = document.getElementById('play') as HTMLButtonElement;
  const loadingFill = document.getElementById('loading-fill')!;
  const loadingText = document.getElementById('loading-text')!;
  document.getElementById('world-info')!.textContent =
    `${meta.name} · seed ${meta.seed} · render distance ${viewRadius} chunks`;
  if (opened.restored) play.textContent = 'Continue';
  document.getElementById('menu-volume')!.append(new VolumeControl(sound).el);
  document.getElementById('open-sound-lab')!.addEventListener('click', () => {
    sound.unlock();
    soundLab.open(sound.lastMaterial);
  });
  // Audio can only start inside a user gesture, so the same click that captures the mouse unlocks it.
  const start = () => {
    sound.unlock();
    void input.lock();
  };
  play.addEventListener('click', start);
  canvas.addEventListener('click', () => {
    if (!input.locked && !play.disabled && !soundLab.isOpen && !panelOpen()) start();
  });
  // E or a click outside goes straight back to play (both are gestures that allow re-locking);
  // Escape lands on the pause menu, like it does from the game.
  const closed = (resume: boolean) => {
    if (resume) start();
    else menu.classList.remove('hidden');
  };
  inventory.onClose = (resume) => {
    game.close_inventory();
    closed(resume);
  };
  machine.onClose = closed;
  research.onClose = closed;
  input.onLockChange = (locked) => {
    menu.classList.toggle('hidden', locked || panelOpen());
    if (!locked) {
      play.textContent = 'Resume';
      game.set_move(0, 0, false, false, false);
      game.set_mining(false);
      game.set_using(false);
      if (!panelOpen()) session?.save().catch(() => {}); // pausing saves
    }
  };
  canvas.addEventListener('webglcontextlost', (e) => {
    e.preventDefault();
    session?.saveNow();
    loadingText.textContent = 'Graphics context lost. Reload the page to continue.';
    menu.classList.remove('hidden');
  });

  // Handy for poking at the engine from the devtools console.
  Object.assign(window, { opencraft: { game, renderer, wasm, sound, soundLab, inventory, machine, research, hints, session } });

  // ---- frame loop
  let last = performance.now();
  let lastSlot = game.selected_slot();
  let fps = 0, frameMs = 0, workMs = 0;
  let wasReady = false;

  const drainEvents = () => {
    for (;;) {
      const kind = game.next_event();
      if (kind === 0) break;
      const x = game.event_x(), y = game.event_y(), z = game.event_z();
      if (kind === 1) {
        // Zero-copy: a view straight into wasm memory, uploaded to the GPU and dropped.
        const verts = new Uint32Array(wasm.memory.buffer, game.mesh_ptr(), game.mesh_len());
        renderer.upsertChunk(x, y, z, verts, game.mesh_opaque_quads(), game.mesh_cutout_quads());
      } else {
        renderer.removeChunk(x, y, z);
      }
    }
  };

  const frame = (now: number) => {
    const dt = Math.min((now - last) / 1000, 0.1);
    last = now;
    const t0 = performance.now();

    if (input.locked) {
      const m = input.movement();
      game.set_move(m.forward, m.strafe, m.jump, m.sprint, m.crouch);
      const [dx, dy] = input.takeLook();
      if (dx !== 0 || dy !== 0) game.look(dx * MOUSE_SENSITIVITY, dy * MOUSE_SENSITIVITY);
      game.set_mining(input.mining);
      game.set_using(input.using);
    }
    for (const a of input.takeActions()) {
      if (a.kind === 'debug') hud.toggleDebug();
      else if (a.kind === 'hint') hints.skip();
      else if (a.kind === 'slot') game.select_slot(a.slot);
      else if (a.kind === 'scroll') game.scroll_slot(a.delta);
      else if (a.kind === 'fly') game.toggle_fly();
      else if (a.kind === 'drop') game.drop_selected();
      else if (a.kind === 'mute') sound.toggleMute();
      else if (a.kind === 'sound-lab') {
        // Open on the block being looked at, else on whatever was heard last (e.g. the ground).
        document.exitPointerLock();
        soundLab.open(game.has_target() ? game.block_sound(game.target_block()) : sound.lastMaterial);
      } else if (a.kind === 'inventory') {
        document.exitPointerLock();
        inventory.open();
      } else if (a.kind === 'research') {
        document.exitPointerLock();
        research.open();
      }
    }
    game.update(dt);
    // A right-clicked machine with a panel: free the mouse and open it (a box opens like a chest).
    const request = game.take_panel_request();
    if (request.length === 3) {
      const [x, y, z] = request;
      document.exitPointerLock();
      if (game.box_slots(x, y, z).length > 0) inventory.open([x, y, z]);
      else machine.open(x, y, z);
      sound.ui();
    }
    // Hotbar changes are engine actions that land on the next tick, so compare across frames.
    if (game.selected_slot() !== lastSlot) {
      lastSlot = game.selected_slot();
      sound.ui();
    }
    sound.playEvents(new Float32Array(wasm.memory.buffer, game.sound_ptr(), game.sound_count() * 6), game.sound_count(), game.yaw());
    game.clear_sounds();

    const ready = game.ready();
    const budget = ready ? WORK_BUDGET_MS : LOADING_WORK_BUDGET_MS;
    const w0 = performance.now();
    game.begin_work();
    while (game.work_step() && performance.now() - w0 < budget);
    workMs = workMs * 0.9 + (performance.now() - w0) * 0.1;
    drainEvents();

    if (!wasReady) {
      const loaded = game.chunks_loaded(), pending = game.chunks_pending();
      const pct = ready ? 100 : Math.round((loaded / Math.max(1, loaded + pending)) * 100);
      loadingFill.style.transform = `scaleX(${pct / 100})`;
      loadingText.textContent = ready ? 'World ready' : `Generating terrain... ${pct}%`;
      if (ready) {
        wasReady = true;
        play.disabled = false;
        menu.classList.add('ready');
      }
    }

    const hasTarget = game.has_target();
    renderer.render({
      eye: [game.eye_x(), game.eye_y(), game.eye_z()],
      yaw: game.yaw(),
      pitch: game.pitch(),
      target: hasTarget ? [game.target_x(), game.target_y(), game.target_z()] : null,
      mineProgress: game.mine_progress(),
      boxes: new Float32Array(wasm.memory.buffer, game.instance_ptr(), game.instance_count() * INSTANCE_FLOATS),
      boxCount: game.instance_count(),
    });

    frameMs = frameMs * 0.9 + (performance.now() - t0) * 0.1;
    fps = fps * 0.9 + (dt > 0 ? 1 / dt : 0) * 0.1;
    hud.update(now, { fps, frameMs, workMs, ...renderer.stats });
    inventory.update();
    machine.update();
    research.update(now);
    hints.update();
    requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);
}

main().catch((err: unknown) => {
  console.error(err);
  const text = document.getElementById('loading-text');
  if (text) text.textContent = `Failed to start: ${err instanceof Error ? err.message : String(err)}`;
});
