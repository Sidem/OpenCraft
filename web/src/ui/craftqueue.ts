// Hand-craft queue strip, above the hotbar (also over the inventory screen): one group per queued order, and
// in it one chip per step still to run, in running order: the parts first (planks, plates), what was asked
// for last (the box), each with the amount still to come and, on the step being worked on, a progress bar.
// Finished steps drop off the front. The ✕ on a group cancels the order and returns its materials and the
// parts already made. The engine owns the queue (`craft_queue`, `craft_steps`); this only draws it. Call
// `update` every frame: it rebuilds the groups when the queue changes and otherwise moves the bar.

import './craftqueue.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';

const ICON_PX = 64;

export class CraftQueueView {
  private readonly el = h('div', 'craftq hidden');
  /** The progress bar of the step being worked on in each order (only the first order's moves). */
  private bars: HTMLElement[] = [];
  private shape = '';

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.el.setAttribute('aria-label', 'Crafting queue');
    document.body.append(this.el);
  }

  update(): void {
    const g = this.game;
    const orders = g.craft_queue().length / 3;
    const steps: Uint32Array[] = [];
    let shape = '';
    for (let o = 0; o < orders; o++) {
      const s = g.craft_steps(o);
      steps.push(s);
      for (let i = 0; i < s.length; i += 3) shape += `${s[i]}:${s[i + 1]},`;
      shape += '|';
    }
    if (shape !== this.shape) this.rebuild(steps, shape);
    this.bars.forEach((bar, o) => (bar.style.width = `${steps[o][2] / 10}%`));
  }

  private rebuild(steps: Uint32Array[], shape: string): void {
    this.shape = shape;
    this.bars = [];
    this.el.classList.toggle('hidden', steps.length === 0);
    const g = this.game;
    const groups = steps.map((s, order) => {
      const group = h('div', 'craftq-order');
      const last = s.length / 3 - 1;
      for (let i = 0; i <= last; i++) {
        const item = s[i * 3];
        const chip = h('div', i === last ? 'craftq-chip' : 'craftq-chip part');
        chip.title = `${g.item_name(item)}${i === last ? '' : ' (a part, made first)'}`;
        const canvas = h('canvas');
        canvas.width = canvas.height = ICON_PX;
        canvas.getContext('2d')!.drawImage(this.icon(item), 0, 0);
        const track = h('span', 'craftq-bar');
        const fill = h('span', 'craftq-fill');
        track.append(fill);
        if (i === 0) this.bars.push(fill);
        chip.append(canvas, h('span', 'craftq-n', `×${s[i * 3 + 1]}`), track);
        if (i > 0) group.append(h('span', 'craftq-next', '›'));
        group.append(chip);
      }
      const cancel = button('craftq-cancel', '✕', () => g.cancel_craft(order));
      cancel.title = 'Cancel this craft and get the materials back';
      cancel.setAttribute('aria-label', 'Cancel this craft');
      group.append(cancel);
      return group;
    });
    this.el.replaceChildren(...groups);
  }
}