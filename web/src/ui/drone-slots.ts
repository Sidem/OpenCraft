// Personal-drone orders from the inventory screen (inventory.ts), and the feedback on every slot an order is tied to.
// The Drone button in the backpack header turns on picking: a click (or an Alt-click at any time) marks slots
// instead of moving stacks. The bar under the hotbar then sends the marked stacks to storage (`drone_store_slot`: to
// the box already holding that item, else an empty box, else a "Misc" box, never a box on a belt line) or fills the marked empty slots
// with a stack of something in reach (`drone_fill_slot`; searched among `drone_stock`). `DroneBadges` marks each slot
// an order is tied to (`drone_slots`): waiting (pulsing), or on its way (a ring filling until it lands, an arrow down
// for a fill and up for a store, seconds left) with a faint picture of what is coming or what left. The HUD's hotbar
// wears the same badges (drone.ts). Only the marks live here. To show another state: a case in `DroneBadges.badge`.

import './drone-slots.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';

const GHOST_PX = 64;
/** Items a fill search shows at most. */
const MAX_CHOICES = 8;

/** Badges on the slots the drone's orders are tied to; `roots()[i]` is pack slot `i`. Redraws itself every frame. */
export class DroneBadges {
  private drawn = '';
  private readonly badges = new Map<number, HTMLElement>();

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
    private readonly roots: () => HTMLElement[],
  ) {
    const frame = () => {
      this.draw();
      requestAnimationFrame(frame);
    };
    requestAnimationFrame(frame);
  }

  private draw(): void {
    const g = this.game;
    const rows = g.drone_slots();
    const key = `${rows.join()}|${g.inventory_version()}`;
    if (key === this.drawn) return;
    this.drawn = key;
    const roots = this.roots();
    const shown = new Set<number>();
    for (let k = 0; k + 5 < rows.length; k += 6) {
      const root = roots[rows[k]];
      if (!root) continue;
      shown.add(rows[k]);
      let b = this.badges.get(rows[k]);
      if (!b || b.parentElement !== root) {
        b?.remove();
        b = h('div');
        root.append(b);
        this.badges.set(rows[k], b);
      }
      this.badge(b, rows[k], rows[k + 1] === 1, rows[k + 2], rows[k + 3] === 1, rows[k + 4], rows[k + 5]);
    }
    for (const [slot, b] of this.badges) {
      if (!shown.has(slot)) {
        b.remove();
        this.badges.delete(slot);
      }
    }
  }

  /** One slot's badge: `store` (else a fill) of `item`, waiting or `flying` with `permille` done and `secs` left. */
  private badge(b: HTMLElement, slot: number, store: boolean, item: number, flying: boolean, permille: number, secs: number) {
    const g = this.game;
    b.className = `ds-badge ${store ? 'ds-out' : 'ds-in'} ${flying ? 'ds-fly' : 'ds-wait'}`;
    b.style.setProperty('--p', `${permille / 10}%`);
    const name = g.item_name(item);
    b.title = store
      ? flying ? `The drone is taking the ${name} to storage · back in ${secs} s` : `Waiting for the drone to take the ${name} to storage`
      : flying ? `The drone is bringing ${name} for this slot · lands in ${secs} s` : `Waiting for the drone to bring ${name} here`;
    if (b.dataset.item !== String(item)) {
      const ghost = h('canvas', 'ds-ghost');
      ghost.width = ghost.height = GHOST_PX;
      ghost.getContext('2d')!.drawImage(this.icon(item), 0, 0, GHOST_PX, GHOST_PX);
      b.replaceChildren(ghost, h('span', 'ds-ring'), h('span', 'ds-arrow', store ? '↑' : '↓'), h('span', 'ds-time'));
      b.dataset.item = String(item);
    }
    // The picture shows only where the slot is empty: what is coming, or what just left.
    b.classList.toggle('ds-empty', g.slot_count(slot) === 0);
    b.querySelector('.ds-time')!.textContent = flying ? `${secs}s` : '…';
  }
}

/** The inventory screen's drone controls: `button` goes in the backpack header, `bar` under the hotbar. */
export class DroneSlots {
  readonly button: HTMLButtonElement;
  readonly bar = h('div', 'ds-bar hidden');
  private picking = false;
  private readonly marked = new Set<number>();
  private readonly hint = h('p', 'ds-hint');
  private readonly store = button('secondary-btn', '', () => this.sendToStorage());
  private readonly fill = h('div', 'ds-fill');
  private readonly fillLabel = h('span', 'ds-label');
  private readonly search = h('input', 'ds-search');
  private readonly choices = h('div', 'ds-choices');
  private drawn = '';

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
    private readonly roots: HTMLElement[],
  ) {
    this.button = button('secondary-btn ds-toggle', 'Drone', () => this.setPicking(!this.picking));
    this.button.title = 'Pick slots for your personal drone: send their stacks to storage, or fill empty ones (or Alt-click a slot)';
    roots.forEach((root, i) => root.addEventListener('pointerdown', (e) => this.press(e, i), { capture: true }));
    this.search.type = 'search';
    this.search.placeholder = 'What to bring';
    this.search.setAttribute('aria-label', 'What the drone brings to the empty slots');
    this.search.addEventListener('input', () => this.drawChoices());
    this.fill.append(this.fillLabel, this.search, this.choices);
    const clear = button('secondary-btn', 'Clear', () => this.setPicking(false));
    const actions = h('div', 'ds-actions');
    actions.append(this.store, clear);
    this.bar.append(this.hint, actions, this.fill);
    new DroneBadges(game, icon, () => roots);
    const frame = () => {
      this.tick();
      requestAnimationFrame(frame);
    };
    requestAnimationFrame(frame);
  }

  /** Every frame: the button only with a drone; marks dropped when the screen closes; the bar follows the pack. */
  private tick(): void {
    const ready = this.game.drone_ready();
    this.button.classList.toggle('hidden', !ready);
    const open = this.roots[0]?.offsetParent != null;
    if ((!open || !ready) && (this.picking || this.marked.size > 0)) this.setPicking(false);
    const key = `${this.game.inventory_version()}|${[...this.marked].join()}|${this.picking}`;
    if (key !== this.drawn) {
      this.drawn = key;
      this.drawBar();
    }
  }

  private setPicking(on: boolean): void {
    this.picking = on;
    this.button.classList.toggle('active', on);
    this.button.textContent = on ? 'Drone: picking' : 'Drone';
    for (const i of this.marked) this.roots[i]?.classList.remove('ds-marked');
    this.marked.clear();
    this.search.value = '';
    this.drawBar();
  }

  /** A click while picking (or an Alt-click) marks or unmarks the slot instead of moving its stack. */
  private press(e: PointerEvent, slot: number): void {
    if (e.button !== 0 || !(this.picking || e.altKey) || !this.game.drone_ready()) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    if (!this.picking) this.setPicking(true);
    if (this.marked.has(slot)) this.marked.delete(slot);
    else this.marked.add(slot);
    this.roots[slot].classList.toggle('ds-marked', this.marked.has(slot));
    this.drawBar();
  }

  /** The marked slots holding something it can carry, and the empty ones. */
  private split(): { full: number[]; empty: number[] } {
    const g = this.game;
    const full: number[] = [], empty: number[] = [];
    for (const i of [...this.marked].sort((a, b) => a - b)) {
      const item = g.slot_item(i);
      if (g.slot_count(i) === 0) empty.push(i);
      else if (item !== g.drone_item()) full.push(i);
    }
    return { full, empty };
  }

  private drawBar(): void {
    this.bar.classList.toggle('hidden', !this.picking);
    if (!this.picking) return;
    const { full, empty } = this.split();
    this.hint.textContent = this.marked.size === 0
      ? 'Click slots for the drone: full ones to send to storage, empty ones to fill. It never fills a box on a belt line.'
      : `${this.marked.size} marked`;
    this.store.textContent = `Send ${full.length} ${full.length === 1 ? 'stack' : 'stacks'} to storage`;
    this.store.classList.toggle('hidden', full.length === 0);
    this.fill.classList.toggle('hidden', empty.length === 0);
    this.fillLabel.textContent = `Fill ${empty.length} empty ${empty.length === 1 ? 'slot' : 'slots'} with a stack of:`;
    if (empty.length > 0) this.drawChoices();
  }

  /** What lies in reach and matches the search (the most plentiful first), each a button that fills the slots. */
  private drawChoices(): void {
    const g = this.game;
    const flat = g.drone_stock();
    const query = this.search.value.trim().toLowerCase();
    const items: { item: number; total: number }[] = [];
    for (let k = 0; k + 3 < flat.length; k += 4) {
      if (g.item_name(flat[k]).toLowerCase().includes(query)) items.push({ item: flat[k], total: flat[k + 1] + flat[k + 2] });
    }
    items.sort((a, b) => b.total - a.total);
    const chips: HTMLElement[] = items.slice(0, MAX_CHOICES).map(({ item, total }) => {
      const b = button('ds-choice', '', () => this.fillWith(item));
      const c = h('canvas');
      c.width = c.height = 32;
      c.getContext('2d')!.drawImage(this.icon(item), 0, 0, 32, 32);
      b.append(c, h('span', '', g.item_name(item)), h('b', '', String(total)));
      b.title = `${total} in boxes and machines in reach`;
      return b;
    });
    if (chips.length === 0) chips.push(h('span', 'ds-label', query ? 'None of that in reach.' : 'Nothing in reach.'));
    this.choices.replaceChildren(...chips);
  }

  private sendToStorage(): void {
    const { full } = this.split();
    full.forEach((i) => this.game.drone_store_slot(i));
    this.done(full);
  }

  private fillWith(item: number): void {
    const { empty } = this.split();
    empty.forEach((i) => this.game.drone_fill_slot(i, item));
    this.done(empty);
  }

  /** Unmarks the slots just ordered; picking ends with the last mark. */
  private done(slots: number[]): void {
    for (const i of slots) {
      this.marked.delete(i);
      this.roots[i]?.classList.remove('ds-marked');
    }
    if (this.marked.size === 0) this.setPicking(false);
    else this.drawBar();
  }
}
