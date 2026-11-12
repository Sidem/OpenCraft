// The storage box screen's label picker: one compact button beside the box's title showing its label (icon and
// name); clicking it opens a list of every label the engine knows (`box_label_names`, `box_label_icons`: engine
// `factory/box_label.rs`) with a search field on top. Typing filters by the label's name or its icon's item name
// ("pick" finds Tools); Enter takes the first match, Escape closes the list, a click outside closes it too.
// "Contents" makes the box show what it holds most of; a category shows its icon. Picking sends `label_box` (an
// action, so co-op players see the same label); the shown label is read back from the engine each frame. Hidden for
// a wagon, which has no label.

import './box-label.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

const ICON_PX = 20;

interface Option {
  row: HTMLButtonElement;
  /** Lower-case name and icon item name, what the search matches. */
  words: string;
}

export class BoxLabel {
  readonly el = h('div', 'box-label');
  private readonly button = h('button', 'box-label-btn') as HTMLButtonElement;
  private readonly menu = h('div', 'box-label-menu hidden');
  private readonly search = h('input', 'box-label-search') as HTMLInputElement;
  private readonly options: Option[] = [];
  private readonly names: string[];
  private readonly icons: (HTMLCanvasElement | null)[];
  private box: [number, number, number] | null = null;
  private shown = -2;

  constructor(private readonly game: Game, icon: (item: number) => HTMLCanvasElement) {
    this.names = game.box_label_names().split('\n');
    const items = game.box_label_icons();
    const paint = (item: number) => {
      if (!item) return null;
      const c = h('canvas');
      c.width = c.height = ICON_PX;
      c.getContext('2d')!.drawImage(icon(item), 0, 0, ICON_PX, ICON_PX);
      return c;
    };
    this.icons = this.names.map((_, i) => paint(items[i]));
    this.button.type = 'button';
    this.button.title = 'What the box shows on top: what it holds most of, or a label you pick';
    this.button.addEventListener('click', () => this.toggle(this.menu.classList.contains('hidden')));

    this.search.type = 'search';
    this.search.placeholder = 'Search labels';
    this.search.addEventListener('input', () => this.filter());
    this.search.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        e.stopPropagation();
        this.toggle(false);
      } else if (e.key === 'Enter') {
        this.options.find((o) => !o.row.classList.contains('hidden'))?.row.click();
      }
    });
    const list = h('div', 'box-label-list');
    this.names.forEach((name, i) => {
      const row = h('button', 'box-label-row') as HTMLButtonElement;
      row.type = 'button';
      const pic = paint(items[i]);
      row.append(pic ?? h('span', 'box-label-blank'), name);
      row.title = items[i] ? `The box shows ${name.toLowerCase()}` : 'The box shows what it holds most of';
      row.addEventListener('click', () => {
        if (this.box) this.game.label_box(...this.box, i);
        this.toggle(false);
      });
      list.append(row);
      this.options.push({ row, words: `${name} ${items[i] ? game.item_name(items[i]) : 'contents auto'}`.toLowerCase() });
    });
    this.menu.append(this.search, list);
    this.el.append(this.button, this.menu);
    // A click anywhere else closes the list.
    document.addEventListener('pointerdown', (e) => {
      if (!this.el.contains(e.target as Node)) this.toggle(false);
    });
  }

  /** Shows the label of the box at `box` (null: none open). */
  open(box: [number, number, number] | null): void {
    this.box = box;
    this.shown = -2;
    this.toggle(false);
    this.update();
  }

  /** Shows the box's label on the button; cheap when it is unchanged. */
  update(): void {
    const label = this.box ? this.game.box_label(...this.box) : -1;
    if (label === this.shown) return;
    this.shown = label;
    this.el.classList.toggle('hidden', label < 0);
    if (label < 0) return;
    const pic = this.icons[label]?.cloneNode() as HTMLCanvasElement | undefined;
    if (pic) pic.getContext('2d')!.drawImage(this.icons[label]!, 0, 0);
    this.button.replaceChildren(pic ?? h('span', 'box-label-blank'), this.names[label], h('span', 'box-label-caret', '▾'));
    this.options.forEach((o, i) => o.row.setAttribute('aria-selected', String(i === label)));
  }

  private toggle(open: boolean): void {
    this.menu.classList.toggle('hidden', !open);
    if (!open) return;
    this.search.value = '';
    this.filter();
    this.search.focus();
  }

  private filter(): void {
    const q = this.search.value.trim().toLowerCase();
    for (const o of this.options) o.row.classList.toggle('hidden', !!q && !o.words.includes(q));
  }
}
