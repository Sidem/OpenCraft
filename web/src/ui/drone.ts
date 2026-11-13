// The personal drone's panel (Y): orders for the drone the player carries (the engine's `helpers/drone/`; its keys,
// restocking and HUD line are `drone_hands.rs`). From the top: what it is doing and has waiting (each waiting order can
// be dropped); the whole-pack orders (put away, keep the hand topped up); a search over everything in boxes and
// machine outputs in reach (fetch a stack, or all of it) that, for an item with none in reach, offers to fetch the
// parts and craft it, or says where it comes from; and the pack's stacks, each sendable to storage. All state is the
// engine's: this redraws from the `drone_*` getters while open. To add an order: an engine `Job`, a `drone_*` method
// in `api/drone.rs`, a button here.

import './machine.css';
import './drone.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import { DroneBadges } from './drone-slots';
import { sourceHint } from './item-guide';

const ICON_PX = 32;
const REDRAW_MS = 250;
/** Rows of things not in reach a search lists at most. */
const MAX_MISSING = 6;
const RESTOCK_KEY = 'opencraft.droneRestock';

interface Stocked {
  item: number;
  boxes: number;
  machines: number;
  stack: number;
}

export class DronePanel {
  /** Called after the screen closes. `resume` is true when play should continue. */
  onClose: (resume: boolean) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp drone');
  private readonly absent = h('p', 'dr-note', 'You carry no personal drone. Craft one and keep it in your pack: it works from there.');
  private readonly content = h('div', 'dr-content');
  private readonly queue = h('div', 'dr-queue');
  private readonly putAway = button('secondary-btn', 'Put away', () => this.game.drone_put_away());
  private readonly restock = h('input');
  private readonly search = h('input', 'dr-search');
  private readonly stock = h('div', 'dr-list');
  private readonly pack = h('div', 'dr-list');
  /** Every item's name, and the hand recipe that makes it (-1 for none), for the search. */
  private readonly items: { item: number; name: string; recipe: number }[] = [];
  private drawn = { queue: '', stock: '', pack: '' };
  private timer = 0;

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    const restock = load();
    if (restock !== null) game.set_drone_restock(restock);
    // The HUD's hotbar shows what the drone is bringing to or taking from each slot.
    const hotbar = () => Array.from(document.querySelectorAll<HTMLElement>('#hotbar > .slot')).slice(0, game.hotbar_size());
    new DroneBadges(game, icon, hotbar);
    const byItem = new Map<number, number>();
    for (let r = game.recipe_count() - 1; r >= 0; r--) byItem.set(game.recipe_output(r), r);
    for (const item of game.guide_items()) if (!byItem.has(item)) byItem.set(item, -1);
    for (const [item, recipe] of byItem) this.items.push({ item, name: game.item_name(item), recipe });

    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;
    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(h('h2', '', 'Personal drone'), h('span', 'mp-keys', 'Y or click outside to return · Shift+Y: a stack more of what you hold'), close);

    this.restock.type = 'checkbox';
    this.restock.addEventListener('change', () => {
      game.set_drone_restock(this.restock.checked);
      save(this.restock.checked);
    });
    const restockLabel = h('label', 'dr-check');
    restockLabel.append(this.restock, 'Top up the stack in my hand when it runs low');
    this.putAway.title = 'Takes every stack outside your hotbar whose item already has a box nearby back to that box';
    const orders = h('div', 'dr-orders');
    orders.append(this.putAway, restockLabel);

    this.search.type = 'search';
    this.search.placeholder = 'Search what to fetch';
    this.search.setAttribute('aria-label', 'Search what to fetch');
    this.search.addEventListener('input', () => this.draw());
    this.content.append(
      h('h3', '', 'Doing now'), this.queue, orders,
      h('h3', '', 'Fetch from boxes and machines'), this.search, this.stock,
      h('h3', '', 'Send to storage'),
      h('p', 'dr-note', 'Each stack goes to the box already holding that item, else to an empty box, else to a box you labelled "Misc" (which takes anything). Never into a box on a belt line, nor an empty box with another label.'),
      this.pack,
    );
    const body = h('div', 'mp-body');
    body.append(this.absent, this.content);
    this.dialog.append(head, body);
    this.backdrop.append(this.dialog);
    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen || e.repeat) return;
      if (e.code === 'KeyY' && e.target !== this.search) {
        e.preventDefault();
        this.close(true);
      } else if (e.key === 'Escape') {
        e.preventDefault();
        this.close(false);
      }
    });
    document.body.append(this.backdrop);
  }

  get isOpen(): boolean {
    return !this.backdrop.classList.contains('hidden');
  }

  open(): void {
    this.drawn = { queue: '-', stock: '-', pack: '-' }; // never a real key, so all three draw
    this.backdrop.classList.remove('hidden');
    this.draw();
    this.search.focus();
    this.timer = window.setInterval(() => this.draw(), REDRAW_MS);
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    window.clearInterval(this.timer);
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  /** Redraws the parts whose numbers changed. */
  private draw(): void {
    const g = this.game;
    const ready = g.drone_ready();
    this.absent.classList.toggle('hidden', ready);
    this.content.classList.toggle('hidden', !ready);
    if (!ready) return;
    this.restock.checked = g.drone_restock();
    const kinds = g.drone_put_away_kinds();
    this.putAway.textContent = kinds > 0 ? `Put away ${kinds} ${kinds === 1 ? 'kind' : 'kinds'}` : 'Nothing to put away';
    this.putAway.disabled = kinds === 0;
    this.drawQueue();
    this.drawStock();
    this.drawPack();
  }

  /** The trip in flight, then the waiting orders, each with a button that drops it. */
  private drawQueue(): void {
    const g = this.game;
    const rows = g.drone_jobs();
    const key = rows.join();
    if (key === this.drawn.queue) return;
    this.drawn.queue = key;
    const what = (item: number, n: number) => (item ? `${g.item_name(item)} ×${n}` : `${n} things`);
    const lines: HTMLElement[] = [];
    let waiting = 0;
    for (let k = 0; k + 3 < rows.length; k += 4) {
      const [kind, item, n, extra] = [rows[k], rows[k + 1], rows[k + 2], rows[k + 3]];
      const row = h('div', 'dr-job');
      if (kind >= 10) {
        row.classList.add('flying');
        row.append(`${kind === 10 ? 'Fetching' : 'Storing'} ${what(item, n)} · back in ${extra} s`);
      } else {
        const text = ['Fetch ' + what(item, n), 'Store ' + what(item, n), 'Put away', `Fetch the parts, then craft ${what(item, n)}`][kind];
        const index = waiting++;
        const drop = button('dr-drop', '×', () => g.drone_cancel(index));
        drop.title = 'Drop this order';
        row.append(h('span', '', text), drop);
      }
      lines.push(row);
    }
    if (lines.length === 0) lines.push(h('p', 'dr-note', `Waiting by you. It reaches ${g.drone_range()} blocks.`));
    this.queue.replaceChildren(...lines);
  }

  /** What lies in reach and matches the search; then, for a search, what does not and how to get it. */
  private drawStock(): void {
    const g = this.game;
    const flat = g.drone_stock();
    const query = this.search.value.trim().toLowerCase();
    const key = `${query}|${g.inventory_version()}|${flat.join()}`;
    if (key === this.drawn.stock) return;
    this.drawn.stock = key;
    const all: Stocked[] = [];
    for (let k = 0; k + 3 < flat.length; k += 4) all.push({ item: flat[k], boxes: flat[k + 1], machines: flat[k + 2], stack: flat[k + 3] });
    const name = (item: number) => g.item_name(item);
    const shown = all.filter((s) => name(s.item).toLowerCase().includes(query)).sort((a, b) => name(a.item).localeCompare(name(b.item)));
    const rows = shown.map((s) => this.stockRow(s));
    if (query.length >= 2) {
      const inReach = new Set(all.map((s) => s.item));
      const missing = this.items.filter((i) => !inReach.has(i.item) && i.name.toLowerCase().includes(query));
      rows.push(...missing.slice(0, MAX_MISSING).map((i) => this.missingRow(i.item, i.recipe)));
    }
    if (rows.length === 0) rows.push(h('p', 'dr-note', query ? 'Nothing by that name.' : 'No boxes or machine outputs in reach hold anything.'));
    this.stock.replaceChildren(...rows);
  }

  private stockRow(s: Stocked): HTMLElement {
    const g = this.game;
    const total = s.boxes + s.machines;
    const where = [s.boxes ? `${s.boxes} in boxes` : '', s.machines ? `${s.machines} in machines` : ''].filter(Boolean).join(' · ');
    const buttons = [button('secondary-btn', total > s.stack ? 'A stack' : `All ${total}`, () => g.drone_fetch(s.item, Math.min(s.stack, total)))];
    if (total > s.stack) buttons.push(button('secondary-btn', `All ${total}`, () => g.drone_fetch(s.item, total)));
    return this.row(s.item, `${g.item_name(s.item)}`, where, buttons);
  }

  /** An item none of which lies in reach: craft it from fetched parts when that works, else where it comes from. */
  private missingRow(item: number, recipe: number): HTMLElement {
    const g = this.game;
    const name = g.item_name(item);
    if (recipe >= 0 && g.recipe_locked_by(recipe) >= 0) return this.row(item, name, `None in reach · research ${g.tech_name(g.recipe_locked_by(recipe))} to make it`, []);
    const fetches = recipe >= 0 ? g.drone_craft_fetches(recipe, 1) : new Uint32Array();
    if (fetches.length === 0) return this.row(item, name, `None in reach · ${sourceHint(g, item) || 'not found nearby'}`, []);
    const parts: string[] = [];
    for (let k = 1; k + 1 < fetches.length; k += 2) parts.push(`${fetches[k + 1]} ${g.item_name(fetches[k])}`);
    const sub = parts.length > 0 ? `None in reach · it can fetch ${parts.join(', ')} and craft it` : 'None in reach · you have the parts';
    return this.row(item, name, sub, [button('secondary-btn', 'Fetch parts & craft', () => g.drone_craft(recipe, 1))]);
  }

  /** The pack's stacks (not the drone itself), each with a button that sends it to a box. */
  private drawPack(): void {
    const g = this.game;
    const key = String(g.inventory_version());
    if (key === this.drawn.pack) return;
    this.drawn.pack = key;
    const rows: HTMLElement[] = [];
    for (let i = 0; i < g.inventory_size(); i++) {
      const item = g.slot_item(i);
      if (item === 0 || item === g.drone_item()) continue;
      const where = i < g.hotbar_size() ? `Hotbar ${i + 1}` : 'Backpack';
      rows.push(this.row(item, `${g.item_name(item)} ×${g.slot_count(i)}`, where, [button('secondary-btn', 'Store', () => g.drone_store_slot(i))]));
    }
    if (rows.length === 0) rows.push(h('p', 'dr-note', 'Nothing in your pack to store.'));
    this.pack.replaceChildren(...rows);
  }

  private row(item: number, name: string, sub: string, buttons: HTMLElement[]): HTMLElement {
    const row = h('div', 'dr-row');
    const icon = h('canvas', 'dr-icon');
    icon.width = icon.height = ICON_PX;
    icon.getContext('2d')!.drawImage(this.icon(item), 0, 0, ICON_PX, ICON_PX);
    const text = h('div', 'dr-text');
    text.append(h('span', 'dr-name', name), h('span', 'dr-sub', sub));
    const acts = h('div', 'dr-acts');
    acts.append(...buttons);
    row.append(icon, text, acts);
    return row;
  }
}

/** The restock choice saved on this device, or null. */
function load(): boolean | null {
  try {
    const v = localStorage.getItem(RESTOCK_KEY);
    return v === null ? null : v === '1';
  } catch {
    return null;
  }
}

function save(on: boolean): void {
  try {
    localStorage.setItem(RESTOCK_KEY, on ? '1' : '0');
  } catch {
    // storage blocked: the choice lasts this visit
  }
}
