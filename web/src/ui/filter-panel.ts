// A filter's section of the machine panel (ui/machine.ts): where each item goes. Three cards laid out like the filter
// seen from above (left, straight on, right, with items coming in from behind), each listing the items picked to go
// that way; exactly one card is "everything else" (an old filter's two sides may share it, shown as "shared"). A card
// adds items through one searchable picker (the item in the filter now and the inventory first, then every item the
// guide knows), drops them with ×, and "Send everything else here" moves the catch-all to it. While the picker is
// open, Shift-clicking an inventory stack adds that item to its card. Every change is an engine action (`filter_pick`,
// `filter_rest`); what is shown is read back each frame from `machine_filter` (engine `factory/filter_rules.rs`).

import './filter-panel.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';

const ICON_PX = 24;
/** Output numbers, as the engine counts them: straight on, left, right, and "no list" (with everything else). */
const STRAIGHT = 0, LEFT = 1, RIGHT = 2, UNPICKED = 3;
const NAMES = ['Straight on', 'Left', 'Right'];
const ARROWS = ['↑', '←', '→'];
/** How many search results the picker lists. */
const MAX_RESULTS = 40;

interface Rules {
  /** Bit `1 << out` for each output taking everything else. */
  rest: number;
  picks: number[][];
}

/** `machine_filter`'s numbers as rules, or null when the machine isn't a filter. */
export function readRules(data: ArrayLike<number>): Rules | null {
  if (data.length === 0) return null;
  const picks: number[][] = [];
  let i = 1;
  for (let out = 0; out < 3; out++) {
    const n = data[i++];
    picks.push(Array.from({ length: n }, (_, k) => data[i + k]));
    i += n;
  }
  return { rest: data[0], picks };
}

export class FilterSection {
  readonly el = h('section', 'fp');
  private readonly cards = h('div', 'fp-cards');
  private readonly picker = h('div', 'fp-picker hidden');
  private readonly pickerTitle = h('span', 'fp-picker-title');
  private readonly search = h('input', 'fp-search') as HTMLInputElement;
  private readonly results = h('div', 'fp-results');
  private readonly guideItems: number[];
  private pos: [number, number, number] = [0, 0, 0];
  private rules: Rules = { rest: 1, picks: [[], [], []] };
  private held = 0;
  private inventory = new Map<number, number>();
  /** The output the picker adds to, or -1 while it is closed. */
  private target = -1;

  constructor(private readonly game: Game, private readonly icon: (item: number) => HTMLCanvasElement) {
    this.guideItems = Array.from(game.guide_items());
    this.search.type = 'search';
    this.search.placeholder = 'Search items';
    this.search.addEventListener('input', () => this.drawResults());
    this.search.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        e.stopPropagation();
        this.openPicker(-1);
      } else if (e.key === 'Enter') {
        (this.results.querySelector('.fp-result') as HTMLButtonElement | null)?.click();
      }
    });
    const head = h('div', 'fp-picker-head');
    head.append(this.pickerTitle, button('close-btn fp-picker-close', '×', () => this.openPicker(-1)));
    this.picker.append(head, this.search, this.results);
    this.el.append(
      h('h3', '', 'Where items go'),
      h('p', 'fp-help', 'Items come in from behind. Pick items for a way out; everything you have not picked takes the way marked "Everything else".'),
      this.cards,
      this.picker,
    );
  }

  /** Redraws for the filter at `pos` (call when its rules, its item or the inventory changed). */
  update(pos: [number, number, number], data: ArrayLike<number>, held: number, inventory: Map<number, number>): void {
    if (pos.some((v, i) => v !== this.pos[i])) this.openPicker(-1);
    this.pos = pos;
    this.rules = readRules(data) ?? this.rules;
    this.held = held;
    this.inventory = inventory;
    // Grid order: straight on above, left and right beside the filter, "in" below.
    const middle = h('div', 'fp-middle');
    middle.append(h('span', 'fp-in-arrow', '↑'), h('span', '', 'Items come in'));
    this.cards.replaceChildren(this.card(STRAIGHT), this.card(LEFT), this.card(RIGHT), middle);
    if (this.target >= 0) this.drawResults();
  }

  /** Shift-click on an inventory stack: adds its item to the card the picker is open for. */
  shiftItem(item: number): boolean {
    if (this.target < 0 || item === 0) return false;
    this.game.filter_pick(...this.pos, item, this.target);
    return true;
  }

  private card(out: number): HTMLElement {
    const r = this.rules;
    const isRest = (r.rest & (1 << out)) !== 0;
    const shared = isRest && (r.rest & ~(1 << out)) !== 0;
    const el = h('div', `fp-card fp-${['straight', 'left', 'right'][out]}`);
    el.classList.toggle('fp-rest', isRest);
    el.classList.toggle('fp-target', this.target === out);
    const head = h('div', 'fp-card-head');
    head.append(h('span', 'fp-arrow', ARROWS[out]), h('span', 'fp-name', NAMES[out]));
    el.append(head);

    const chips = h('div', 'fp-chips');
    for (const item of r.picks[out]) chips.append(this.chip(item));
    if (isRest) {
      const label = shared ? 'Everything else (shared with the other side)' : 'Everything else';
      chips.prepend(h('span', 'fp-rest-badge', label));
    } else if (r.picks[out].length === 0) {
      chips.append(h('span', 'fp-empty', 'Nothing'));
    }
    el.append(chips);

    const actions = h('div', 'fp-card-actions');
    const full = r.picks[out].length >= this.game.filter_max_picks();
    if (!isRest || r.picks[out].length > 0) {
      const add = button('secondary-btn fp-add', full ? 'Full' : this.target === out ? 'Adding…' : '+ Add item', () =>
        this.openPicker(this.target === out ? -1 : out),
      );
      add.disabled = full;
      actions.append(add);
    }
    if (!isRest || shared) {
      const rest = button('secondary-btn fp-make-rest', 'Everything else here', () => {
        this.game.filter_rest(...this.pos, out);
        if (this.target === out) this.openPicker(-1);
      });
      rest.title = `Send everything not picked for another way ${NAMES[out].toLowerCase()}`;
      actions.append(rest);
    }
    el.append(actions);
    return el;
  }

  private chip(item: number): HTMLElement {
    const chip = h('span', 'fp-chip');
    const remove = button('fp-chip-x', '×', () => this.game.filter_pick(...this.pos, item, UNPICKED));
    remove.title = `Stop picking ${this.game.item_name(item)}: it goes with everything else`;
    chip.append(this.iconCanvas(item), h('span', 'fp-chip-name', this.game.item_name(item)), remove);
    return chip;
  }

  private openPicker(out: number): void {
    const was = this.target;
    this.target = out;
    this.picker.classList.toggle('hidden', out < 0);
    if (out < 0) {
      if (was >= 0) this.cards.querySelectorAll('.fp-target').forEach((c) => c.classList.remove('fp-target'));
      return;
    }
    this.pickerTitle.textContent = `Add to ${NAMES[out].toLowerCase()} ${ARROWS[out]}`;
    this.search.value = '';
    this.update(this.pos, this.rulesData(), this.held, this.inventory);
    this.search.focus({ preventScroll: true });
    this.picker.scrollIntoView({ block: 'nearest' });
  }

  /** The current rules back as `machine_filter`'s numbers, to redraw without a frame. */
  private rulesData(): number[] {
    return [this.rules.rest, ...this.rules.picks.flatMap((list) => [list.length, ...list])];
  }

  /** The picker's list: with no search, the item in the filter and the inventory's items; with one, every match. */
  private drawResults(): void {
    const g = this.game;
    const query = this.search.value.trim().toLowerCase();
    const seen = new Set<number>();
    const rows: HTMLElement[] = [];
    const add = (item: number, note: string) => {
      if (item === 0 || seen.has(item) || rows.length >= MAX_RESULTS) return;
      const name = g.item_name(item);
      if (query && !name.toLowerCase().includes(query)) return;
      seen.add(item);
      rows.push(this.result(item, name, note));
    };
    add(this.held, 'in the filter now');
    for (const [item, n] of this.inventory) add(item, `×${n} in your inventory`);
    if (query) for (const item of this.guideItems) add(item, '');
    if (rows.length === 0) rows.push(h('p', 'fp-empty', query ? 'No item by that name.' : 'Type to search every item.'));
    else if (!query) rows.push(h('p', 'fp-empty', 'Type to search every item.'));
    this.results.replaceChildren(...rows);
  }

  private result(item: number, name: string, note: string): HTMLElement {
    const now = this.rules.picks.findIndex((list) => list.includes(item));
    const row = button('fp-result', '', () => this.game.filter_pick(...this.pos, item, this.target));
    row.append(this.iconCanvas(item), h('span', 'fp-result-name', name));
    if (now === this.target) row.disabled = true;
    const where = now === this.target ? 'already here' : now >= 0 ? `now goes ${NAMES[now].toLowerCase()}` : note;
    if (where) row.append(h('span', 'fp-result-note', where));
    return row;
  }

  private iconCanvas(item: number): HTMLCanvasElement {
    const c = h('canvas', 'fp-icon');
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0, ICON_PX, ICON_PX);
    return c;
  }
}
