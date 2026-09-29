// Build menu in the inventory screen: every hand recipe as a compact tile (icon, name, how many it
// makes), grouped by the engine's recipe groups, with a text search (names of outputs and materials)
// and a state filter (all, can craft, missing items, locked). A tile's details (description,
// materials with have/need, the time it takes, what locks it) show in one info card on hover or keyboard
// focus. Clicking a tile queues one craft (craftqueue.ts shows the queue), Shift-click up to 5. The inventory
// panel calls `update` when the inventory changes.
// To show another fact about a recipe: add it to `showInfo`.

import './crafting.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

const ICON_PX = 64;
/** Shift-clicking a tile makes up to this many at once. */
const BULK_CRAFT = 5;
const FILTERS = [
  { id: 'all', label: 'All' },
  { id: 'ready', label: 'Can craft' },
  { id: 'short', label: 'Missing items' },
  { id: 'locked', label: 'Locked' },
] as const;
type State = 'ready' | 'short' | 'locked';
type Filter = (typeof FILTERS)[number]['id'];

interface Tile {
  id: number;
  root: HTMLButtonElement;
  group: number;
  /** Lower-case output and material names, for the search. */
  words: string;
  state: State;
}

export class BuildMenu {
  /** Called after something was crafted. */
  onCraft: () => void = () => {};
  readonly el = h('section', 'inv-build');
  private readonly search = h('input', 'craft-search');
  private readonly filterButtons = new Map<Filter, HTMLButtonElement>();
  private readonly sections: { root: HTMLElement; tiles: Tile[] }[] = [];
  private readonly tiles: Tile[] = [];
  private readonly empty = h('p', 'craft-empty hidden', 'No recipe matches.');
  private readonly info = h('div', 'craft-info hidden');
  private filter: Filter = 'all';
  /** The tile whose card is showing. */
  private hovered: Tile | null = null;

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.search.type = 'search';
    this.search.placeholder = 'Search recipes or materials';
    this.search.setAttribute('aria-label', 'Search recipes or materials');
    this.search.addEventListener('input', () => this.applyFilter());
    const filters = h('div', 'craft-filters');
    filters.setAttribute('role', 'group');
    filters.setAttribute('aria-label', 'Show recipes');
    for (const f of FILTERS) {
      const b = h('button', 'craft-filter');
      b.type = 'button';
      b.addEventListener('click', () => {
        this.filter = f.id;
        this.applyFilter();
      });
      this.filterButtons.set(f.id, b);
      filters.append(b);
    }
    const head = h('div', 'craft-head');
    head.append(h('h3', '', 'Build'), this.search);

    const groups = game.recipe_group_count();
    for (let g = 0; g < groups; g++) {
      const root = h('div', 'craft-section');
      root.append(h('h4', '', game.recipe_group_name(g)), h('div', 'craft-grid'));
      this.sections.push({ root, tiles: [] });
    }
    for (let r = 0; r < game.recipe_count(); r++) {
      const tile = this.makeTile(r);
      this.tiles.push(tile);
      this.sections[tile.group].tiles.push(tile);
      this.sections[tile.group].root.lastElementChild!.append(tile.root);
    }
    const list = h('div', 'craft-list');
    list.append(...this.sections.map((s) => s.root), this.empty);
    this.el.append(head, filters, list, this.info);
  }

  /** Recomputes which recipes can be crafted; call when the inventory changed. */
  update(): void {
    const g = this.game;
    for (const t of this.tiles) {
      t.state = g.recipe_locked_by(t.id) >= 0 ? 'locked' : g.craftable_times(t.id) > 0 ? 'ready' : 'short';
      t.root.dataset.state = t.state;
      t.root.setAttribute('aria-disabled', String(t.state !== 'ready'));
    }
    this.applyFilter();
    if (this.hovered) this.showInfo(this.hovered);
  }

  /** Hides the info card (the screen closed). */
  hideInfo(): void {
    this.hovered = null;
    this.info.classList.add('hidden');
  }

  private makeTile(r: number): Tile {
    const g = this.game;
    const out = g.recipe_output(r);
    const root = h('button', 'craft-tile');
    root.type = 'button';
    const canvas = h('canvas', 'craft-icon');
    canvas.width = canvas.height = ICON_PX;
    canvas.getContext('2d')!.drawImage(this.icon(out), 0, 0);
    root.append(canvas, h('span', 'craft-name', g.item_name(out)));
    const n = g.recipe_output_count(r);
    if (n > 1 && g.tool_uses(out) === 0) root.append(h('span', 'craft-yield', `×${n}`));
    root.append(h('span', 'craft-lock'));
    const names = [g.item_name(out)];
    const flat = g.recipe_inputs(r);
    for (let k = 0; k + 1 < flat.length; k += 2) names.push(g.item_name(flat[k]));
    const tile: Tile = { id: r, root, group: g.recipe_group(r), words: names.join(' ').toLowerCase(), state: 'short' };
    root.addEventListener('click', (e) => {
      if (tile.state === 'ready' && g.craft(r, e.shiftKey ? BULK_CRAFT : 1) > 0) this.onCraft();
    });
    root.addEventListener('pointerenter', () => this.showInfo(tile));
    root.addEventListener('focus', () => this.showInfo(tile));
    root.addEventListener('pointerleave', () => this.hideInfo());
    root.addEventListener('blur', () => this.hideInfo());
    return tile;
  }

  /** Shows the tiles that match the search and filter; counts per filter go on its button. */
  private applyFilter(): void {
    const words = this.search.value.trim().toLowerCase().split(/\s+/).filter(Boolean);
    const matches = (t: Tile) => words.every((w) => t.words.includes(w));
    const counts: Record<Filter, number> = { all: 0, ready: 0, short: 0, locked: 0 };
    for (const t of this.tiles) {
      if (!matches(t)) continue;
      counts.all++;
      counts[t.state]++;
    }
    for (const f of FILTERS) {
      const b = this.filterButtons.get(f.id)!;
      b.textContent = `${f.label} ${counts[f.id]}`;
      b.setAttribute('aria-pressed', String(f.id === this.filter));
    }
    let shown = 0;
    for (const s of this.sections) {
      let any = false;
      for (const t of s.tiles) {
        const show = matches(t) && (this.filter === 'all' || t.state === this.filter);
        t.root.classList.toggle('hidden', !show);
        any ||= show;
        shown += Number(show);
      }
      s.root.classList.toggle('hidden', !any);
    }
    this.empty.classList.toggle('hidden', shown > 0);
  }

  /** Fills the info card for `t` and places it beside the tile, inside the window. */
  private showInfo(t: Tile): void {
    const g = this.game;
    this.hovered = t;
    const out = g.recipe_output(t.id);
    const title = h('h4', '', g.item_name(out));
    const n = g.recipe_output_count(t.id);
    if (n > 1 && g.tool_uses(out) === 0) title.append(h('span', 'craft-yield', ` ×${n}`));
    const inputs = h('div', 'craft-inputs');
    const flat = g.recipe_inputs(t.id);
    for (let k = 0; k + 1 < flat.length; k += 2) {
      const item = flat[k], need = flat[k + 1], have = g.item_total(item);
      const row = h('div', have < need ? 'craft-input short' : 'craft-input');
      const icon = h('canvas', 'craft-input-icon');
      icon.width = icon.height = ICON_PX;
      icon.getContext('2d')!.drawImage(this.icon(item), 0, 0);
      row.append(icon, h('span', '', g.item_name(item)), h('span', 'craft-have', `${have}/${need}`));
      inputs.append(row);
    }
    const secs = g.recipe_tenths(t.id) / 10;
    const parts = g.recipe_part_crafts(t.id);
    const time = h('p', 'craft-time', `Takes ${secs} s by hand`);
    if (parts > 0) time.append(` (${parts} part crafts first, queued for you)`);
    const tech = g.recipe_locked_by(t.id);
    const status =
      t.state === 'locked'
        ? `Research ${g.tech_name(tech)} to unlock it (T).`
        : t.state === 'ready'
          ? `Click to queue it · Shift-click for up to ${BULK_CRAFT}`
          : 'Gather the missing materials to craft it.';
    const note = h('p', `craft-status ${t.state}`, status);
    this.info.replaceChildren(title, h('p', 'craft-blurb', g.recipe_blurb(t.id)), inputs, time, note);
    this.info.classList.remove('hidden');
    const r = t.root.getBoundingClientRect(), box = this.info.getBoundingClientRect();
    const right = r.right + 8 + box.width <= window.innerWidth;
    const x = right ? r.right + 8 : Math.max(8, r.left - 8 - box.width);
    const y = Math.min(Math.max(8, r.top), window.innerHeight - box.height - 8);
    this.info.style.transform = `translate(${x}px, ${y}px)`;
  }
}
