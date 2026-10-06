// The world being played: `openWorld` picks and loads it at startup, `Session` autosaves it, and
// `switchTo` changes worlds. Autosaves run every minute and when the menu opens (gzipped), and when the
// tab is hidden or closed (raw, since there may be no time to compress). A co-op client never saves: the
// world is the host's. Each save also stores the player's notes: the prospected deposits and map pins
// in the world's record, and the explored map (gzipped, only when it changed, not while the page
// closes) in its own slot; a session restores them. Switching worlds saves, marks the other world as the latest and reloads the page
// (without co-op parameters: `soloUrl`), which is simpler and leaner than swapping engines in place
// (wasm memory never shrinks).

import { Game } from '../wasm/engine.js';
import type { Pins } from '../ui/pins';
import { newWorld, pack, type WorldMeta, type WorldStore } from './store';

const AUTOSAVE_MS = 60_000;
/** Seed of the first world a new player gets. */
export const FIRST_SEED = 1337;

export interface Opened {
  game: Game;
  meta: WorldMeta;
  /** True when the world came from a save (the menu offers "Continue"). */
  restored: boolean;
  /** Something the player should know, such as the backup having been used. */
  notice: string;
}

/** Opens the most recently played world, or a new one for `?seed=` or when there are none. Throws a
 * message a player can read when the world can't be loaded; the menu then offers the others. Without
 * storage (`store` null: the browser refused it) the world is new and can't be saved. */
export async function openWorld(store: WorldStore | null, seed: number | null, viewRadius: number): Promise<Opened> {
  if (!store) {
    const notice = "This browser doesn't let the game store data, so this world won't be saved.";
    const meta = newWorld('Unsaved world', seed ?? FIRST_SEED);
    return { game: freshGame(meta, viewRadius), meta, restored: false, notice };
  }
  let meta = seed === null ? (await store.list())[0] : undefined;
  if (!meta) {
    meta = newWorld(seed === null ? 'My world' : `Seed ${seed}`, seed ?? FIRST_SEED);
    await store.put(meta);
  }
  if (meta.slot === null) return { game: freshGame(meta, viewRadius), meta, restored: false, notice: '' };
  try {
    return { game: await load(store, meta, meta.slot, viewRadius), meta, restored: true, notice: '' };
  } catch (err) {
    // The previous save is kept for exactly this.
    const backup = 1 - meta.slot;
    const game = await load(store, meta, backup, viewRadius).catch(() => null);
    if (!game) throw new Error(`"${meta.name}" can't be opened. ${message(err)}`);
    const notice = `The latest save of "${meta.name}" couldn't be loaded (${message(err)}), so the one before it was.`;
    return { game, meta: { ...meta, slot: backup }, restored: true, notice };
  }
}

/** Keeps the open world saved. */
export class Session {
  /** Called with a message when a save fails. */
  onError: (message: string) => void = () => {};
  private seq = 0;
  private stopped = false;
  /** Game time of the newest save, to skip saving an unchanged world (closing a tab both hides it and
   * closes it). State only changes on ticks, so the same time means the same world. */
  private savedTime: number;
  private readonly timer: number;
  private pins: Pins | null = null;
  /** The pins changed since the last save (they may change while no game time passes). */
  private notesChanged = false;
  /** blueprint_version at the last save: the blueprints changed when it moves. */
  private blueprintsSeen = 0;
  /** `world_map_version` of the explored map last stored; null until the stored one is back in. */
  private mapVersion: number | null = null;

  constructor(
    private readonly store: WorldStore,
    public meta: WorldMeta,
    private readonly game: Game,
  ) {
    this.savedTime = game.play_seconds();
    game.set_known_deposits(new Int32Array(meta.marks ?? []));
    if (meta.blueprints) game.blueprint_import(meta.blueprints);
    this.blueprintsSeen = game.blueprint_version();
    store
      .readMap(meta.id)
      .then((map) => map && game.set_explored_map(map))
      .catch(() => {})
      .finally(() => (this.mapVersion = game.world_map_version()));
    this.timer = window.setInterval(() => this.save().catch(() => {}), AUTOSAVE_MS);
    document.addEventListener('visibilitychange', () => {
      if (document.hidden) this.saveNow();
    });
    window.addEventListener('pagehide', () => this.saveNow());
  }

  /** Keeps `pins` with the world's record from now on. */
  keepPins(pins: Pins): void {
    this.pins = pins;
    pins.subscribe(() => (this.notesChanged = true));
  }

  /** Saves, gzipped; resolves once stored, rejects if storing failed (after `onError`). */
  async save(): Promise<void> {
    const time = this.game.play_seconds();
    if (this.skip(time)) return;
    const seq = ++this.seq;
    const bytes = await pack(this.game.save());
    const version = this.game.world_map_version();
    const map = this.mapVersion !== null && version !== this.mapVersion ? await pack(this.game.explored_map()) : undefined;
    // A newer save (e.g. the page closing) overtook this one while it compressed.
    if (seq !== this.seq || this.stopped) return;
    await this.write(bytes, time, map);
    if (map) this.mapVersion = version;
  }

  /** Saves raw bytes at once, for when the page may be about to close. */
  saveNow(): void {
    const time = this.game.play_seconds();
    if (this.skip(time)) return;
    this.seq++;
    this.write(this.game.save(), time).catch(() => {});
  }

  /** Saves one last time and stops, before the page switches to another world. */
  async finish(): Promise<void> {
    await this.save();
    this.stopped = true;
    window.clearInterval(this.timer);
  }

  /** Nothing to save: stopped, unchanged, or a co-op client's copy of the host's world. */
  private skip(time: number): boolean {
    return this.stopped || (time === this.savedTime && !this.notesChanged && this.game.blueprint_version() === this.blueprintsSeen) || this.game.is_client();
  }

  /** Writes into the slot that isn't newest, keeping the newest as the backup. */
  private async write(bytes: Uint8Array, time: number, map?: Uint8Array): Promise<void> {
    const slot = this.meta.slot === 0 ? 1 : 0;
    const marks = Array.from(this.game.known_deposits());
    const pins = this.pins ? this.pins.list.map((p) => ({ ...p })) : this.meta.pins;
    this.blueprintsSeen = this.game.blueprint_version();
    const blueprints = this.game.blueprint_export();
    const creative = this.game.is_creative();
    this.meta = { ...this.meta, updated: Date.now(), playTime: time, slot, marks, pins, blueprints, creative };
    this.savedTime = time;
    this.notesChanged = false;
    try {
      await this.store.write(this.meta, bytes, map);
    } catch (err) {
      this.savedTime = -1;
      this.onError(`Couldn't save the world: ${message(err)}`);
      throw err;
    }
  }
}

/** Opens another world (new or saved): saves the current one, marks that one as the latest and reloads
 * the page, which then opens it. */
export async function switchTo(store: WorldStore, session: Session | null, meta: WorldMeta): Promise<void> {
  await session?.finish();
  await store.put({ ...meta, updated: Date.now() });
  location.assign(soloUrl());
}

/** This page's address without the co-op parameters, so loading it plays the latest world alone. */
export function soloUrl(): string {
  const url = new URL(location.href);
  for (const name of ['host', 'join', 'relay']) url.searchParams.delete(name);
  return url.href;
}

/** The text of anything thrown: an Error, or the plain string the engine throws. */
export function message(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

/** A world that was never saved starts from its seed, in the mode it was made with. */
function freshGame(meta: WorldMeta, viewRadius: number): Game {
  return meta.creative ? Game.new_creative(meta.seed, viewRadius) : new Game(meta.seed, viewRadius);
}

async function load(store: WorldStore, meta: WorldMeta, slot: number, viewRadius: number): Promise<Game> {
  const bytes = await store.read(meta.id, slot);
  if (!bytes) throw new Error(`"${meta.name}" has no save.`);
  return Game.load(bytes, viewRadius);
}
