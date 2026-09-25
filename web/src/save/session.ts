// The world being played: `openWorld` picks and loads it at startup, `Session` autosaves it, and
// `switchTo` changes worlds. Autosaves run every minute and when the menu opens (gzipped), and when the
// tab is hidden or closed (raw, since there may be no time to compress). Switching worlds saves, marks
// the other world as the latest and reloads the page, which is simpler and leaner than swapping engines
// in place (wasm memory never shrinks).

import { Game } from '../wasm/engine.js';
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
    return { game: new Game(meta.seed, viewRadius), meta, restored: false, notice };
  }
  let meta = seed === null ? (await store.list())[0] : undefined;
  if (!meta) {
    meta = newWorld(seed === null ? 'My world' : `Seed ${seed}`, seed ?? FIRST_SEED);
    await store.put(meta);
  }
  if (meta.slot === null) return { game: new Game(meta.seed, viewRadius), meta, restored: false, notice: '' };
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

  constructor(
    private readonly store: WorldStore,
    public meta: WorldMeta,
    private readonly game: Game,
  ) {
    this.savedTime = game.play_seconds();
    this.timer = window.setInterval(() => this.save().catch(() => {}), AUTOSAVE_MS);
    document.addEventListener('visibilitychange', () => {
      if (document.hidden) this.saveNow();
    });
    window.addEventListener('pagehide', () => this.saveNow());
  }

  /** Saves, gzipped; resolves once stored, rejects if storing failed (after `onError`). */
  async save(): Promise<void> {
    const time = this.game.play_seconds();
    if (this.stopped || time === this.savedTime) return;
    const seq = ++this.seq;
    const bytes = await pack(this.game.save());
    // A newer save (e.g. the page closing) overtook this one while it compressed.
    if (seq === this.seq && !this.stopped) await this.write(bytes, time);
  }

  /** Saves raw bytes at once, for when the page may be about to close. */
  saveNow(): void {
    const time = this.game.play_seconds();
    if (this.stopped || time === this.savedTime) return;
    this.seq++;
    this.write(this.game.save(), time).catch(() => {});
  }

  /** Saves one last time and stops, before the page switches to another world. */
  async finish(): Promise<void> {
    await this.save();
    this.stopped = true;
    window.clearInterval(this.timer);
  }

  /** Writes into the slot that isn't newest, keeping the newest as the backup. */
  private async write(bytes: Uint8Array, time: number): Promise<void> {
    const slot = this.meta.slot === 0 ? 1 : 0;
    this.meta = { ...this.meta, updated: Date.now(), playTime: time, slot };
    this.savedTime = time;
    try {
      await this.store.write(this.meta, bytes);
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
  location.reload();
}

/** The text of anything thrown: an Error, or the plain string the engine throws. */
export function message(err: unknown): string {
  return err instanceof Error ? err.message : String(err);
}

async function load(store: WorldStore, meta: WorldMeta, slot: number, viewRadius: number): Promise<Game> {
  const bytes = await store.read(meta.id, slot);
  if (!bytes) throw new Error(`"${meta.name}" has no save.`);
  return Game.load(bytes, viewRadius);
}
