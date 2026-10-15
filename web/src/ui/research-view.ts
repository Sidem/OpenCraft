// The research tree's viewport: a pan and zoom camera over a stage of fixed world size (no scrollbars). Drag the
// background to pan, wheel to zoom at the pointer (Shift-wheel pans sideways), arrow keys pan, + and - zoom.
// `fit`, `focusRect` and `centreOn` glide the camera to a place. The stage is transformed by one CSS matrix; the
// viewport carries the zoom as a level of detail (`lod-near`, `lod-far`, `lod-tiny`, see research.css) and the
// stage a `--inv` variable (the size that keeps a label readable when zoomed out). `onChange` fires after each move.

import { h } from './dom';

const MAX_ZOOM = 1.3;
const MIN_ZOOM = 0.12;
const MARGIN = 40;
const GLIDE_MS = 420;
/** Zoom below which nodes show big names only, and below which only coloured tiles. */
export const FAR_ZOOM = 0.75;
export const TINY_ZOOM = 0.3;
/** The smallest zoom that jumping to an era uses: big names are readable from here. */
const READ_ZOOM = 0.5;

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export class PanZoom {
  readonly el = h('div', 'rs-view');
  /** World to screen: screen = world * k + (x, y). */
  x = 0;
  y = 0;
  k = 1;
  onChange: () => void = () => {};
  private glide = 0;

  constructor(
    readonly stage: HTMLElement,
    private readonly world: { w: number; h: number },
  ) {
    this.el.append(stage);
    this.el.tabIndex = 0;
    let from: { x: number; y: number; px: number; py: number } | null = null;
    this.el.addEventListener('pointerdown', (e) => {
      if ((e.target as HTMLElement).closest('.rs-node, .rs-hud') || e.button > 1) return;
      from = { x: e.clientX, y: e.clientY, px: this.x, py: this.y };
      this.el.setPointerCapture(e.pointerId);
      this.el.classList.add('panning');
    });
    this.el.addEventListener('pointermove', (e) => {
      if (from) this.go(from.px + e.clientX - from.x, from.py + e.clientY - from.y, this.k, false);
    });
    for (const end of ['pointerup', 'pointercancel']) {
      this.el.addEventListener(end, () => {
        from = null;
        this.el.classList.remove('panning');
      });
    }
    this.el.addEventListener('wheel', (e) => this.onWheel(e), { passive: false });
    this.el.addEventListener('keydown', (e) => this.onKey(e));
  }

  get width(): number {
    return this.el.clientWidth;
  }

  get height(): number {
    return this.el.clientHeight;
  }

  /** The part of the world on screen. */
  visible(): Rect {
    return { x: -this.x / this.k, y: -this.y / this.k, w: this.width / this.k, h: this.height / this.k };
  }

  /** Zoom that shows the whole world. */
  fitZoom(): number {
    return Math.min(MAX_ZOOM, this.width / (this.world.w + 2 * MARGIN), this.height / (this.world.h + 2 * MARGIN));
  }

  fit(): void {
    this.focusRect({ x: 0, y: 0, w: this.world.w, h: this.world.h }, true);
  }

  /** Brings `r` into view as large as fits (but not past `MAX_ZOOM`, or `zoom` when given), gliding or at once. */
  focusRect(r: Rect, glide: boolean, zoom?: number): void {
    const k = zoom ?? Math.min(MAX_ZOOM, (this.width - 2 * MARGIN) / r.w, (this.height - 2 * MARGIN) / r.h);
    this.go(this.width / 2 - (r.x + r.w / 2) * k, this.height / 2 - (r.y + r.h / 2) * k, k, glide);
  }

  /** Brings the band `r` (an era) to the left of the view, whole if it fits, at a zoom where names can be read. */
  showBand(r: Rect): void {
    const k = Math.min(MAX_ZOOM, Math.max(READ_ZOOM, Math.min((this.width - 2 * MARGIN) / r.w, (this.height - 2 * MARGIN) / r.h)));
    this.go(MARGIN - (r.x - 10) * k, MARGIN - r.y * k, k, true);
  }

  centreOn(wx: number, wy: number, glide: boolean, zoom = this.k): void {
    this.go(this.width / 2 - wx * zoom, this.height / 2 - wy * zoom, zoom, glide);
  }

  /** Zooms by `factor` keeping the screen point (`px`, `py`) still. */
  zoomAt(px: number, py: number, factor: number, glide = false): void {
    const k = Math.min(MAX_ZOOM, Math.max(this.minZoom(), this.k * factor));
    const f = k / this.k;
    this.go(px - (px - this.x) * f, py - (py - this.y) * f, k, glide);
  }

  zoomBy(factor: number): void {
    this.zoomAt(this.width / 2, this.height / 2, factor, true);
  }

  /** Re-applies the transform, e.g. after the viewport changed size. */
  refresh(): void {
    this.moveTo(this.x, this.y, this.k);
  }

  private minZoom(): number {
    return Math.max(MIN_ZOOM, Math.min(this.fitZoom(), 0.3));
  }

  private go(x: number, y: number, k: number, glide: boolean): void {
    window.clearTimeout(this.glide);
    this.stage.classList.toggle('glide', glide);
    if (glide) this.glide = window.setTimeout(() => this.stage.classList.remove('glide'), GLIDE_MS + 50);
    this.moveTo(x, y, k);
  }

  /** Sets the camera, keeping some of the world on screen. */
  private moveTo(x: number, y: number, k: number): void {
    const [w, h] = [this.world.w * k, this.world.h * k];
    this.k = k;
    this.x = this.keep(x, w, this.width);
    this.y = this.keep(y, h, this.height);
    this.stage.style.transform = `translate(${this.x}px, ${this.y}px) scale(${k})`;
    this.stage.style.setProperty('--inv', String(Math.min(3.2, Math.max(1, 1 / k))));
    this.el.classList.toggle('lod-near', k >= FAR_ZOOM);
    this.el.classList.toggle('lod-far', k < FAR_ZOOM && k >= TINY_ZOOM);
    this.el.classList.toggle('lod-tiny', k < TINY_ZOOM);
    this.onChange();
  }

  /** Offset `at` of a world `size` wide in a view `room` wide: centred when it fits, else not past an edge. */
  private keep(at: number, size: number, room: number): number {
    return size + 2 * MARGIN <= room ? (room - size) / 2 : Math.min(MARGIN, Math.max(room - size - MARGIN, at));
  }

  private onWheel(e: WheelEvent): void {
    e.preventDefault();
    const rect = this.el.getBoundingClientRect();
    if (e.shiftKey || Math.abs(e.deltaX) > Math.abs(e.deltaY)) {
      this.go(this.x - (e.deltaX || e.deltaY), this.y, this.k, false);
    } else {
      const step = e.ctrlKey ? 0.01 : 0.0016;
      this.zoomAt(e.clientX - rect.left, e.clientY - rect.top, Math.exp(-e.deltaY * step));
    }
  }

  private onKey(e: KeyboardEvent): void {
    const step = 160;
    const pan: Record<string, [number, number]> = {
      ArrowLeft: [step, 0],
      ArrowRight: [-step, 0],
      ArrowUp: [0, step],
      ArrowDown: [0, -step],
    };
    if (pan[e.key]) this.go(this.x + pan[e.key][0], this.y + pan[e.key][1], this.k, true);
    else if (e.key === '+' || e.key === '=') this.zoomBy(1.4);
    else if (e.key === '-') this.zoomBy(1 / 1.4);
    else return;
    e.preventDefault();
  }
}
