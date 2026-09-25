// World management in the pause menu: the saved worlds (play, export, delete), a new-world form (name
// and an optional seed) and importing `.ocworld` files. Storage and switching live in `save/`; this
// panel draws the list and calls them. main.ts builds it into `#worlds`.

import './worlds.css';
import { Game } from '../wasm/engine.js';
import { message, type Session, switchTo } from '../save/session';
import { newWorld, pack, unpack, type WorldMeta, type WorldStore } from '../save/store';
import { button, h } from './dom';

export class WorldsPanel {
  readonly el = h('section', 'worlds');
  private readonly list = h('ul', 'world-list');
  private readonly status = h('p', 'world-status');
  private readonly form = h('form', 'new-world hidden');
  private readonly nameInput = h('input');
  private readonly seedInput = h('input');
  private readonly file = h('input');

  /** `session` is the world being played, or null when none could be opened. */
  constructor(
    private readonly store: WorldStore,
    private readonly session: Session | null,
    notice = '',
  ) {
    const head = h('div', 'worlds-head');
    head.append(
      h('h2', '', 'Worlds'),
      button('secondary-btn', 'New world', () => {
        this.form.classList.remove('hidden');
        this.nameInput.focus();
      }),
      button('secondary-btn', 'Import', () => this.file.click()),
    );
    this.nameInput.placeholder = 'Name';
    this.nameInput.maxLength = 40;
    this.seedInput.placeholder = 'Seed (optional)';
    this.form.append(
      this.nameInput,
      this.seedInput,
      h('button', 'secondary-btn active', 'Create'),
      button('secondary-btn', 'Cancel', () => this.form.classList.add('hidden')),
    );
    this.form.addEventListener('submit', (e) => {
      e.preventDefault();
      const name = this.nameInput.value.trim() || 'New world';
      this.run(() => this.open(newWorld(name, parseSeed(this.seedInput.value))));
    });
    this.file.type = 'file';
    this.file.accept = '.ocworld';
    this.file.className = 'hidden';
    this.file.addEventListener('change', () => {
      const file = this.file.files?.[0];
      this.file.value = '';
      if (file) this.run(() => this.import(file));
    });
    this.el.append(head, this.form, this.list, this.status, this.file);
    if (session) session.onError = (text) => this.say(text);
    this.say(notice);
    this.run(() => this.refresh());
  }

  /** Shows a line under the list (errors and notices); empty hides it. */
  say(text: string): void {
    this.status.textContent = text;
  }

  private async refresh(): Promise<void> {
    const worlds = await this.store.list();
    this.list.replaceChildren(...worlds.map((w) => this.row(w)));
  }

  private row(w: WorldMeta): HTMLLIElement {
    const current = w.id === this.session?.meta.id;
    const li = h('li', current ? 'world current' : 'world');
    const text = h('div', 'world-text');
    text.append(h('span', 'world-name', w.name), h('span', 'world-meta', describe(w)));
    li.append(text);
    if (current) li.append(h('span', 'world-playing', 'Playing'));
    else li.append(button('secondary-btn', 'Play', () => this.run(() => this.open(w))));
    li.append(button('secondary-btn', 'Export', () => this.run(() => this.export(w))));
    if (!current) li.append(button('secondary-btn danger', 'Delete', () => this.run(() => this.delete(w))));
    return li;
  }

  /** Runs a button's task, showing what went wrong if it fails. */
  private run(task: () => Promise<void>): void {
    task().catch((err: unknown) => this.say(message(err)));
  }

  private open(meta: WorldMeta): Promise<void> {
    this.say('Saving...');
    return switchTo(this.store, this.session, meta);
  }

  /** Downloads a world's newest save as a gzipped `.ocworld` file (saving the open world first). */
  private async export(w: WorldMeta): Promise<void> {
    if (w.id === this.session?.meta.id) await this.session.save();
    const meta = await this.store.get(w.id);
    const bytes = meta?.slot != null ? await this.store.read(meta.id, meta.slot) : null;
    if (!bytes) throw new Error(`"${w.name}" hasn't been saved yet, so there's nothing to export.`);
    const url = URL.createObjectURL(new Blob([(await pack(bytes)) as Uint8Array<ArrayBuffer>]));
    const a = h('a');
    a.href = url;
    a.download = `${w.name.replace(/[\\/:*?"<>|]/g, '_')}.ocworld`;
    a.click();
    setTimeout(() => URL.revokeObjectURL(url), 10_000);
  }

  /** Adds a world from a file. The engine reads it once, which checks it and gives its seed and time. */
  private async import(file: File): Promise<void> {
    const fail = (why: string) => new Error(`Couldn't import ${file.name}: ${why}`);
    const bytes = await unpack(new Uint8Array(await file.arrayBuffer())).catch(() => {
      throw fail('This is not an OpenCraft world file.');
    });
    let game: Game;
    try {
      game = Game.load(bytes, 2);
    } catch (err) {
      throw fail(message(err));
    }
    const name = file.name.replace(/\.ocworld$/i, '') || 'Imported world';
    const meta = { ...newWorld(name, game.seed()), playTime: game.play_seconds(), slot: 0 };
    game.free();
    await this.store.write(meta, await pack(bytes));
    await this.refresh();
    this.say(`Imported "${name}".`);
  }

  private async delete(w: WorldMeta): Promise<void> {
    if (!confirm(`Delete "${w.name}" for good? Export it first if you might want it back.`)) return;
    await this.store.delete(w.id);
    await this.refresh();
  }
}

/** "seed 1337 · 12 min played · 25 Sep 2026, 14:03" (the time it was last saved or opened). */
function describe(w: WorldMeta): string {
  const when = new Date(w.updated).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'short' });
  return `seed ${w.seed} · ${w.slot === null ? 'new' : `${duration(w.playTime)} played`} · ${when}`;
}

function duration(seconds: number): string {
  const min = Math.floor(seconds / 60);
  if (min < 1) return 'under a minute';
  return min < 60 ? `${min} min` : `${Math.floor(min / 60)} h ${min % 60} min`;
}

/** A seed from the form: a whole number as typed, other text hashed (FNV-1a), blank random. */
function parseSeed(text: string): number {
  const t = text.trim();
  if (!t) return crypto.getRandomValues(new Uint32Array(1))[0];
  if (/^\d{1,10}$/.test(t) && Number(t) <= 0xffffffff) return Number(t);
  let hash = 0x811c9dc5;
  for (const c of t) hash = Math.imul(hash ^ c.codePointAt(0)!, 0x01000193) >>> 0;
  return hash;
}
