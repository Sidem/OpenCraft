// The quarry's part of the machine panel (ui/machine.ts): the box's size and depth choices, pause and
// resume, and how far it got ("Layer 3 of 17 · 120 dug · about 600 left") with the deposits it
// uncovered. Read from the engine (`quarry_panel`, `quarry_found`, at most every `REFRESH_MS`, since
// counting what is left walks the box); buttons queue `set_quarry`, which starts a new box over.
// Styles come from machine.css. A new choice needs no change here: the lists come from the engine.

import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';

const REFRESH_MS = 250;

type Pos = [number, number, number];

export class QuarrySection {
  readonly el = h('section', 'mp-quarry hidden');
  private readonly report = h('p', 'mp-status');
  private readonly found = h('p', 'mp-note');
  private readonly widths: HTMLButtonElement[];
  private readonly depths: HTMLButtonElement[];
  private readonly pause: HTMLButtonElement;
  private pos: Pos = [0, 0, 0];
  private data: Uint32Array = new Uint32Array(0);
  private readAt = 0;
  private drawn = '';

  constructor(private readonly game: Game) {
    this.widths = Array.from(game.quarry_widths(), (w, i) =>
      button('secondary-btn mp-insert', `${w} × ${w}`, () => this.set(i, this.data[1], this.data[2] === 1)),
    );
    this.depths = Array.from({ length: game.quarry_depth_count() }, (_, i) => {
      const text = game.quarry_depth_label(i);
      return button('secondary-btn mp-insert', text[0].toUpperCase() + text.slice(1), () =>
        this.set(this.data[0], i, this.data[2] === 1),
      );
    });
    this.pause = button('secondary-btn', 'Pause', () => this.set(this.data[0], this.data[1], this.data[2] !== 1));
    const size = h('div', 'mp-inserts');
    size.append(...this.widths);
    const depth = h('div', 'mp-inserts');
    depth.append(...this.depths);
    this.el.append(h('h3', '', 'Size'), size, h('h3', '', 'Depth'), depth, this.report, this.found, this.pause);
  }

  get visible(): boolean {
    return !this.el.classList.contains('hidden');
  }

  /** Shows the quarry at `pos`, or hides the section for any other machine. Returns whether it shows. */
  update(pos: Pos): boolean {
    const now = performance.now();
    const moved = pos.some((v, i) => v !== this.pos[i]);
    if (moved || now - this.readAt >= REFRESH_MS) {
      this.pos = [...pos];
      this.data = this.game.quarry_panel(...pos);
      this.readAt = now;
    }
    this.el.classList.toggle('hidden', this.data.length === 0);
    if (this.data.length === 0) return false;
    const key = this.data.join(',');
    if (key !== this.drawn) this.draw(key);
    return true;
  }

  /** The panel shows new choices at once, before the next read. */
  private set(width: number, depth: number, paused: boolean): void {
    this.game.set_quarry(...this.pos, width, depth, paused);
    this.data = Uint32Array.from([width, depth, paused ? 1 : 0, ...this.data.slice(3)]);
    this.readAt = 0;
  }

  private draw(key: string): void {
    this.drawn = key;
    const [width, depth, paused, layer, layers, dug, left] = this.data;
    this.widths.forEach((b, i) => b.classList.toggle('active', i === width));
    this.depths.forEach((b, i) => b.classList.toggle('active', i === depth));
    this.pause.textContent = paused ? 'Resume' : 'Pause';
    this.report.textContent =
      layers === 0
        ? 'Nothing to dig at this depth from here.'
        : `Layer ${layer} of ${layers} · ${dug} dug · about ${left} left`;
    this.found.textContent = this.game.quarry_found(...this.pos);
  }
}
