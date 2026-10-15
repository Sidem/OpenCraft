// The eras of the research tree: where the nodes sit (`geometry`), the tinted band behind each era with its name,
// pack icons and progress, and the era buttons in the screen's header that fly the camera to a band. An era is the
// group of techs that cost the same science packs (layout: tech-tree.ts); its colour is taken from its newest
// pack's icon, so a new pack colour needs no change here.

import './research-eras.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import { itemIcon } from './research-node';
import type { Rect } from './research-view';
import type { TreeLayout } from './tech-tree';

export const NODE_W = 176;
export const NODE_H = 58;
const COL_GAP = 56;
const ROW_GAP = 16;
const ERA_GAP = 56;
const HEAD_H = 180;
const PAD = 20;

export interface Geometry {
  /** Top-left of tech `t`'s node in world pixels. */
  at: (t: number) => { x: number; y: number };
  /** The band of each era. */
  bands: Rect[];
  width: number;
  height: number;
}

export function geometry(lay: TreeLayout): Geometry {
  const left = (era: number) => PAD + lay.eras[era].first * (NODE_W + COL_GAP) + era * ERA_GAP;
  const at = (t: number) => ({
    x: left(lay.era[t]) + (lay.col[t] - lay.eras[lay.era[t]].first) * (NODE_W + COL_GAP),
    y: HEAD_H + lay.row[t] * (NODE_H + ROW_GAP),
  });
  const height = HEAD_H + lay.rows * (NODE_H + ROW_GAP) - ROW_GAP + PAD;
  const bands = lay.eras.map((e, i) => ({
    x: left(i) - PAD,
    y: 0,
    w: e.cols * (NODE_W + COL_GAP) - COL_GAP + 2 * PAD,
    h: height,
  }));
  const last = bands[bands.length - 1];
  return { at, bands, width: last.x + last.w + PAD, height };
}

export class Eras {
  /** The era buttons, for the screen's header. */
  readonly tabs = h('div', 'rs-eras');
  /** The era's colour as "r, g, b". */
  readonly colour: string[];
  private readonly counts: HTMLElement[][] = [];
  private readonly bars: HTMLElement[] = [];

  constructor(
    game: Game,
    private readonly lay: TreeLayout,
    geo: Geometry,
    stage: HTMLElement,
    icon: (item: number) => HTMLCanvasElement,
    onJump: (band: Rect) => void,
  ) {
    this.colour = lay.eras.map((era) => {
      // The packs of a tech in the era that costs the era's full number of packs (any, for an odd one out).
      const t = era.techs.find((o) => game.tech_packs(o).length === era.packs) ?? era.techs[0];
      const packs = Array.from(game.tech_packs(t));
      const key = packs.length > 0 ? packs[packs.length - 1] : -1;
      return key >= 0 ? average(itemIcon(icon, key, '')) : '160, 170, 190';
    });
    lay.eras.forEach((era, i) => {
      const t = era.techs.find((o) => game.tech_packs(o).length === era.packs) ?? era.techs[0];
      const packs = Array.from(game.tech_packs(t));
      const name = packs.length > 0 ? game.item_name(packs[packs.length - 1]).replace(/ Pack$/, '') : 'Free';
      const band = h('div', 'rs-era');
      const b = geo.bands[i];
      band.style.cssText = `left:${b.x}px;top:0;width:${b.w}px;height:${b.h}px;--bw:${b.w}px;--era:${this.colour[i]}`;
      const head = h('div', 'rs-era-head');
      const icons = h('span', 'rs-era-packs');
      for (const p of packs) icons.append(itemIcon(icon, p, 'rs-pack'));
      const count = h('span', 'rs-era-count');
      const track = h('span', 'rs-era-track');
      const bar = h('i', '');
      track.append(bar);
      head.append(h('span', 'rs-era-name', name), icons, count, track);
      band.append(head);
      stage.append(band);
      this.bars.push(bar);

      const tab = button('rs-era-tab', '', () => onJump(b));
      tab.style.setProperty('--era', this.colour[i]);
      tab.title = `Show the ${name} era`;
      const tabCount = h('span', 'rs-era-tab-count');
      tab.append(h('span', 'rs-era-dot'), name, tabCount);
      this.tabs.append(tab);
      this.counts.push([count, tabCount]);
    });
  }

  /** Refreshes each era's "done of total" and bar. */
  update(done: boolean[]): void {
    this.lay.eras.forEach((era, i) => {
      const n = era.techs.filter((t) => done[t]).length;
      this.counts[i][1].textContent = `${n}/${era.techs.length}`;
      this.counts[i][0].textContent = `${n} of ${era.techs.length} done`;
      this.bars[i].style.transform = `scaleX(${n / era.techs.length})`;
      this.tabs.children[i].classList.toggle('complete', n === era.techs.length);
    });
  }
}

/** The mean colour of the icon's saturated pixels as "r, g, b". */
function average(c: HTMLCanvasElement): string {
  const data = c.getContext('2d')!.getImageData(0, 0, c.width, c.height).data;
  let [r, g, b, w] = [0, 0, 0, 0];
  for (let i = 0; i < data.length; i += 4) {
    const [pr, pg, pb] = [data[i], data[i + 1], data[i + 2]];
    const sat = Math.max(pr, pg, pb) - Math.min(pr, pg, pb);
    if (data[i + 3] < 200 || sat < 40) continue;
    r += pr * sat;
    g += pg * sat;
    b += pb * sat;
    w += sat;
  }
  return w > 0 ? `${Math.round(r / w)}, ${Math.round(g / w)}, ${Math.round(b / w)}` : '160, 170, 190';
}
