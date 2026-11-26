// The space elevator part of its anchor's machine panel (ui/machine.ts), shown on top like a depot's town (ui/town.ts,
// whose line styles it shares): how tall the tether stands, whether its climbers or its town hold it up, the open
// section's order with a bar a line, and what the climbers use a minute with how many minutes the anchor holds. Read
// from the engine (`elevator_panel`) and redrawn only when those numbers change. Hidden for any other machine.

import './town.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

type Pos = [number, number, number];

const ICON_PX = 64;

export class ElevatorSection {
  readonly el = h('section', 'tp-town hidden');
  private readonly heading = h('p', 'mp-status');
  private readonly stall = h('p', 'tp-stall');
  private readonly orderHead = h('h3');
  private readonly order = h('div', 'tp-lines');
  private readonly drawHead = h('h3');
  private readonly draw = h('div', 'tp-lines');
  private drawn = '';

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.el.append(this.heading, this.stall, this.orderHead, this.order, this.drawHead, this.draw);
  }

  /** Redraws for the anchor at `pos`; hidden when no elevator's anchor is there. */
  update(pos: Pos): void {
    const data = this.game.elevator_panel(...pos);
    this.el.classList.toggle('hidden', data.length === 0);
    const key = data.join(',');
    if (data.length === 0 || key === this.drawn) return;
    this.drawn = key;

    const [built, sections, height, stalled, waiting, orderLines] = data;
    this.heading.textContent = `Space elevator · ${fmt(built * height)} of ${fmt(sections * height)} blocks`;
    const why = waiting
      ? 'Waiting: its town is stalled or asleep.'
      : stalled
        ? 'Stalled: the climbers are short of a minute of goods. The section waits until they are here.'
        : '';
    this.stall.textContent = why;
    this.stall.classList.toggle('hidden', why === '');

    this.orderHead.textContent = built < sections ? `Section ${built + 1} of ${sections}` : 'Phase 1 done';
    const order: HTMLElement[] = [];
    for (let i = 0; i < orderLines; i++) {
      const [item, got, want] = data.slice(6 + i * 3, 9 + i * 3);
      order.push(this.line(item, `${fmt(got)} / ${fmt(want)}`, got / Math.max(want, 1), got >= want));
    }
    this.order.replaceChildren(...order);

    const at = 6 + orderLines * 3;
    this.drawHead.textContent = data[at] > 0 ? 'Climbers use a minute' : '';
    this.drawHead.classList.toggle('hidden', data[at] === 0);
    const draw: HTMLElement[] = [];
    for (let i = 0; i < data[at]; i++) {
      const [item, n, held] = data.slice(at + 1 + i * 3, at + 4 + i * 3);
      const minutes = Math.floor(held / Math.max(n, 1));
      const row = this.line(item, `${fmt(n)} · ${fmt(held)} held, ${minutes} min`, -1, minutes > 0);
      row.classList.toggle('short', built > 0 && minutes === 0);
      draw.push(row);
    }
    this.draw.replaceChildren(...draw);
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
