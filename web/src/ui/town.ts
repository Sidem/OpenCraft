// The town part of a depot's machine panel (ui/machine.ts), shown on top: the town and its stage, whether it is stalled,
// the open order with a progress bar a line, each upkeep good with how many minutes the depot holds, and the perk. Read
// from the engine (`town_panel`, `town_heading`, `town_perk`) and redrawn only when those numbers change; the depot's
// slots, the take button and the put-in buttons stay the machine panel's. Hidden for any other machine. A harbour's
// contract work and its product choice follow the perk (ui/harbour.ts).
// A new kind or stage needs no change here: names and numbers come from the engine.

import './town.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import { HarbourSection } from './harbour';

type Pos = [number, number, number];

const ICON_PX = 64;

export class TownSection {
  readonly el = h('section', 'tp-town hidden');
  private readonly heading = h('p', 'mp-status');
  private readonly stall = h('p', 'tp-stall');
  private readonly perk = h('p', 'mp-note');
  private readonly orderHead = h('h3');
  private readonly order = h('div', 'tp-lines');
  private readonly upkeepHead = h('h3');
  private readonly upkeep = h('div', 'tp-lines');
  private readonly harbour: HarbourSection;
  private drawn = '';

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.harbour = new HarbourSection(game, icon);
    this.el.append(this.heading, this.stall, this.perk, this.harbour.el, this.orderHead, this.order, this.upkeepHead, this.upkeep);
  }

  /** Redraws for the depot at `pos`; false (and hidden) when no town's depot is there. */
  update(pos: Pos): boolean {
    const g = this.game;
    const data = g.town_panel(...pos);
    this.el.classList.toggle('hidden', data.length === 0);
    if (data.length === 0) return false;
    this.harbour.update(pos);
    const key = data.join(',');
    if (key === this.drawn) return true;
    this.drawn = key;

    const [, built, stages, stalled, orderLines] = data;
    this.heading.textContent = g.town_heading(...pos);
    this.stall.textContent = stalled ? 'Stalled: a minute of its upkeep is missing. Its order and perk wait until it is here.' : '';
    this.stall.classList.toggle('hidden', !stalled);
    this.perk.textContent = g.town_perk(...pos);

    this.orderHead.textContent = built < stages ? `For the next stage (${built + 1} of ${stages})` : 'Fully grown';
    const order: HTMLElement[] = [];
    for (let i = 0; i < orderLines; i++) {
      const [item, got, want] = data.slice(5 + i * 3, 8 + i * 3);
      order.push(this.line(item, `${fmt(got)} / ${fmt(want)}`, got / Math.max(want, 1), got >= want));
    }
    this.order.replaceChildren(...order);

    const at = 5 + orderLines * 3;
    this.upkeepHead.textContent = built === 0 ? 'The camp will use a minute' : 'Uses a minute';
    const upkeep: HTMLElement[] = [];
    for (let i = 0; i < data[at]; i++) {
      const [item, n, held] = data.slice(at + 1 + i * 3, at + 4 + i * 3);
      const minutes = Math.floor(held / Math.max(n, 1));
      const stock = minutes === 0 ? 'less than a minute held' : `${fmt(held)} held, ${minutes} min`;
      const row = this.line(item, `${fmt(n)} · ${stock}`, -1, minutes > 0);
      row.classList.toggle('short', built > 0 && minutes === 0);
      upkeep.push(row);
    }
    this.upkeep.replaceChildren(...upkeep);
    return true;
  }

  /** A line: the item's icon and name, a figure, and a progress bar unless `fill` is negative. */
  private line(item: number, figure: string, fill: number, done: boolean): HTMLElement {
    const row = h('div', done ? 'tp-line done' : 'tp-line');
    const c = h('canvas', 'mp-chip-icon');
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    row.append(c, h('span', 'tp-name', this.game.item_name(item)), h('span', 'tp-figure', figure));
    if (fill >= 0) {
      const bar = h('div', 'mp-bar tp-bar');
      const f = h('div', 'mp-bar-fill');
      f.style.transform = `scaleX(${Math.min(fill, 1)})`;
      bar.append(f);
      row.append(bar);
    }
    return row;
  }
}

function fmt(n: number): string {
  return n.toLocaleString('en-US');
}