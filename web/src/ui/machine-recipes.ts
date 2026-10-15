// The recipe picker of a machine panel ("Choose what it makes"): one card per recipe, read at a glance. The product
// sits in a tile on the left with the amount a batch makes; its name and the batch time are on the first line; below
// it, "Uses" and a chip for each ingredient (icon, count, name). Byproducts show as "Also gives". A recipe research still
// locks goes last, dimmed, with the tech to finish. Long lists (the upgrader has dozens) get a search box.
// `update` marks the active card and re-sorts only when a lock changed. To change a card: `buildCard`.

import './machine-recipes.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

const ICON_PX = 64;
/** More recipes than this and the list gets a search box. */
const SEARCH_FROM = 8;

const CLOCK =
  '<svg viewBox="0 0 16 16" aria-hidden="true"><circle cx="8" cy="8" r="6.2" fill="none" stroke="currentColor" ' +
  'stroke-width="1.6"/><path d="M8 4.4V8l2.4 1.6" fill="none" stroke="currentColor" stroke-width="1.6" ' +
  'stroke-linecap="round"/></svg>';

interface Card {
  id: number;
  el: HTMLButtonElement;
  lock: HTMLElement;
  /** Lower-case words to search: product and ingredient names. */
  words: string;
  locked: boolean;
}

/** "4 s", "1.5 s", "1 min 30 s". */
export function formatSeconds(s: number): string {
  if (s < 60) return `${Number.isInteger(s) ? s : s.toFixed(1)} s`;
  const m = Math.floor(s / 60);
  const rest = Math.round(s - m * 60);
  return rest === 0 ? `${m} min` : `${m} min ${rest} s`;
}

export class RecipePicker {
  readonly el = h('div', 'rp');
  private readonly bar = h('div', 'rp-bar');
  private readonly search = h('input', 'rp-search') as HTMLInputElement;
  private readonly count = h('span', 'rp-count');
  private readonly list = h('div', 'rp-list');
  private readonly empty = h('p', 'rp-empty hidden', 'No recipe matches.');
  private cards: Card[] = [];
  private order = '';

  /** `pick` is called with a recipe's id when its card is clicked. */
  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
    private readonly pick: (id: number) => void,
  ) {
    this.search.type = 'search';
    this.search.placeholder = 'Search by product or ingredient';
    this.search.setAttribute('aria-label', 'Search recipes');
    this.search.addEventListener('input', () => this.filter());
    this.search.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && this.search.value) {
        // The first Escape only empties the box (the panel sees the prevented default and stays open).
        e.preventDefault();
        this.search.value = '';
        this.filter();
      }
    });
    this.bar.append(this.search, this.count);
    this.el.append(this.bar, this.list, this.empty);
  }

  get isEmpty(): boolean {
    return this.cards.length === 0;
  }

  /** Builds the cards of the recipes machine block `block` makes. */
  build(block: number): void {
    const g = this.game;
    this.cards = Array.from(g.machine_recipes(block), (id) => this.buildCard(id));
    this.search.value = '';
    this.order = '';
    this.bar.classList.toggle('hidden', this.cards.length <= SEARCH_FROM);
    this.count.textContent = `${this.cards.length} recipes`;
    this.list.replaceChildren(...this.cards.map((c) => c.el));
    this.empty.classList.add('hidden');
  }

  /** Marks the card of recipe `active` (or none: -1), and whether cards can be clicked at all. */
  update(active: number, choosable: boolean): void {
    for (const c of this.cards) {
      const tech = this.game.machine_recipe_locked_by(c.id);
      c.locked = tech >= 0;
      c.el.classList.toggle('active', c.id === active);
      c.el.classList.toggle('locked', c.locked);
      c.el.disabled = !choosable || c.locked;
      c.el.setAttribute('aria-pressed', String(c.id === active));
      const note = c.locked ? `Research ${this.game.tech_name(tech)} first` : '';
      if (c.lock.textContent !== note) c.lock.textContent = note;
    }
    // Recipes you can use first, the locked ones after them; each group keeps the engine's order.
    const order = this.cards.map((c) => (c.locked ? 'l' : 'u')).join('');
    if (order === this.order) return;
    this.order = order;
    const sorted = [...this.cards].sort((a, b) => Number(a.locked) - Number(b.locked));
    this.list.replaceChildren(...sorted.map((c) => c.el));
  }

  private filter(): void {
    const words = this.search.value.toLowerCase().split(/\s+/).filter(Boolean);
    let shown = 0;
    for (const c of this.cards) {
      const hit = words.every((w) => c.words.includes(w));
      c.el.classList.toggle('hidden', !hit);
      if (hit) shown++;
    }
    this.count.textContent = words.length === 0 ? `${this.cards.length} recipes` : `${shown} of ${this.cards.length}`;
    this.empty.classList.toggle('hidden', shown > 0);
  }

  private buildCard(id: number): Card {
    const g = this.game;
    const outs = g.machine_recipe_output(id);
    const ins = g.machine_recipe_inputs(id);
    const name = g.item_name(outs[0]);
    const names = [name];

    const tile = h('span', 'rp-tile');
    tile.append(this.iconCanvas(outs[0], 'rp-icon'), h('b', 'rp-amount', `×${outs[1]}`));

    const time = h('span', 'rp-time');
    time.innerHTML = CLOCK;
    time.append(formatSeconds(g.machine_recipe_seconds(id)));
    time.title = 'How long one batch takes at Mk1 speed';
    const top = h('span', 'rp-top');
    top.append(h('b', 'rp-name', name), h('span', 'rp-making', 'Making'), time);

    const cost = h('span', 'rp-cost');
    cost.append(h('span', 'rp-label', 'Uses'));
    for (let k = 0; k + 1 < ins.length; k += 2) {
      const chip = h('span', 'rp-chip');
      const label = g.item_name(ins[k]);
      names.push(label);
      chip.title = `${ins[k + 1]} ${label}`;
      chip.append(this.iconCanvas(ins[k], 'rp-chip-icon'), h('b', '', String(ins[k + 1])), label);
      cost.append(chip);
    }

    const main = h('span', 'rp-main');
    main.append(top, cost);
    if (outs.length > 2) {
      const extra = h('span', 'rp-extra');
      extra.append(h('span', 'rp-label', 'Also gives'));
      for (let k = 2; k + 1 < outs.length; k += 2) {
        const label = g.item_name(outs[k]);
        names.push(label);
        extra.append(h('span', 'rp-chip rp-chip-extra', `${outs[k + 1]} ${label}`));
      }
      main.append(extra);
    }
    const lock = h('span', 'rp-lock');
    main.append(lock);

    const el = h('button', 'rp-card');
    el.type = 'button';
    el.title = 'Click to choose it; click again to clear. What the machine holds comes back to you.';
    el.append(tile, main);
    el.addEventListener('click', () => this.pick(id));
    return { id, el, lock, words: names.join(' ').toLowerCase(), locked: false };
  }

  private iconCanvas(item: number, className: string): HTMLCanvasElement {
    const c = h('canvas', className);
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    return c;
  }
}
