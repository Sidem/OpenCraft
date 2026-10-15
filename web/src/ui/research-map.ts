// The research tree's minimap: the whole tree as a small ribbon (era bands, one tick per tech coloured by state)
// with the part on screen outlined. Click or drag on it to move the camera there. `update` redraws when the tree's
// states or the camera changed since the last call.

import './research-map.css';
import { h } from './dom';
import { NODE_H, NODE_W, type Geometry } from './research-eras';
import type { PanZoom } from './research-view';

/** Colour of a tech's tick by state. */
const COLOURS: Record<string, string> = {
  done: '#8fdc9c',
  available: '#7dd3fc',
  chosen: '#fa9549',
  locked: 'rgba(255, 255, 255, 0.28)',
};
const WIDTH = 360;

export class TreeMap {
  readonly el = h('canvas', 'rs-map') as HTMLCanvasElement;
  private readonly scale: number;
  private dirty = true;
  private states: string[] = [];

  constructor(
    private readonly view: PanZoom,
    private readonly geo: Geometry,
    private readonly colours: string[],
    private readonly count: number,
  ) {
    this.scale = WIDTH / geo.width;
    this.el.width = WIDTH;
    this.el.height = Math.max(40, Math.ceil(geo.height * this.scale));
    const go = (e: PointerEvent) => {
      const r = this.el.getBoundingClientRect();
      const [wx, wy] = [((e.clientX - r.left) / r.width) * geo.width, ((e.clientY - r.top) / r.height) * this.el.height / this.scale];
      this.view.centreOn(wx, wy, false);
    };
    this.el.addEventListener('pointerdown', (e) => {
      this.el.setPointerCapture(e.pointerId);
      go(e);
    });
    this.el.addEventListener('pointermove', (e) => {
      if (this.el.hasPointerCapture(e.pointerId)) go(e);
    });
  }

  /** `states[t]` is `done`, `available`, `chosen` or `locked`; pass nothing when only the camera moved. */
  update(states?: string[]): void {
    if (states) {
      this.states = states;
      this.dirty = true;
    }
    if (this.dirty) this.draw();
  }

  /** The camera moved: redraw at the next update. */
  moved(): void {
    this.dirty = true;
  }

  private draw(): void {
    this.dirty = false;
    const ctx = this.el.getContext('2d')!;
    const s = this.scale;
    ctx.clearRect(0, 0, this.el.width, this.el.height);
    this.geo.bands.forEach((b, i) => {
      ctx.fillStyle = `rgba(${this.colours[i]}, 0.2)`;
      ctx.fillRect(b.x * s, 0, b.w * s, this.el.height);
    });
    for (let t = 0; t < this.count; t++) {
      const p = this.geo.at(t);
      ctx.fillStyle = COLOURS[this.states[t] ?? 'locked'];
      ctx.fillRect(p.x * s, p.y * s, Math.max(3, NODE_W * s), Math.max(2, NODE_H * s));
    }
    const v = this.view.visible();
    ctx.strokeStyle = '#fff';
    ctx.lineWidth = 1.5;
    ctx.strokeRect(v.x * s, v.y * s, v.w * s, v.h * s);
  }
}
