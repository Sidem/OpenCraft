// The build menu's info card (crafting.ts): what hovering or focusing a tile shows beside it. A hand recipe's card
// has the building's facts (craft-card.ts), description, materials with have/need (a missing one says where it comes
// from: item-guide.ts), the time it takes and what to do next; a guide tile's card (an item no hand recipe makes)
// lists the ways to get it. To show another fact about a recipe: add it to `show`.

import './craft-info.css';
import type { Game } from '../wasm/engine.js';
import { factsBlock } from './craft-card';
import { h } from './dom';
import { sourceHint, waySummary } from './item-guide';
import { takeApartNote } from './take-apart';

const ICON_PX = 64;
/** Shift-clicking a tile makes up to this many at once. */
export const BULK_CRAFT = 5;

/** What the card needs of a tile: its hand recipe (-1 for a guide tile), item, state and button. */
export interface InfoTile {
  id: number;
  item: number;
  state: string;
  root: HTMLElement;
}

export class CraftInfo {
  readonly el = h('div', 'craft-info hidden');

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {}

  hide(): void {
    this.el.classList.add('hidden');
  }

  /** Fills the card for `t` (in the build menu section `section`) and places it beside the tile. */
  show(t: InfoTile, section: string): void {
    const g = this.game;
    const out = t.item;
    const title = h('h4', '', g.item_name(out));
    const n = t.id < 0 ? 1 : g.recipe_output_count(t.id);
    if (n > 1 && g.tool_uses(out) === 0) title.append(h('span', 'craft-yield', ` ×${n}`));
    const head = h('div', 'card-head');
    head.append(this.canvas(out, 'card-icon'), title, h('span', 'card-group', section));
    if (t.id < 0) {
      const ways = waySummary(g, out).map((line) => h('p', 'craft-way', line));
      const note = h('p', 'craft-status ready', 'Click for how to get it.');
      this.place(t, [head, h('p', 'craft-blurb', 'No hand recipe makes it:'), ...ways, note]);
      return;
    }
    const inputs = h('div', 'craft-inputs');
    const flat = g.recipe_inputs(t.id);
    for (let k = 0; k + 1 < flat.length; k += 2) {
      const item = flat[k], need = flat[k + 1], have = g.item_total(item);
      const row = h('div', have < need ? 'craft-input short' : 'craft-input');
      const name = h('span', '', g.item_name(item));
      // What is missing says where it comes from, so the card alone answers the next question.
      if (have < need) name.append(h('span', 'craft-source', ` · ${sourceHint(g, item)}`));
      row.append(this.canvas(item, 'craft-input-icon'), name, h('span', 'craft-have', `${have}/${need}`));
      inputs.append(row);
    }
    const parts = g.recipe_part_crafts(t.id);
    const time = h('p', 'craft-time', `Takes ${g.recipe_tenths(t.id) / 10} s by hand`);
    if (parts > 0) time.append(` (${parts} part crafts first, queued for you)`);
    const status =
      t.state === 'locked'
        ? `Research ${g.tech_name(g.recipe_locked_by(t.id))} to unlock it (T). Click for how to get it.`
        : t.state === 'ready'
          ? `Click to queue it · Shift-click for up to ${BULK_CRAFT}`
          : 'Click for where the missing materials come from.';
    const facts = factsBlock(g, out);
    const apart = takeApartNote(g, t.id);
    const blurb = h('p', 'craft-blurb', g.recipe_blurb(t.id));
    const note = h('p', `craft-status ${t.state}`, status);
    this.place(t, [head, ...(facts ? [facts] : []), blurb, inputs, time, note, ...(apart ? [apart] : [])]);
  }

  /** Fills the card with `parts` and places it beside tile `t`, inside the window. */
  private place(t: InfoTile, parts: HTMLElement[]): void {
    this.el.replaceChildren(...parts);
    this.el.classList.remove('hidden');
    const r = t.root.getBoundingClientRect(), box = this.el.getBoundingClientRect();
    const right = r.right + 8 + box.width <= window.innerWidth;
    const x = right ? r.right + 8 : Math.max(8, r.left - 8 - box.width);
    const y = Math.min(Math.max(8, r.top), window.innerHeight - box.height - 8);
    this.el.style.transform = `translate(${x}px, ${y}px)`;
  }

  private canvas(item: number, className: string): HTMLCanvasElement {
    const c = h('canvas', className);
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    return c;
  }
}
