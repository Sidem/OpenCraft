// Build menu in the inventory screen: every hand recipe as a compact tile (icon, name, how many it
// makes), grouped by the engine's recipe groups (every tier of one building side by side: `recipe_order`) and paged
// instead of scrolled: a category starts a new page, and so does a full page (as many tiles as the menu's height and
// width hold). The page header above the grid, the pager below it, the mouse wheel and a row of category tabs all
// change page. A text search (names of outputs and materials, plus related words: craft-terms.ts)
// and a state filter (unlocked, the default; all, can craft, missing items, locked). A tile's details (a building's
// facts: craft-card.ts, description, materials with have/need, the time it takes, what locks it) show in one info card on hover or keyboard
// focus. Clicking a tile queues one craft (craftqueue.ts shows the queue), Shift-click up to 5; right-clicking takes
// one apart (take-apart.ts). The inventory panel calls `update` when the inventory changes.
// To show another fact about a recipe: add it to `showInfo`.

import './crafting.css';
import './craft-nav.css';
import type { Game } from '../wasm/engine.js';
import { factsBlock } from './craft-card';
import { relatedWords } from './craft-terms';
import { h } from './dom';
import { Pager, tilesPerPage, type PageInfo } from './pager';
import { takeApart, takeApartNote } from './take-apart';

const ICON_PX = 64;
/** A tile's height and the grid's gap, in px: the same numbers as `.craft-grid` in crafting.css. */
export const TILE_H = 80;
export const TILE_GAP = 6;
/** Shift-clicking a tile makes up to this many at once. */
const BULK_CRAFT = 5;
const FILTERS = [
  { id: 'unlocked', label: 'Unlocked' },
  { id: 'all', label: 'All' },
  { id: 'ready', label: 'Can craft' },
  { id: 'short', label: 'Missing items' },
  { id: 'locked', label: 'Locked' },
] as const;
type State = 'ready' | 'short' | 'locked';
type Filter = (typeof FILTERS)[number]['id'];

interface Page {
  title: string;
  group: number;
  /** Which page of its category this is, of how many. */
  part: number;
  parts: number;
  tiles: Tile[];
}

interface Tile {
  id: number;
  root: HTMLButtonElement;
  group: number;
  /** Where it sorts inside its category (the engine's `recipe_order`). */
  order: number;
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
  private readonly groupNames: string[] = [];
  private readonly tiles: Tile[] = [];
  private readonly empty = h('p', 'craft-empty hidden', 'No recipe matches.');
  private readonly info = h('div', 'craft-info hidden');
  private readonly tabs = h('div', 'craft-tabs');
  private readonly pageTitle = h('h4', 'craft-page-title');
  private readonly pageNote = h('span', 'craft-page-note');
  private readonly pageBack = h('button', 'craft-page-step');
  private readonly pageNext = h('button', 'craft-page-step more');
  private readonly grid = h('div', 'craft-grid');
  private readonly pager = new Pager();
  /** The tiles of each page, for the current search and filter. */
  private pages: Page[] = [];
  /** The tiles on show, to leave the DOM alone (and any hover on it) when a redraw changes nothing. */
  private shown: Tile[] = [];
  private filter: Filter = 'unlocked';
  /** The tile whose card is showing. */
  private hovered: Tile | null = null;

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.search.type = 'search';
    this.search.placeholder = 'Search recipes or materials';
    this.search.setAttribute('aria-label', 'Search recipes or materials');
    this.search.addEventListener('input', () => {
      this.pager.page = 0;
      this.applyFilter();
    });
    const filters = h('div', 'craft-filters');
    filters.setAttribute('role', 'group');
    filters.setAttribute('aria-label', 'Show recipes');
    for (const f of FILTERS) {
      const b = h('button', 'craft-filter');
      b.type = 'button';
      b.addEventListener('click', () => {
        this.filter = f.id;
        this.pager.page = 0;
        this.applyFilter();
      });
      this.filterButtons.set(f.id, b);
      filters.append(b);
    }
    const head = h('div', 'craft-head');
    head.append(h('h3', '', 'Build'), this.search);

    for (let g = 0; g < game.recipe_group_count(); g++) this.groupNames.push(game.recipe_group_name(g));
    for (let r = 0; r < game.recipe_count(); r++) this.tiles.push(this.makeTile(r));
    // Tiles keep their category order, and every tier of a building stands together, whatever the recipe order.
    this.tiles.sort((a, b) => a.group - b.group || a.order - b.order);

    const pageHead = h('div', 'craft-page-head');
    for (const [b, d] of [[this.pageBack, -1], [this.pageNext, 1]] as const) {
      b.type = 'button';
      b.addEventListener('click', () => this.pager.go(this.pager.page + d));
    }
    pageHead.append(this.pageBack, this.pageTitle, this.pageNote, this.pageNext);
    const list = h('div', 'craft-list');
    list.append(pageHead, this.grid, this.empty);
    this.pager.onChange = () => this.showPage();
    this.pager.wheel(list);
    // The page size follows the menu's size: lay the pages out again whenever the grid is resized.
    new ResizeObserver(() => this.applyFilter()).observe(this.grid);
    this.el.append(head, filters, this.tabs, list, this.pager.el, this.info);
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
    const group = g.recipe_group(r);
    const words = `${names.join(' ')} ${g.recipe_group_name(group)} ${relatedWords(names[0])}`.toLowerCase();
    const tile: Tile = { id: r, root, group, order: g.recipe_order(r), words, state: 'short' };
    root.addEventListener('click', (e) => {
      if (tile.state === 'ready' && g.craft(r, e.shiftKey ? BULK_CRAFT : 1) > 0) this.onCraft();
    });
    root.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      if (takeApart(g, r, e.shiftKey)) this.onCraft();
    });
    root.addEventListener('pointerenter', () => this.showInfo(tile));
    root.addEventListener('focus', () => this.showInfo(tile));
    root.addEventListener('pointerleave', () => this.hideInfo());
    root.addEventListener('blur', () => this.hideInfo());
    return tile;
  }

  /** Finds the tiles that match the search and filter (counts per filter go on its button) and pages them. */
  private applyFilter(): void {
    const words = this.search.value.trim().toLowerCase().split(/\s+/).filter(Boolean);
    const matches = (t: Tile) => words.every((w) => hasWord(t.words, w));
    const counts: Record<Filter, number> = { unlocked: 0, all: 0, ready: 0, short: 0, locked: 0 };
    const visible: Tile[] = [];
    for (const t of this.tiles) {
      if (!matches(t)) continue;
      counts.all++;
      counts[t.state]++;
      if (t.state !== 'locked') counts.unlocked++;
      if (this.passes(t)) visible.push(t);
    }
    for (const f of FILTERS) {
      const b = this.filterButtons.get(f.id)!;
      b.textContent = `${f.label} ${counts[f.id]}`;
      b.setAttribute('aria-pressed', String(f.id === this.filter));
    }
    this.empty.classList.toggle('hidden', visible.length > 0);
    this.layout(visible);
  }

  /** Cuts `visible` (in category order) into pages: a new category starts a page, and a page holds as many
   * tiles as the grid has room for. Nothing is laid out while the menu is hidden. */
  private layout(visible: Tile[]): void {
    const size = tilesPerPage(this.grid, TILE_H, TILE_GAP);
    if (size === 0) return;
    this.pages = [];
    const infos: PageInfo[] = [];
    for (let i = 0; i < visible.length; ) {
      const group = visible[i].group;
      let end = i;
      while (end < visible.length && visible[end].group === group) end++;
      // A page ends before a building's tiers (Mk1, Mk2, ...) that would be split over two pages, unless they are
      // more than a page.
      const cuts: number[] = [i];
      while (cuts[cuts.length - 1] < end) {
        const start = cuts[cuts.length - 1];
        let stop = Math.min(end, start + size);
        let family = stop;
        while (family > start && stop < end && sameFamily(visible[family - 1], visible[family])) family--;
        if (family > start) stop = family;
        cuts.push(stop);
      }
      const parts = cuts.length - 1;
      for (let p = 0; p < parts; p++) {
        const name = this.groupNames[group];
        const title = parts > 1 ? `${name} (${p + 1} of ${parts})` : name;
        this.pages.push({ title, group, part: p + 1, parts, tiles: visible.slice(cuts[p], cuts[p + 1]) });
        infos.push({ title, gap: p === 0 });
      }
      i = end;
    }
    this.pager.set(infos);
    this.makeTabs();
    this.showPage();
  }

  /** One tab per category that has a page; clicking jumps to the category's first page. */
  private makeTabs(): void {
    const seen = new Set<number>();
    const buttons: HTMLButtonElement[] = [];
    this.pages.forEach((page, index) => {
      if (seen.has(page.group)) return;
      seen.add(page.group);
      const b = h('button', 'craft-tab', this.groupNames[page.group]);
      b.type = 'button';
      b.dataset.group = String(page.group);
      b.addEventListener('click', () => this.pager.go(index));
      buttons.push(b);
    });
    this.tabs.replaceChildren(...buttons);
    this.tabs.classList.toggle('hidden', buttons.length < 2);
  }

  /** Labels a page-step button with where it leads: another page of this category, or the category after it. */
  private stepButton(b: HTMLButtonElement, to: Page | undefined, from: Page | undefined, before: string, after: string) {
    b.classList.toggle('hidden', !to || !from);
    if (!to || !from) return;
    const next = to.part > from.part || to.group > from.group;
    const lead = next ? 'Next' : 'Previous';
    const word = to.group !== from.group ? `${lead}: ${this.groupNames[to.group]}` : `${lead} page`;
    b.textContent = before + word + after;
  }

  /** Puts the current page's tiles in the grid, leaving the DOM alone when they are already there. */
  private showPage(): void {
    const index = this.pager.page;
    const page = this.pages[index];
    const tiles = page ? page.tiles : [];
    this.pageTitle.textContent = page ? this.groupNames[page.group] : '';
    this.pageNote.textContent = page && page.parts > 1 ? `Page ${page.part} of ${page.parts}` : '';
    this.stepButton(this.pageBack, this.pages[index - 1], page, '‹ ', '');
    this.stepButton(this.pageNext, this.pages[index + 1], page, '', ' ›');
    for (const b of Array.from(this.tabs.children)) {
      b.setAttribute('aria-pressed', String(page !== undefined && (b as HTMLElement).dataset.group === String(page.group)));
    }
    if (tiles.length === this.shown.length && tiles.every((t, i) => t === this.shown[i])) return;
    this.shown = tiles;
    this.grid.replaceChildren(...tiles.map((t) => t.root));
  }

  /** Whether the chosen state filter lets `t` through. */
  private passes(t: Tile): boolean {
    if (this.filter === 'all') return true;
    return this.filter === 'unlocked' ? t.state !== 'locked' : t.state === this.filter;
  }

  /** Fills the info card for `t` and places it beside the tile, inside the window. */
  private showInfo(t: Tile): void {
    const g = this.game;
    this.hovered = t;
    const out = g.recipe_output(t.id);
    const title = h('h4', '', g.item_name(out));
    const n = g.recipe_output_count(t.id);
    if (n > 1 && g.tool_uses(out) === 0) title.append(h('span', 'craft-yield', ` ×${n}`));
    const icon = h('canvas', 'card-icon');
    icon.width = icon.height = ICON_PX;
    icon.getContext('2d')!.drawImage(this.icon(out), 0, 0);
    const head = h('div', 'card-head');
    head.append(icon, title, h('span', 'card-group', this.groupNames[t.group]));
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
    const facts = factsBlock(g, out);
    const apart = takeApartNote(g, t.id);
    this.info.replaceChildren(head, ...(facts ? [facts] : []), h('p', 'craft-blurb', g.recipe_blurb(t.id)), inputs, time, note);
    if (apart) this.info.append(apart);
    this.info.classList.remove('hidden');
    const r = t.root.getBoundingClientRect(), box = this.info.getBoundingClientRect();
    const right = r.right + 8 + box.width <= window.innerWidth;
    const x = right ? r.right + 8 : Math.max(8, r.left - 8 - box.width);
    const y = Math.min(Math.max(8, r.top), window.innerHeight - box.height - 8);
    this.info.style.transform = `translate(${x}px, ${y}px)`;
  }
}

/** Whether two tiles are tiers of one building (the engine's `recipe_order` gives a building's tiers one rank). */
function sameFamily(a: Tile, b: Tile): boolean {
  return a.group === b.group && Math.floor(a.order / 8) === Math.floor(b.order / 8);
}

/** Whether `text` has a word that starts with `w` (a plural also finds the singular: "belts" finds "belt"); a
 * very short `w` must be a whole word, so "ai" finds the AI Lab but not "air". */
function hasWord(text: string, w: string): boolean {
  const padded = ` ${text} `;
  if (w.length <= 2) return padded.includes(` ${w} `);
  return padded.includes(` ${w}`) || (w.length > 3 && w.endsWith('s') && padded.includes(` ${w.slice(0, -1)}`));
}
