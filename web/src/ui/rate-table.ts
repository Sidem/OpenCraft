// The rate table at the top of a machine panel: what one machine takes in and puts out at full speed, one row per item
// with the amount a minute and a second, so the ratio between two machines reads off the numbers (`machine_rate_rows`:
// `[kind (0 uses, 1 makes), item, tenths of an item a minute]` per row). Redrawn only when the rows change; hidden for a
// machine without them. `machine_rate_notes` adds a few muted lines under it (a miner's deposit).

import './rate-table.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import { formatRate } from './recipe-rates';

const ICON_PX = 64;

export class RateTable {
  readonly el = h('section', 'rt hidden');
  private readonly notes = h('p', 'rt-notes hidden');
  private drawn = '';

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {}

  /** Redraws for the machine at `pos`. Cheap when nothing changed; call every frame. */
  update(pos: [number, number, number]): void {
    const g = this.game;
    const rows = g.machine_rate_rows(...pos);
    const notes = g.machine_rate_notes(...pos);
    const key = `${rows.join(',')}|${notes}`;
    if (key === this.drawn) return;
    this.drawn = key;
    this.el.classList.toggle('hidden', rows.length === 0);
    this.notes.classList.toggle('hidden', !notes);
    this.notes.textContent = notes;
    const head = h('div', 'rt-row rt-head');
    head.append(h('span', 'rt-kind rt-title', 'Full speed'), h('b', '', 'per min'), h('b', '', 'per sec'));
    const lines: HTMLElement[] = [head];
    for (let i = 0; i + 2 < rows.length; i += 3) {
      const [kind, item, tenths] = [rows[i], rows[i + 1], rows[i + 2]];
      const row = h('div', `rt-row ${kind ? 'rt-out' : 'rt-in'}`);
      const canvas = h('canvas', 'rt-icon');
      canvas.width = canvas.height = ICON_PX;
      canvas.getContext('2d')!.drawImage(this.icon(item), 0, 0);
      const name = h('span', 'rt-name');
      name.append(canvas, g.item_name(item));
      row.append(
        h('span', 'rt-kind', kind ? 'Makes' : 'Uses'),
        name,
        h('b', '', formatRate(tenths / 10)),
        h('b', '', formatRate(tenths / 600)),
      );
      lines.push(row);
    }
    this.el.replaceChildren(...lines, this.notes);
  }
}
