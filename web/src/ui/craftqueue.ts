// Hand-craft queue strip: one chip per queued craft above the hotbar (icon, amount still to come, a
// progress bar), also over the inventory screen. Click a chip to cancel that order and get its materials
// back. The engine owns the queue (`craft_queue`: item, amount left and permille done per order); this
// only draws it. Call `update` every frame: it rebuilds the chips when the queue changes and otherwise
// moves the bars.

import './craftqueue.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

const ICON_PX = 64;

interface Chip {
  root: HTMLButtonElement;
  bar: HTMLElement;
}

export class CraftQueueView {
  private readonly el = h('div', 'craftq hidden');
  private readonly chips: Chip[] = [];
  private shape = '';

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.el.setAttribute('aria-label', 'Crafting queue');
    document.body.append(this.el);
  }

  update(): void {
    const q = this.game.craft_queue();
    const orders = q.length / 3;
    let shape = '';
    for (let i = 0; i < q.length; i += 3) shape += `${q[i]}:${q[i + 1]},`;
    if (shape !== this.shape) this.rebuild(q, orders, shape);
    for (let i = 0; i < orders; i++) this.chips[i].bar.style.width = `${q[i * 3 + 2] / 10}%`;
  }

  private rebuild(q: Uint32Array, orders: number, shape: string): void {
    this.shape = shape;
    this.chips.length = 0;
    this.el.classList.toggle('hidden', orders === 0);
    const g = this.game;
    const chips: HTMLElement[] = [];
    for (let i = 0; i < orders; i++) {
      const item = q[i * 3];
      const root = h('button', 'craftq-chip');
      root.type = 'button';
      root.title = `${g.item_name(item)}: click to cancel and get the materials back`;
      const canvas = h('canvas');
      canvas.width = canvas.height = ICON_PX;
      canvas.getContext('2d')!.drawImage(this.icon(item), 0, 0);
      const bar = h('span', 'craftq-fill');
      const track = h('span', 'craftq-bar');
      track.append(bar);
      root.append(canvas, h('span', 'craftq-n', `×${q[i * 3 + 1]}`), track);
      root.addEventListener('click', () => g.cancel_craft(i));
      this.chips.push({ root, bar });
      chips.push(root);
    }
    this.el.replaceChildren(...chips);
  }
}
