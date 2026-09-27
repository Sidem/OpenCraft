// The ore guide beside the world map: a depth chart (how far below the ground each ore's buried
// pockets and veins lie), then per ore where it is common and how to spot it, then general advice. All
// numbers and words come from the engine (`ore_guide`, `ore_guide_notes`), which reads them from the
// world's own generator, so the guide is true for old worlds too. Built once: a world keeps its
// generator.

import './ore-guide.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import { hex } from './pins';

/** Depth chart ticks, in blocks. */
const TICK = 10;

export class OreGuide {
  readonly el = h('section', 'og');

  constructor(game: Game) {
    const rows = game.ore_guide().trim().split('\n').map((line) => line.split('\t'));
    const deepest = Math.ceil(Math.max(...rows.map((r) => Number(r[4]))) / TICK) * TICK;
    const pct = (depth: number) => `${(depth / deepest) * 100}%`;

    const chart = h('div', 'og-chart');
    const axis = h('div', 'og-axis');
    for (let d = 0; d <= deepest; d += TICK) {
      const tick = h('span', 'og-tick', `${d}`);
      tick.style.left = pct(d);
      axis.append(tick);
    }
    chart.append(h('div', 'og-caption', 'Depth below the ground (blocks)'), axis);
    const details = h('div', 'og-details');
    for (const [, color, name, lo, hi, common, signs] of rows) {
      const swatch = h('span', 'og-swatch');
      swatch.style.background = hex(Number(color));
      const band = h('div', 'og-band');
      band.style.left = pct(Number(lo));
      band.style.width = pct(Number(hi) - Number(lo));
      band.style.background = hex(Number(color));
      band.title = `${name}: ${lo} to ${hi} blocks down`;
      const track = h('div', 'og-track');
      track.append(band);
      const label = h('span', 'og-name');
      label.append(swatch.cloneNode(), name);
      chart.append(label, track, h('span', 'og-range', `${lo}–${hi}`));

      const item = h('div', 'og-ore');
      const title = h('h4', '');
      title.append(swatch, name);
      item.append(title, h('p', '', common), h('p', '', signs));
      details.append(item);
    }
    const notes = h('ul', 'og-notes');
    for (const line of game.ore_guide_notes().split('\n')) notes.append(h('li', '', line));
    notes.append(h('li', '', 'Ore you have seen at the surface is marked on the map with a diamond. Click the map to pin anything else.'));
    this.el.append(h('h3', '', 'Finding ore'), chart, notes, details);
  }
}
