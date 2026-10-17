// Entry point: loads the wasm engine, opens the latest world or joins a co-op game (net/session.ts),
// builds the renderer, HUD, input, sound and panels, wires the pause menu, and runs the frame loop
// (input → `game.update` → co-op pump → sounds → streaming work → mesh events → render → name tags →
// HUD). Keeps no game state of its own; everything lives in the engine.

import './base.css';
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
import { AnalyticsPanel } from './ui/analytics';
import { ComfortPanel } from './ui/comfort';
import { SkyStore } from './render/sky/settings';
import { SkyPanel } from './ui/sky';
import { isTouchDevice, showDesktopOnly } from './ui/desktop-only';
import { CoopPanel } from './ui/coop';
import { CraftQueueView } from './ui/craftqueue';
import { BlueprintPanel } from './ui/blueprints';
import { SitePanel } from './ui/site';
import { LogbookPanel } from './ui/logbook';
import { VaultPanel } from './ui/vault';
import { ConsolePanel } from './ui/console';
import { consoleHost } from './console/host';
import { Hints } from './ui/hints';
import { ProspectPanel } from './ui/prospect';
import { Hud } from './ui/hud';
import { InventoryPanel } from './ui/inventory';
import { MachinePanel } from './ui/machine';
import { Menu } from './ui/menu';
import { Minimap } from './ui/minimap';
import { NameTags } from './ui/nametags';
import { Pins } from './ui/pins';
import { PlayerList } from './ui/players';
import { ResearchPanel } from './ui/research';
import { SoundLab } from './ui/sound-lab';
import { Vignette } from './ui/vignette';
import { SoundPanel } from './ui/sound-panel';
import { WorldMap } from './ui/worldmap';
import { WorldsPanel } from './ui/worlds';

const MOUSE_SENSITIVITY = 0.0022; // radians per pixel at 100% (the comfort setting scales it)
// Streaming work (generating and meshing chunks) gets what a 60 fps frame has left after `game.update`: at most
// WORK_BUDGET_MS, at least WORK_MIN_MS. A step is not started when the slowest recent one would overrun it.
// Only the first load, before the world is ready once, may take LOADING_WORK_BUDGET_MS a frame.
const WORK_BUDGET_MS = 6;
const WORK_MIN_MS = 2.5;
const FRAME_ENGINE_MS = 13; // update + streaming work together
const LOADING_WORK_BUDGET_MS = 28;

const intParam = (params: URLSearchParams, name: string, fallback: number, min: number, max: number) => {
  const v = Number.parseInt(params.get(name) ?? '', 10);
  return Number.isFinite(v) ? Math.min(max, Math.max(min, v)) : fallback;
};

async function main(): Promise<void> {
  const menu = new Menu();
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
    menu.say(message(err));
    if (store) worlds.append(new WorldsPanel(store, null).el);
    menu.show('worlds');
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
  vignette.previewing = () => menu.previewing;
  document.getElementById('comfort')!.append(new ComfortPanel(comfort).el);
  const skyStore = new SkyStore();
  skyStore.subscribe(() => (renderer.skySettings = skyStore.settings));
  document.getElementById('sky-settings')!.append(new SkyPanel(skyStore).el);
  const input = new Input(canvas);
  const views =new ViewControls(game, input, renderer, canvas, comfort);
  document.getElementById('menu-camera')!.append(views.menu);
  const sound = new SoundSystem();
  hud.setMuted(sound.silent);
  sound.subscribe(() => hud.setMuted(sound.silent));
  // Every block except air, with the sound material it uses.
  const blocks = Array.from({ length: game.block_count() - 1 }, (_, i) => ({
    id: i + 1,
    name: game.block_name(i + 1),
    material: game.block_sound(i + 1),
  }));
  const soundLab = new SoundLab(sound, blocks, (id) => hud.itemIcon(id));
  const inventory = new InventoryPanel(game, (id) => hud.itemIcon(id));
  inventory.onCraft = () => sound.ui();
  const machine = new MachinePanel(game, (id) => hud.itemIcon(id), inventory);
  const research = new ResearchPanel(game, (id) => hud.itemIcon(id));
  research.onDone = () => sound.ui();
  const analytics = new AnalyticsPanel(game, (id) => hud.itemIcon(id));
  const hints = new Hints(game, renderer, () => sound.preview('powerup', 0));
  const blueprints = new BlueprintPanel(game);
  const sites = new SitePanel(game);
  const craftQueue = new CraftQueueView(game, (id) => hud.itemIcon(id));
  const prospect = new ProspectPanel(game);
  prospect.onReading = () => sound.scan();
  const pins = new Pins(game, meta.pins);
  const logbook = new LogbookPanel(game, pins);
  const vault = new VaultPanel(game);
  const adminConsole = new ConsolePanel(consoleHost(game, renderer, () => coop !== null));
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
  // Screens the keys open (the action's kind names them); each frees the mouse while open.
  const screens = { blueprints, research, analytics, 'world-map': worldMap };
  const panels = [inventory, research, worldMap, blueprints, sites, analytics, logbook, vault, adminConsole]; // a machine's panel is in the inventory
  const panelOpen = () => panels.some((p) => p.isOpen);

  // ---- menu / pointer lock
  const play = menu.play;
  // Worlds made before Milestone 4 keep their first terrain (generator version 1).
  const terrain = game.worldgen_version() === 1 ? ' · classic terrain' : '';
  const range = viewRadius === 8 ? '' : ` · render distance ${viewRadius}`;
  const showWorldInfo = () => {
    const time = `Day ${game.day_number()}, ${clock(game.time_of_day())}`;
    menu.world(meta.name, `${time} · seed ${meta.seed}${terrain}${range}`);
  };
  showWorldInfo();
  menu.opened(opened.restored);
  if (opened.restored) play.textContent = 'Continue';
  document.getElementById('sound-panel')!.append(new SoundPanel(sound).el);
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
    menu.pause();
  };
  // E or a click outside goes straight back to play (both are gestures that allow re-locking). Escape
  // isn't one, so it leaves the game on screen with a hint: a click plays on, a second Escape pauses.
  const closed = (resume: boolean) => {
    if (resume) start();
    else resumeHint.classList.remove('hidden');
  };
  // Escape: a screen's own handler closes it first (and prevents the default). On the game with the hint it pauses; on
  // the pause screen it plays on. Escape isn't a gesture that lets a page grab the mouse again, so when the browser
  // refuses, the game stays on screen with the hint (a click plays on) rather than leaving the menu stuck.
  window.addEventListener('keydown', (e) => {
    if (e.key !== 'Escape' || e.repeat || e.defaultPrevented) return;
    if (!menu.isPaused) {
      if (resumeHint.classList.contains('hidden')) return;
      pause();
      session?.save().catch(() => {}); // pausing saves
      return;
    }
    if ((e.target as HTMLElement | null)?.closest?.('input, textarea, select') || soundLab.isOpen || panelOpen()) return;
    e.preventDefault();
    menu.el.classList.add('hidden');
    sound.unlock();
    void input.lock().then(() => {
      if (!input.locked && !document.pointerLockElement) resumeHint.classList.remove('hidden');
    });
  });
  inventory.onClose = (resume) => {
    game.close_inventory();
    closed(resume);
  };
  for (const p of [research, blueprints, sites, worldMap, analytics, logbook, vault, adminConsole]) p.onClose = closed;
  // Browsers never let a page block Ctrl+W, so closing the tab mid-game (or just after the pointer was
  // freed by it) asks first. With the menu showing, its own links and reloads leave without asking.
  let leftGame = -Infinity;
  window.addEventListener('beforeunload', (e) => {
    if (!menu.isOpen || performance.now() - leftGame < 1000) e.preventDefault();
  });
  input.onLockChange = (locked) => {
    resumeHint.classList.add('hidden');
    if (locked || panelOpen()) menu.el.classList.add('hidden');
    else pause();
    if (!locked) {
      leftGame = performance.now();
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
    pause();
    menu.say('Graphics context lost. Reload the page to continue.');
  });

  // Handy for poking at the engine from the devtools console.
  const handles = {
    game, menu, renderer, wasm, sound, soundLab, inventory, machine, research, analytics, blueprints, sites, logbook, vault, adminConsole, hints, prospect, minimap, worldMap, pins, session, coop, comfort, vignette, views,
  };
  Object.assign(window, { opencraft: handles });

  // ---- frame loop
  let last = performance.now();
  let lastFrame = last;
  let lastSlot = game.selected_slot();
  let fps = 0, frameMs = 0, workMs = 0, stepMs = 0;
  let loaded = false; // the world has been ready once: from then on streaming never takes a loading-screen budget
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
      else if (a.kind === 'scan-target') prospect.nextTarget();
      else if (a.kind === 'blueprint-mark') game.blueprint_mark();
      else if (a.kind === 'blueprint-copy') game.blueprint_copy_selection();
      else if (a.kind === 'mute') sound.toggleMute();
      else if (a.kind === 'sound-lab') {
        // Open on the block being looked at, else on whatever was heard last (e.g. the ground).
        input.unlock();
        soundLab.open(game.has_target() ? game.block_sound(game.target_block()) : sound.lastMaterial);
      } else if (a.kind === 'console') {
        adminConsole.open();
        input.unlock();
      } else if (a.kind === 'inventory') {
        inventory.open();
        input.unlock();
      } else if (a.kind in screens) {
        screens[a.kind as keyof typeof screens].open();
        input.unlock();
      }
    }
    const u0 = performance.now();
    game.update(dt);
    const updateMs = performance.now() - u0;
    coop?.pump();
    // A right-clicked machine with a panel: free the mouse and open it (a box opens like a chest).
    const request = game.take_panel_request();
    if (request.length === 3) {
      const [x, y, z] = request;
      if (game.box_slots(x, y, z).length > 0) inventory.open([x, y, z]);
      else if (!logbook.open(x, y, z) && !vault.open(x, y, z)) machine.open(x, y, z);
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
    sound.ambience.weather = renderer.weather.ease.value; // rain is heard as it is drawn
    if (!document.hidden) sound.poll(game, dt);

    loaded ||= game.ready();
    const budget = loaded ? Math.min(WORK_BUDGET_MS, Math.max(WORK_MIN_MS, FRAME_ENGINE_MS - updateMs)) : LOADING_WORK_BUDGET_MS;
    const w0 = performance.now();
    game.begin_work();
    for (let more = true, from = w0; more; ) {
      more = game.work_step();
      const now = performance.now();
      stepMs = Math.max(stepMs * 0.995, now - from); // the slowest recent step, slowly forgotten
      from = now;
      if (now - w0 + (loaded ? stepMs * 0.5 : 0) >= budget) break;
    }
    workMs = workMs * 0.9 + (performance.now() - w0) * 0.1;
    drainEvents();
    return dt;
  };

  // The weather is a function of the seed and the biome underfoot (render/sky/weather.ts); the biome is read twice a second.
  renderer.precip.roofSource = () => game.rain_roof(); // where rain stops (roofs, caves)
  const worldSeed = game.seed();
  let biome = 0, biomeRead = -1e9;
  const biomeNow = (now: number) => {
    if (now - biomeRead > 500) {
      biomeRead = now;
      biome = game.surroundings()[0];
    }
    return biome;
  };

  const frame = (now: number) => {
    const t0 = performance.now();
    lastFrame = t0;
    const dt = advance(now);
    const ready = game.ready();

    if (!wasReady) {
      const loaded = game.chunks_loaded(), pending = game.chunks_pending();
      menu.loading(ready ? 100 : Math.min(99, Math.round((loaded / Math.max(1, loaded + pending)) * 100)));
      wasReady = ready;
    }

    const hasTarget = game.has_target();
    const cursorBlock = game.view_mode() >= 2 ? game.strategy_hover() : [];
    renderer.render({
      eye: [game.eye_x(), game.eye_y(), game.eye_z()],
      yaw: game.camera_yaw(),
      pitch: game.camera_pitch(),
      target: cursorBlock.length === 3 ? [cursorBlock[0], cursorBlock[1], cursorBlock[2]]
        : hasTarget ? [game.target_x(), game.target_y(), game.target_z()] : null,
      targetFit: cursorBlock.length === 3 || !hasTarget ? [] : Array.from(game.target_fit()),
      mineProgress: game.mine_progress(),
      time: game.time_of_day(),
      days: game.day_number() - 1 + game.time_of_day(),
      seed: worldSeed,
      biome: biomeNow(now),
      boxes: new Float32Array(wasm.memory.buffer, game.instance_ptr(), game.instance_count() * instanceFloats),
      boxCount: game.instance_count(),
      lineCells: game.line_cells(),
      quarryBoxes: machine.quarryBox() ?? game.placement_box(),
      cracks: game.quarry_cracks(),
      underwater: game.eye_in_water(),
      satellites: game.sky_satellites(),
    });
    vignette.update(dt, game.camera_yaw(), game.camera_pitch(), game.eye_x(), game.eye_y(), game.eye_z());
    views.sync();
    const labels = new Float32Array(wasm.memory.buffer, game.label_ptr(), game.label_count() * 4);
    nameTags.update(labels, game.label_count(), renderer, (id) => coop?.name(id));

    frameMs = frameMs * 0.9 + (performance.now() - t0) * 0.1;
    fps = fps * 0.9 + (dt > 0 ? 1 / dt : 0) * 0.1;
    hud.update(now, { fps, frameMs, workMs, ...renderer.stats });
    inventory.update(); // and the machine panel shown in it
    research.update(now);
    analytics.update(now);
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
      menu.show('together');
      coopPanel.end(reason);
    };
    tickWhenStalled(() => lastFrame, () => advance(performance.now()));
    return c;
  }

  requestAnimationFrame(frame);
  if (coop) beginCoop(coop);
}

if (isTouchDevice()) {
  showDesktopOnly();
} else {
  main().catch((err: unknown) => {
    console.error(err);
    const text = document.getElementById('loading-text');
    if (text) text.textContent = `Failed to start: ${err instanceof Error ? err.message : String(err)}`;
  });
}
