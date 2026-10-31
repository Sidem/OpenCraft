// The power tab's grid picker (analytics-power.ts): one button for all grids and one per grid with a machine on it,
// in the grid's colour (the collar its poles wear), with its machines and its use against what it could make. Shown
// only when there are two grids or more. A grid is known by where its oldest pole stands (`analytics_power_grids`),
// so the pick survives a relink; when that grid is gone the picker falls back to all grids.

import './analytics-grids.css';
import type { Game } from '../wasm/engine.js';
import { kw } from './analytics-format';
import { button, h } from './dom';

export class GridPicker {
  readonly el = h('div', 'an-grids hidden');
  /** The picked grid's key ("x,y,z"), or '' for all grids. */
  private picked = '';
  private drawn = '';

  constructor(private readonly changed: () => void) {}

  /** Rebuilds the buttons and returns the picked grid's index for the `analytics_power_*` calls (-1: all grids). */
  refresh(g: Game): number {
    const rows = g.analytics_power_grids();
    const fields = g.analytics_grid_fields();
    const names = g.analytics_power_grid_names().split('\n');
    const grids: { key: string; label: string; colour: string; title: string }[] = [];
    for (let i = 0; i + fields <= rows.length; i += fields) {
      const [x, y, z, rgb, poles, machines, could, , wanted] = rows.subarray(i, i + fields);
      const name = names[grids.length] ?? '';
      grids.push({
        key: `${x},${y},${z}`,
        label: `${name[0]?.toUpperCase() ?? ''}${name.slice(1)} grid`,
        colour: `#${rgb.toString(16).padStart(6, '0')}`,
        title: `${machines} machines, ${poles} poles · wants ${kw(wanted)} of the ${kw(could)} it could make`,
      });
    }
    let index = grids.findIndex((gr) => gr.key === this.picked);
    if (index < 0) this.picked = '';
    this.el.classList.toggle('hidden', grids.length < 2);
    const look = JSON.stringify([this.picked, grids]);
    if (look !== this.drawn) {
      this.drawn = look;
      const all = button('an-grid', 'All grids', () => this.pick(''));
      all.setAttribute('aria-pressed', String(this.picked === ''));
      const buttons = grids.map((gr) => {
        const b = button('an-grid', '', () => this.pick(gr.key));
        const dot = h('i', 'an-grid-dot');
        dot.style.background = gr.colour;
        b.append(dot, gr.label);
        b.title = gr.title;
        b.setAttribute('aria-pressed', String(gr.key === this.picked));
        return b;
      });
      this.el.replaceChildren(all, ...buttons);
    }
    index = grids.findIndex((gr) => gr.key === this.picked);
    return index;
  }

  /** The picked grid's name for headings, or '' for all grids. */
  label(): string {
    const on = this.el.querySelector<HTMLElement>('.an-grid[aria-pressed="true"]');
    return this.picked === '' ? '' : (on?.textContent ?? '');
  }

  private pick(key: string): void {
    this.picked = key;
    this.changed();
  }
}
