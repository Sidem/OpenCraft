// A harbour's contract work, inside the town part of its depot's panel (ui/town.ts): the product it sends out with
// what is paid toward the next (a bar per raw material), and the products to choose from (`harbour_offers`: ones the
// harbour offers that the player's machines have made). Clicking a product picks it (clicking it again picks none),
// which starts the payment over. Read from the engine (`harbour_contract`, `harbour_offers`) and redrawn only when
// those change; hidden for any other town.

import './harbour.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';

type Pos = [number, number, number];

const ICON_PX = 64;

export class HarbourSection {
  readonly el = h('section', 'hb-harbour hidden');
  private readonly price = h('div', 'tp-lines');
  private readonly offers = h('div', 'hb-offers');
  private readonly note = h('p', 'mp-note');
  private drawn = '';

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.el.append(h('h3', '', 'Contract work'), this.price, h('h3', '', 'Choose what it sends out'), this.note, this.offers);
  }

  /** Redraws for the depot at `pos`; hidden unless a harbour's depot is there. */
  update(pos: Pos): void {
    const g = this.game;
    const contract = g.harbour_contract(...pos);
    this.el.classList.toggle('hidden', contract.length === 0);
    if (contract.length === 0) return;
    const offers = g.harbour_offers(...pos);
    const key = `${contract.join(',')}|${offers.join(',')}`;
    if (key === this.drawn) return;
    this.drawn = key;

    const product = contract[0];
    const lines: HTMLElement[] = [];
    if (product === 0) lines.push(h('p', 'mp-note', 'No product chosen yet.'));
    for (let i = 1; i + 2 < contract.length; i += 3) {
      const [item, n, paid] = contract.slice(i, i + 3);
      lines.push(this.priceLine(item, n, paid));
    }
    this.price.replaceChildren(...lines);

    this.note.textContent = offers.length === 0
      ? 'Nothing yet: it takes on products your machines have made from at least two raw materials (not bauxite or uranium).'
      : `Each one costs ${g.harbour_price_factor()} times its raw materials, paid in raw goods.`;
    this.offers.replaceChildren(...[...offers].map((item) => {
      const b = button('secondary-btn mp-insert', '', () => g.set_harbour_product(...pos, item === product ? 0 : item));
      b.classList.toggle('active', item === product);
      b.append(this.iconCanvas(item), g.item_name(item));
      return b;
    }));
  }

  /** A raw material of the price: its icon and name, paid of how many, and a bar. */
  private priceLine(item: number, n: number, paid: number): HTMLElement {
    const row = h('div', paid >= n ? 'tp-line done' : 'tp-line');
    const bar = h('div', 'mp-bar tp-bar');
    const fill = h('div', 'mp-bar-fill');
    fill.style.transform = `scaleX(${Math.min(paid / Math.max(n, 1), 1)})`;
    bar.append(fill);
    row.append(this.iconCanvas(item), h('span', 'tp-name', this.game.item_name(item)), h('span', 'tp-figure', `${paid} / ${n}`), bar);
    return row;
  }

  private iconCanvas(item: number): HTMLCanvasElement {
    const c = h('canvas', 'mp-chip-icon');
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    return c;
  }
}
