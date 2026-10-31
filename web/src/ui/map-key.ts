// The world map's key: a folding list beside the filter chips that draws every kind of mark (with `drawMark`, as the
// maps do) next to what it means. The rows are map-key-rows.ts; samples take the first ore's colour from the engine.

import './map-key.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import { KEY_ROWS } from './map-key-rows.ts';
import { drawMark } from './pins';

const SIZE = 22;
const MACHINE_GREY = 0xcccccc;
/** A reach for the footprint samples, in sample pixels. */
const SAMPLE_REACH = 8;

export class MapKey {
  readonly el = h('details', 'mk');

  constructor(game: Game) {
    const ore = Number(game.ore_guide().split('\n')[0]?.split('\t')[1] ?? 0xd0703a);
    this.el.append(h('summary', 'mk-head', 'What the marks mean'));
    const list = h('div', 'mk-list');
    for (const row of KEY_ROWS) {
      const canvas = h('canvas', 'mk-sample');
      canvas.width = canvas.height = SIZE * devicePixelRatio;
      const c = canvas.getContext('2d');
      if (c) {
        c.scale(devicePixelRatio, devicePixelRatio);
        const colour = row.tint === 'ore' ? ore : row.tint === 'reach' ? SAMPLE_REACH : MACHINE_GREY;
        drawMark(c, SIZE / 2, SIZE / 2, colour, row.sample ?? row.shape, 1.6, 1);
      }
      const line = h('div', 'mk-row');
      line.append(canvas, h('span', '', row.label));
      list.append(line);
    }
    this.el.append(list);
  }
}
