// Entry point: loads the wasm engine, opens the latest world or joins a co-op game (net/session.ts),
// builds the renderer, HUD, input, sound and panels, wires the pause menu, and runs the frame loop
// (input → `game.update` → co-op pump → sounds → streaming work → mesh events → render → name tags →
// HUD). Keeps no game state of its own; everything lives in the engine.

import './base.css';
import './ui/menu.css';
import init from './wasm/engine.js';
import { SoundSystem } from './audio/sound';
import { ComfortStore } from './comfort/settings';
import { ViewControls } from './controls/view';
import type { Coop } from './net/coop';
import { hostWorld, startCoop } from './net/session';
import { tickWhenStalled } from './net/ticker';
import { Input } from './input';
import { Renderer } from './render/renderer';
import { clock } from './render/sky';
import { FIRST_SEED, message, openWorld, type Opened, Session } from './save/session';
import { WorldStore } from './save/store';
import { ComfortPanel } from './ui/comfort';
import { CoopPanel } from './ui/coop';
import { CraftQueueView } from './ui/craftqueue';
import { BlueprintPanel } from './ui/blueprints';
import { SitePanel } from './ui/site';
import { Hints } from './ui/hints';
import { ProspectPanel } from './ui/prospect';
import { Hud } from './ui/hud';
import { InventoryPanel } from './ui/inventory';
import { MachinePanel } from './ui/machine';
import { Minimap } from './ui/minimap';
import { NameTags } from './ui/nametags';
import { Pins } from './ui/pins';
import { PlayerList } from './ui/players';
import { ResearchPanel } from './ui/research';
import { SoundLab } from './ui/sound-lab';
import { Vignette } from './ui/vignette';
import { VolumeControl } from './ui/volume-control';
import { WorldMap } from './ui/worldmap';
import { WorldsPanel } from './ui/worlds';

const MOUSE_SENSITIVITY = 0.0022; // radians per pixel at 100% (the comfort setting scales it)
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
  // A development review world uses the real controls without reading or overwriting saved worlds.
  const preview = import.meta.env.DEV && params.has('preview');
  const store = preview ? null : await WorldStore.open().catch(() => null);
  let opened: Opened;
  let coop: Coop | null;
  try {
    ({ opened, coop } = await startCoop(params, viewRadius, () => openWorld(store, seed, viewRadius)));
  } catch (err) {
    // The latest world can't be loaded, or joining failed: offer the others instead of starting.
    document.getElementById('loading-text')!.textContent = message(err);
    if (store) worlds.append(new WorldsPanel(store, null).el);
    return;
  }
  const { game, meta } = opened;
  const session = store && !game.is_client() ? new Session(store, meta, game) : null;
  if (store) worlds.append(new WorldsPanel(store, session, opened.notice).el);
  else worlds.textContent = preview ? 'Controls preview · this world is not saved' : opened.notice;

  const canvas = document.getElementById('game') as HTMLCanvasElement;
  const instanceFloats = game.instance_floats();
  const renderer = new Renderer(canvas, viewRadius, instanceFloats);
  const texPixels = new Uint8Array(wasm.memory.buffer, game.texture_ptr(), game.texture_byte_len()).slice();
  renderer.setTextures(texPixels, game.texture_size(), game.texture_layers());
  const hud = new Hud(game, texPixels, game.texture_size());
  const comfort = new ComfortStore();
  const vignette = new Vignette(comfort);
  document.getElementById('comfort')!.append(new ComfortPanel(comfort, vignette, document.getElementById('menu')!).el);
  const input = new Input(canvas);
  const views = new ViewControls(game, input, renderer, canvas, comfort);
  document.getElementById('comfort')!.prepend(views.menu);
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
  const blueprints = new BlueprintPanel(game);
  const sites = new SitePanel(game);
  const craftQueue = new CraftQueueView(game, (id) => hud.itemIcon(id));
  const prospect = new ProspectPanel(game);
  prospect.onReading = () => sound.scan();
  const pins = new Pins(game, meta.pins);
  session?.keepPins(pins);
  const minimap = new Minimap(game, wasm.memory, pins);
  const worldMap = new WorldMap(game, wasm.memory, pins);
  const nameTags = new NameTags();
  const playerList = new PlayerList();
  const coopPanel = new CoopPanel(coop, {
    host: async () => beginCoop(await hostWorld(game, '')),
    leave: () => session?.finish() ?? Promise.resolve(),
  });
  document.getElementById('coop')!.append(coopPanel.el);
  const panelOpen = () => inventory.isOpen || machine.isOpen || research.isOpen || worldMap.isOpen || blueprints.isOpen || sites.isOpen;

  // ---- menu / pointer lock
  const menu = document.getElementById('menu')!;
  const play = document.getElementById('play') as HTMLButtonElement;
  const loadingFill = document.getElementById('loading-fill')!;
  const loadingText = document.getElementById('loading-text')!;
  // Worlds made before Milestone 4 keep their first terrain (generator version 1).
  const terrain = game.worldgen_version() === 1 ? ' · classic terrain' : '';
  const worldInfo = document.getElementById('world-info')!;
  const showWorldInfo = () => {
    const time = `day ${game.day_number()}, ${clock(game.time_of_day())}`;
    worldInfo.textContent = `${meta.name} · ${time} · seed ${meta.seed}${terrain} · render distance ${viewRadius} chunks`;
  };
  showWorldInfo();
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
  const resumeHint = document.getElementById('resume-hint')!;
  const pause = () => {
    showWorldInfo();
    resumeHint.classList.add('hidden');
    menu.classList.remove('hidden');
  };
  // E or a click outside goes straight back to play (both are gestures that allow re-locking). Escape
  // isn't one, so it leaves the game on screen with a hint: a click plays on, a second Escape pauses.
  const closed = (resume: boolean) => {
    if (resume) start();
    else resumeHint.classList.remove('hidden');
  };
  window.addEventListener('keydown', (e) => {
    // A panel's own Escape handler already ran (and prevented the default) when it closed.
    if (e.key !== 'Escape' || e.repeat || e.defaultPrevented || resumeHint.classList.contains('hidden')) return;
    pause();
    session?.save().catch(() => {}); // pausing saves
  });
  inventory.onClose = (resume) => {
    game.close_inventory();
    closed(resume);
  };
  machine.onClose = closed;
  research.onClose = closed;
  blueprints.onClose = closed;
  sites.onClose = closed;
  worldMap.onClose = closed;
  // Browsers never let a page block Ctrl+W, so closing the tab mid-game (or just after the pointer was
  // freed by it) asks first. With the menu showing, its own links and reloads leave without asking.
  let leftGame = -Infinity;
  window.addEventListener('beforeunload', (e) => {
    if (menu.classList.contains('hidden') || performance.now() - leftGame < 1000) e.preventDefault();
  });
  input.onLockChange = (locked) => {
    menu.classList.toggle('hidden', locked || panelOpen());
    resumeHint.classList.add('hidden');
    if (!locked) {
      leftGame = performance.now();
      showWorldInfo();
      play.textContent = 'Resume';
      game.set_move(0, 0, false, false, false);
      game.pan_camera(0, 0, false);
      game.cancel_move_order();
      game.set_mining(false);
      game.cancel_line();
      game.set_using(false);
      if (!panelOpen()) session?.save().catch(() => {}); // pausing saves
    }
  };
  canvas.addEventListener('webglcontextlost', (e) => {
    e.preventDefault();
    session?.saveNow();
    loadingText.textContent = 'Graphics context lost. Reload the page to continue.';
    pause();
  });

  // Handy for poking at the engine from the devtools console.
  const handles = {
    game, renderer, wasm, sound, soundLab, inventory, machine, research, blueprints, sites, hints, prospect, minimap, worldMap, pins, session, coop, comfort, vignette, views,
  };
  Object.assign(window, { opencraft: handles });

  // ---- frame loop
  let last = performance.now();
  let lastFrame = last;
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
        renderer.upsertChunk(x, y, z, verts, game.mesh_opaque_quads(), game.mesh_cutout_quads(), game.mesh_liquid_quads());
      } else {
        renderer.removeChunk(x, y, z);
      }
    }
  };

  // Everything but drawing: input, the engine, co-op, panels, sounds, streaming work. A co-op tab whose
  // frames stopped (hidden, minimised) runs just this (net/ticker.ts), so a hidden host doesn't stop
  // everyone.
  const advance = (now: number) => {
    const dt = Math.min((now - last) / 1000, 0.1);
    last = now;

    if (input.locked) {
      views.advance((MOUSE_SENSITIVITY * comfort.settings.sensitivity) / 100);
    }
    for (const a of input.takeActions()) {
      if (views.action(a)) continue;
      if (a.kind === 'debug') hud.toggleDebug();
      else if (a.kind === 'hint') hints.skip();
      else if (a.kind === 'map') minimap.toggle();
      else if (a.kind === 'slot') game.select_slot(a.slot);
      else if (a.kind === 'fly') game.toggle_fly();
      else if (a.kind === 'drop') game.drop_selected();
      else if (a.kind === 'rotate') game.rotate_target();
      else if (a.kind === 'ghost') game.toggle_ghost_mode();
      else if (a.kind === 'fetch') game.fetch_held();
      else if (a.kind === 'blueprint-mark') game.blueprint_mark();
      else if (a.kind === 'blueprint-copy') game.blueprint_copy_selection();
      else if (a.kind === 'mute') sound.toggleMute();
      else if (a.kind === 'sound-lab') {
        // Open on the block being looked at, else on whatever was heard last (e.g. the ground).
        input.unlock();
        soundLab.open(game.has_target() ? game.block_sound(game.target_block()) : sound.lastMaterial);
      } else if (a.kind === 'inventory') {
        inventory.open();
        input.unlock();
      } else if (a.kind === 'blueprints') {
        blueprints.open();
        input.unlock();
      } else if (a.kind === 'research') {
        research.open();
        input.unlock();
      } else if (a.kind === 'world-map') {
        worldMap.open();
        input.unlock();
      }
    }
    game.update(dt);
    coop?.pump();
    // A right-clicked machine with a panel: free the mouse and open it (a box opens like a chest).
    const request = game.take_panel_request();
    if (request.length === 3) {
      const [x, y, z] = request;
      if (game.box_slots(x, y, z).length > 0) inventory.open([x, y, z]);
      else machine.open(x, y, z);
      input.unlock();
      sound.ui();
    }
    // A planner request: free the mouse and show the site panel.
    const site = game.take_site_request();
    if (site.length > 0) {
      sites.open(Array.from(site));
      input.unlock();
      sound.ui();
    }
    // Hotbar changes are engine actions that land on the next tick, so compare across frames.
    if (game.selected_slot() !== lastSlot) {
      lastSlot = game.selected_slot();
      sound.ui();
    }
    if (!document.hidden) {
      const events = new Float32Array(wasm.memory.buffer, game.sound_ptr(), game.sound_count() * 6);
      sound.playEvents(events, game.sound_count(), game.camera_yaw());
    }
    game.clear_sounds();

    const budget = game.ready() ? WORK_BUDGET_MS : LOADING_WORK_BUDGET_MS;
    const w0 = performance.now();
    game.begin_work();
    while (game.work_step() && performance.now() - w0 < budget);
    workMs = workMs * 0.9 + (performance.now() - w0) * 0.1;
    drainEvents();
    return dt;
  };

  const frame = (now: number) => {
    const t0 = performance.now();
    lastFrame = t0;
    const dt = advance(now);
    const ready = game.ready();

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
    const cursorBlock = game.view_mode() >= 2 ? game.strategy_hover() : [];
    renderer.render({
      eye: [game.eye_x(), game.eye_y(), game.eye_z()],
      yaw: game.camera_yaw(),
      pitch: game.camera_pitch(),
      target: cursorBlock.length === 3 ? [cursorBlock[0], cursorBlock[1], cursorBlock[2]]
        : hasTarget ? [game.target_x(), game.target_y(), game.target_z()] : null,
      mineProgress: game.mine_progress(),
      time: game.time_of_day(),
      boxes: new Float32Array(wasm.memory.buffer, game.instance_ptr(), game.instance_count() * instanceFloats),
      boxCount: game.instance_count(),
      lineCells: game.line_cells(),
      quarryBoxes: machine.quarryBox() ?? game.placement_box(),
      cracks: game.quarry_cracks(),
      underwater: game.eye_in_water(),
    });
    vignette.update(dt, game.camera_yaw(), game.camera_pitch(), game.eye_x(), game.eye_y(), game.eye_z());
    views.sync();
    const labels = new Float32Array(wasm.memory.buffer, game.label_ptr(), game.label_count() * 4);
    nameTags.update(labels, game.label_count(), renderer, (id) => coop?.name(id));

    frameMs = frameMs * 0.9 + (performance.now() - t0) * 0.1;
    fps = fps * 0.9 + (dt > 0 ? 1 / dt : 0) * 0.1;
    hud.update(now, { fps, frameMs, workMs, ...renderer.stats });
    inventory.update();
    machine.update();
    research.update(now);
    hints.update();
    craftQueue.update();
    prospect.update();
    minimap.update(now);
    worldMap.update(now);
    playerList.update(coop, input.held('Tab'), now);
    coopPanel.update(now);
    requestAnimationFrame(frame);
  };

  // A co-op session, from the URL or the menu: notices, its end (back to the menu, the world stopped),
  // and ticking on while this tab gets no frames (net/ticker.ts), so a hidden host doesn't stop everyone.
  function beginCoop(c: Coop): Coop {
    coop = handles.coop = c;
    c.onNotice = (text) => playerList.notice(text);
    c.onEnd = (reason) => {
      play.disabled = true;
      input.unlock();
      pause();
      coopPanel.end(reason);
    };
    tickWhenStalled(() => lastFrame, () => advance(performance.now()));
    return c;
  }

  requestAnimationFrame(frame);
  if (coop) beginCoop(coop);
}

main().catch((err: unknown) => {
  console.error(err);
  const text = document.getElementById('loading-text');
  if (text) text.textContent = `Failed to start: ${err instanceof Error ? err.message : String(err)}`;
});
