// Build menu in the inventory screen: every hand recipe as a compact tile (icon, name, how many it makes), grouped by
// the engine's recipe groups (every tier of one building side by side: `recipe_order`) and paged instead of scrolled:
// a category starts a new page, and so does a full page (as many tiles as the menu's height and width hold). The page
// header above the grid, the pager below it, the mouse wheel and a row of category tabs all change page. A state
// filter (unlocked, the default; all, can craft, missing items, locked) and a text search (names of outputs and
// materials, plus related words: craft-terms.ts). A search also finds the items no hand recipe makes (guide tiles) and
// ranks instead of grouping, in one list that fills each page: first what is called by the words typed (the best
// match marked, Enter opens it; shown even if locked), then what is made from it or related. Hover or focus shows a tile's card (craft-info.ts). Clicking a tile
// queues one craft (craftqueue.ts shows the queue), Shift-click up to 5, right-click takes one apart (take-apart.ts);
// clicking one you cannot craft yet opens its page in the item guide (item-guide.ts). The inventory panel calls
// `update` when the inventory changes. To rank search results differently: `nameScore` in craft-terms.ts.

import './crafting.css';
import './craft-nav.css';
import type { Game } from '../wasm/engine.js';
import { BULK_CRAFT, CraftInfo } from './craft-info';
import { hasWord, nameScore, relatedWords } from './craft-terms';
import { h } from './dom';
import { ItemGuide, wayWords } from './item-guide';
import { Pager, tilesPerPage, type PageInfo } from './pager';
import { takeApart } from './take-apart';

const ICON_PX = 64;
/** A tile's height and the grid's gap, in px: the same numbers as `.craft-grid` in crafting.css. */
export const TILE_H = 80;
export const TILE_GAP = 6;
const FILTERS = [
  { id: 'unlocked', label: 'Unlocked' },
  { id: 'all', label: 'All' },
  { id: 'ready', label: 'Can craft' },
  { id: 'short', label: 'Missing items' },
  { id: 'locked', label: 'Locked' },
] as const;
/** `guide`: an item no hand recipe makes, shown only for a search. */
type State = 'ready' | 'short' | 'locked' | 'guide';
type Filter = (typeof FILTERS)[number]['id'];
/** The guide tiles' category: the items a search finds that no hand recipe makes. */
const ELSEWHERE = 'Made in machines or found';
/** A search's one section (categories are 0 and up): every result in rank order, filling each page. */
const RESULTS = -1;

interface Page {
  /** Its category, or a search section. */
  section: number;
  /** Which page of its section this is, of how many. */
  part: number;
  parts: number;
  tiles: Tile[];
}

interface Tile {
  /** The hand recipe, or -1 for a guide tile. */
  id: number;
  item: number;
  root: HTMLButtonElement;
  group: number;
  /** Where it sorts inside its category (the engine's `recipe_order`). */
  order: number;
  /** The item's name, and the lower-case output and material names for the search. */
  name: string;
  words: string;
  state: State;
  /** How well its name answers the search (`nameScore`; 3 when there is none). */
  score: number;
}

export class BuildMenu {
  /** Called after something was crafted. */
  onCraft: () => void = () => {};
  readonly el = h('section', 'inv-build');
  readonly guide: ItemGuide;
  private readonly search = h('input', 'craft-search');
  private readonly filterButtons = new Map<Filter, HTMLButtonElement>();
  private readonly groupNames: string[] = [];
  private readonly tiles: Tile[] = [];
  private readonly empty = h('p', 'craft-empty hidden', 'Nothing matches.');
  private readonly info: CraftInfo;
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
  /** The search's best match (Enter opens it), if its name answers the search. */
  private best: Tile | null = null;

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.search.type = 'search';
    this.search.placeholder = 'Search any item or material';
    this.search.setAttribute('aria-label', 'Search any item or material');
    this.search.addEventListener('input', () => {
      this.pager.page = 0;
      this.applyFilter();
    });
    this.search.addEventListener('keydown', (e) => this.searchKey(e));
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
    const byHand = new Set(this.tiles.map((t) => t.item));
    this.groupNames.push(ELSEWHERE);
    for (const item of game.guide_items()) if (!byHand.has(item)) this.tiles.push(this.makeGuideTile(item));
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
    this.info = new CraftInfo(game, icon);
    this.guide = new ItemGuide(game, icon);
    this.el.append(head, filters, this.tabs, list, this.pager.el, this.info.el, this.guide.el);
  }

  /** Recomputes which recipes can be crafted; call when the inventory changed. */
  update(): void {
    const g = this.game;
    for (const t of this.tiles) {
      if (t.state === 'guide') continue;
      t.state = g.recipe_locked_by(t.id) >= 0 ? 'locked' : g.craftable_times(t.id) > 0 ? 'ready' : 'short';
      t.root.dataset.state = t.state;
    }
    this.applyFilter();
    if (this.hovered) this.showInfo(this.hovered);
    this.guide.refresh();
  }

  /** Hides the info card (the screen closed). */
  hideInfo(): void {
    this.hovered = null;
    this.info.hide();
  }

  private makeTile(r: number): Tile {
    const g = this.game;
    const out = g.recipe_output(r);
    const root = this.tileRoot(out);
    const n = g.recipe_output_count(r);
    if (n > 1 && g.tool_uses(out) === 0) root.append(h('span', 'craft-yield', `×${n}`));
    root.append(h('span', 'craft-lock'));
    const names = [g.item_name(out)];
    const flat = g.recipe_inputs(r);
    for (let k = 0; k + 1 < flat.length; k += 2) names.push(g.item_name(flat[k]));
    const group = g.recipe_group(r);
    const words = `${names.join(' ')} ${g.recipe_group_name(group)} ${relatedWords(names[0])}`.toLowerCase();
    const order = g.recipe_order(r);
    const tile: Tile = { id: r, item: out, root, group, order, name: names[0], words, state: 'short', score: 3 };
    root.addEventListener('click', (e) => {
      if (tile.state !== 'ready') this.openGuide(out);
      else if (g.craft(r, e.shiftKey ? BULK_CRAFT : 1) > 0) this.onCraft();
    });
    root.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      if (takeApart(g, r, e.shiftKey)) this.onCraft();
    });
    this.hover(tile);
    return tile;
  }

  /** A tile for an item no hand recipe makes: its search words are its name and what goes into it. */
  private makeGuideTile(item: number): Tile {
    const name = this.game.item_name(item);
    const root = this.tileRoot(item);
    root.dataset.state = 'guide';
    const words = `${name} ${wayWords(this.game, item)} ${relatedWords(name)}`.toLowerCase();
    const group = this.groupNames.length - 1;
    const tile: Tile = { id: -1, item, root, group, order: item, name, words, state: 'guide', score: 3 };
    root.addEventListener('click', () => this.openGuide(item));
    this.hover(tile);
    return tile;
  }

  /** A tile's button with the item's icon and name. */
  private tileRoot(item: number): HTMLButtonElement {
    const root = h('button', 'craft-tile');
    root.type = 'button';
    const canvas = h('canvas', 'craft-icon');
    canvas.width = canvas.height = ICON_PX;
    canvas.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    root.append(canvas, h('span', 'craft-name', this.game.item_name(item)));
    return root;
  }

  private hover(tile: Tile): void {
    tile.root.addEventListener('pointerenter', () => this.showInfo(tile));
    tile.root.addEventListener('focus', () => this.showInfo(tile));
    tile.root.addEventListener('pointerleave', () => this.hideInfo());
    tile.root.addEventListener('blur', () => this.hideInfo());
  }

  private showInfo(t: Tile): void {
    this.hovered = t;
    this.info.show(t, this.groupNames[t.group]);
  }

  private openGuide(item: number): void {
    this.hideInfo();
    this.guide.open(item);
  }

  /** Enter opens the best match's page; the down arrow moves into the results. */
  private searchKey(e: KeyboardEvent): void {
    if (e.key === 'Enter' && this.best) {
      e.preventDefault();
      this.openGuide(this.best.item);
    } else if (e.key === 'ArrowDown' && this.shown.length > 0) {
      e.preventDefault();
      this.shown[0].root.focus();
    }
  }

  /** Finds the tiles that match the search and filter (counts per filter go on its button) and pages them; a search
   * ranks them by name. */
  private applyFilter(): void {
    const words = this.search.value.trim().toLowerCase().split(/\s+/).filter(Boolean);
    const counts: Record<Filter, number> = { unlocked: 0, all: 0, ready: 0, short: 0, locked: 0 };
    const visible: Tile[] = [];
    for (const t of this.tiles) {
      if (!words.every((w) => hasWord(t.words, w))) continue;
      t.score = nameScore(t.name, words);
      if (t.state === 'guide') {
        if (words.length > 0 && (this.filter === 'all' || this.filter === 'unlocked')) visible.push(t);
        continue;
      }
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
    // A stable sort: within a score, the category order stays.
    if (words.length > 0) visible.sort((a, b) => a.score - b.score);
    this.best?.root.classList.remove('best');
    this.best = visible.length > 0 && visible[0].score < 3 ? visible[0] : null;
    this.best?.root.classList.add('best');
    this.empty.classList.toggle('hidden', visible.length > 0);
    this.layout(visible, words.length > 0);
  }

  /** Cuts `visible` (in order) into pages: a new section (a category; a `search` is one section, so its best results
   * share the first page) starts a page, and a page holds as many tiles as the grid has room for. Nothing is laid out
   * while hidden. */
  private layout(visible: Tile[], search: boolean): void {
    const size = tilesPerPage(this.grid, TILE_H, TILE_GAP);
    if (size === 0) return;
    const sectionOf = (t: Tile) => (search ? RESULTS : t.group);
    this.pages = [];
    const infos: PageInfo[] = [];
    for (let i = 0; i < visible.length; ) {
      const section = sectionOf(visible[i]);
      let end = i;
      while (end < visible.length && sectionOf(visible[end]) === section) end++;
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
        const name = this.sectionName(section);
        const title = parts > 1 ? `${name} (${p + 1} of ${parts})` : name;
        this.pages.push({ section, part: p + 1, parts, tiles: visible.slice(cuts[p], cuts[p + 1]) });
        infos.push({ title, gap: p === 0 });
      }
      i = end;
    }
    this.pager.set(infos);
    this.makeTabs();
    this.showPage();
  }

  private sectionName(section: number): string {
    return section === RESULTS ? `Results for “${this.search.value.trim()}”` : this.groupNames[section];
  }

  /** One tab per section that has a page; clicking jumps to the section's first page. */
  private makeTabs(): void {
    const seen = new Set<number>();
    const buttons: HTMLButtonElement[] = [];
    this.pages.forEach((page, index) => {
      if (seen.has(page.section)) return;
      seen.add(page.section);
      const b = h('button', 'craft-tab', this.sectionName(page.section));
      b.type = 'button';
      b.dataset.section = String(page.section);
      b.addEventListener('click', () => this.pager.go(index));
      buttons.push(b);
    });
    this.tabs.replaceChildren(...buttons);
    this.tabs.classList.toggle('hidden', buttons.length < 2);
  }

  /** Labels a page-step button with where it leads: another page of this section, or the section after it. */
  private stepButton(b: HTMLButtonElement, to: Page | undefined, from: Page | undefined, before: string, after: string) {
    b.classList.toggle('hidden', !to || !from);
    if (!to || !from) return;
    const lead = this.pages.indexOf(to) > this.pages.indexOf(from) ? 'Next' : 'Previous';
    const word = to.section !== from.section ? `${lead}: ${this.sectionName(to.section)}` : `${lead} page`;
    b.textContent = before + word + after;
  }

  /** Puts the current page's tiles in the grid, leaving the DOM alone when they are already there. */
  private showPage(): void {
    const index = this.pager.page;
    const page = this.pages[index];
    const tiles = page ? page.tiles : [];
    this.pageTitle.textContent = page ? this.sectionName(page.section) : '';
    this.pageNote.textContent = page && page.parts > 1 ? `Page ${page.part} of ${page.parts}` : '';
    this.stepButton(this.pageBack, this.pages[index - 1], page, '‹ ', '');
    this.stepButton(this.pageNext, this.pages[index + 1], page, '', ' ›');
    for (const b of Array.from(this.tabs.children)) {
      const on = page !== undefined && (b as HTMLElement).dataset.section === String(page.section);
      b.setAttribute('aria-pressed', String(on));
    }
    if (tiles.length === this.shown.length && tiles.every((t, i) => t === this.shown[i])) return;
    this.shown = tiles;
    this.grid.replaceChildren(...tiles.map((t) => t.root));
  }

  /** Whether the chosen state filter lets `t` through. "Unlocked" still shows a locked tile the search names: you
   * typed its name, so it should come up (dimmed, with its lock). */
  private passes(t: Tile): boolean {
    if (this.filter === 'all') return true;
    return this.filter === 'unlocked' ? t.state !== 'locked' || t.score < 3 : t.state === this.filter;
  }
}

/** Whether two tiles are tiers of one building (the engine's `recipe_order` gives a building's tiers one rank). */
function sameFamily(a: Tile, b: Tile): boolean {
  return a.group === b.group && Math.floor(a.order / 8) === Math.floor(b.order / 8);
}
