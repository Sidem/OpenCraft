// Chart for the analytics hub (analytics.ts and its tabs): a canvas with a value axis, a time axis, optional stacked
// areas, any number of lines (gaps where a value is NaN: before the world was opened), end-of-line labels and a hover
// readout of everything at the pointer. `focus(name)` dims every series but one, so a row in a list can light up its
// line. The chart knows nothing about what it shows: pass `Line`s to `set`.
// To chart something else: build `Line`s (areas stack from the bottom up in the order given) and call `set`.

import { ago, short } from './analytics-format';
import { h } from './dom';

export interface Line {
  name: string;
  colour: string;
  values: ArrayLike<number>;
  dashed?: boolean;
  /** Text for the end label and tooltip when it differs from `name` (for example "Iron plate (used)"). */
  label?: string;
}

export interface Plot {
  lines: Line[];
  /** Stacked from the bottom up. */
  areas?: Line[];
  /** Seconds across the whole chart, and the number of time-axis divisions (labels fall on round times). */
  secs: number;
  ticks: number;
  /** Label each line at its right end (the lines must be few enough to read). */
  labels?: boolean;
}

const PAD = { left: 50, right: 12, top: 10, bottom: 22 };
const LABEL_ROOM = 150;
const GRID_LINES = 4;
const TEXT = '#a3a9b6';
const GRID = 'rgba(255, 255, 255, 0.08)';
const DIM = 0.16;

/** The smallest 1, 2 or 5 times a power of ten that is at least `v`. */
function niceCeil(v: number): number {
  if (!(v > 0)) return 1;
  const base = 10 ** Math.floor(Math.log10(v));
  return [1, 2, 5, 10].map((m) => m * base).find((m) => m >= v) ?? base * 10;
}

export class LineChart {
  readonly el = h('div', 'an-chart');
  private readonly canvas = h('canvas', 'an-canvas');
  private readonly tip = h('div', 'an-tip hidden');
  private plot: Plot = { lines: [], secs: 60, ticks: 4 };
  private focused: string | null = null;
  private hover = -1;

  constructor(
    private readonly unit: string,
    height = 190,
  ) {
    this.canvas.style.height = `${height}px`;
    this.el.append(this.canvas, this.tip);
    this.canvas.addEventListener('pointermove', (e) => {
      const n = this.points();
      const rect = this.canvas.getBoundingClientRect();
      const plot = rect.width - PAD.left - this.right();
      this.hover = n < 2 ? -1 : Math.round(Math.min(1, Math.max(0, (e.clientX - rect.left - PAD.left) / plot)) * (n - 1));
      this.draw();
    });
    this.canvas.addEventListener('pointerleave', () => {
      this.hover = -1;
      this.draw();
    });
  }

  /** Pixels across, for choosing how many points to ask the engine for (0 while hidden). */
  get width(): number {
    return this.canvas.clientWidth;
  }

  set(plot: Plot): void {
    this.plot = plot;
    this.draw();
  }

  /** Lights the series with this name and dims the rest (null: all alike). */
  focus(name: string | null): void {
    if (name === this.focused) return;
    this.focused = name;
    this.draw();
  }

  private right(): number {
    return PAD.right + (this.plot.labels ? LABEL_ROOM : 0);
  }

  private points(): number {
    return [...this.plot.lines, ...(this.plot.areas ?? [])].reduce((n, l) => Math.max(n, l.values.length), 0);
  }

  private alpha(l: Line): number {
    return this.focused === null || this.focused === l.name ? 1 : DIM;
  }

  draw(): void {
    const w = this.canvas.clientWidth, hgt = this.canvas.clientHeight;
    if (w === 0 || hgt === 0) return;
    const dpr = window.devicePixelRatio || 1;
    this.canvas.width = Math.round(w * dpr);
    this.canvas.height = Math.round(hgt * dpr);
    const c = this.canvas.getContext('2d')!;
    c.setTransform(dpr, 0, 0, dpr, 0, 0);
    const { lines, areas = [] } = this.plot;
    const n = this.points();
    const x0 = PAD.left, x1 = w - this.right(), y0 = hgt - PAD.bottom, y1 = PAD.top;

    // Each area's top edge: the sum of it and the areas below.
    const tops: Float64Array[] = [];
    areas.forEach((a, k) => {
      const t = new Float64Array(n);
      for (let i = 0; i < n; i++) t[i] = (k > 0 ? tops[k - 1][i] : 0) + (Number.isNaN(a.values[i]) ? 0 : a.values[i] ?? 0);
      tops.push(t);
    });
    let max = 0, min = 0;
    for (const t of tops.slice(-1)) for (const v of t) max = Math.max(max, v);
    for (const l of lines) {
      for (let i = 0; i < l.values.length; i++) {
        if (l.values[i] > max) max = l.values[i];
        if (l.values[i] < min) min = l.values[i];
      }
    }
    // A signed chart (a balance) runs from -top to top; otherwise from 0 (at least 4, so an empty chart has whole gridlines).
    const top = Math.max(4, niceCeil(Math.max(max, -min) * 1.05));
    const bottom = min < 0 ? -top : 0;
    const xAt = (i: number) => x0 + (n < 2 ? 0 : (i / (n - 1)) * (x1 - x0));
    const yAt = (v: number) => y0 - ((v - bottom) / (top - bottom)) * (y0 - y1);

    c.font = '11px system-ui, sans-serif';
    c.fillStyle = TEXT;
    c.strokeStyle = GRID;
    c.lineWidth = 1;
    c.textAlign = 'right';
    c.textBaseline = 'middle';
    for (let k = 0; k <= GRID_LINES; k++) {
      const v = bottom + ((top - bottom) * k) / GRID_LINES;
      const y = Math.round(yAt(v)) + 0.5;
      c.strokeStyle = v === 0 && bottom < 0 ? 'rgba(255, 255, 255, 0.3)' : GRID;
      c.beginPath();
      c.moveTo(x0, y);
      c.lineTo(x1, y);
      c.stroke();
      c.fillText(short(v), x0 - 6, y);
    }
    c.textBaseline = 'top';
    for (let k = 0; k <= this.plot.ticks; k++) {
      c.textAlign = k === 0 ? 'left' : k === this.plot.ticks ? 'right' : 'center';
      c.fillText(ago(this.plot.secs * (1 - k / this.plot.ticks)), x0 + (k / this.plot.ticks) * (x1 - x0), y0 + 6);
    }

    areas.forEach((a, k) => {
      c.globalAlpha = 0.85 * this.alpha(a);
      c.fillStyle = a.colour;
      c.beginPath();
      for (let i = 0; i < n; i++) c[i ? 'lineTo' : 'moveTo'](xAt(i), yAt(tops[k][i]));
      for (let i = n - 1; i >= 0; i--) c.lineTo(xAt(i), yAt(k > 0 ? tops[k - 1][i] : 0));
      c.closePath();
      c.fill();
    });
    c.globalAlpha = 1;

    c.lineJoin = 'round';
    for (const l of lines) {
      c.globalAlpha = this.alpha(l);
      c.lineWidth = this.focused === l.name ? 3 : 2;
      c.strokeStyle = l.colour;
      c.setLineDash(l.dashed ? [6, 4] : []);
      c.beginPath();
      let pen = false;
      for (let i = 0; i < l.values.length; i++) {
        const v = l.values[i];
        if (Number.isNaN(v)) {
          pen = false;
          continue;
        }
        c[pen ? 'lineTo' : 'moveTo'](xAt(i), yAt(v));
        pen = true;
      }
      c.stroke();
    }
    c.setLineDash([]);
    c.globalAlpha = 1;
    if (this.plot.labels) this.drawLabels(c, x1, yAt, y0, y1);
    this.drawHover(c, xAt, yAt, tops, y0, y1);
  }

  /** Each line's name at its right end, pushed apart so the labels do not overlap. */
  private drawLabels(c: CanvasRenderingContext2D, x1: number, yAt: (v: number) => number, y0: number, y1: number): void {
    const ends = this.plot.lines
      .map((l) => {
        let last = NaN;
        for (let i = l.values.length - 1; i >= 0 && Number.isNaN(last); i--) last = l.values[i];
        return { l, y: yAt(Number.isNaN(last) ? 0 : last) };
      })
      .sort((a, b) => a.y - b.y);
    const gap = 13;
    ends.forEach((e, i) => {
      if (i > 0) e.y = Math.max(e.y, ends[i - 1].y + gap);
    });
    for (let i = ends.length - 1; i >= 0; i--) {
      ends[i].y = Math.min(ends[i].y, y0 - 4, i < ends.length - 1 ? ends[i + 1].y - gap : Infinity);
      ends[i].y = Math.max(ends[i].y, y1 + 4);
    }
    c.textAlign = 'left';
    c.textBaseline = 'middle';
    c.font = '12px system-ui, sans-serif';
    for (const { l, y } of ends) {
      c.globalAlpha = this.alpha(l);
      c.fillStyle = l.colour;
      c.fillText(l.label ?? l.name, x1 + 8, y);
    }
    c.globalAlpha = 1;
  }

  private drawHover(
    c: CanvasRenderingContext2D,
    xAt: (i: number) => number,
    yAt: (v: number) => number,
    tops: Float64Array[],
    y0: number,
    y1: number,
  ): void {
    const i = this.hover;
    if (i < 0) {
      this.tip.classList.add('hidden');
      return;
    }
    const x = xAt(i);
    c.strokeStyle = 'rgba(255, 255, 255, 0.35)';
    c.lineWidth = 1;
    c.beginPath();
    c.moveTo(Math.round(x) + 0.5, y1);
    c.lineTo(Math.round(x) + 0.5, y0);
    c.stroke();
    this.tip.replaceChildren();
    const n = this.points();
    this.tip.append(h('div', 'an-tip-time', ago(this.plot.secs * (1 - i / Math.max(1, n - 1)))));
    const rows: { l: Line; v: number; y: number }[] = [];
    (this.plot.areas ?? []).forEach((a, k) => {
      const v = a.values[i];
      if (v !== undefined && !Number.isNaN(v) && v > 0) rows.push({ l: a, v, y: tops[k][i] });
    });
    for (const l of this.plot.lines) {
      const v = l.values[i];
      if (v !== undefined && !Number.isNaN(v)) rows.push({ l, v, y: v });
    }
    // Biggest first, so the tooltip reads like the stack.
    rows.sort((a, b) => b.v - a.v);
    for (const { l, v, y } of rows) {
      c.fillStyle = l.colour;
      c.beginPath();
      c.arc(x, yAt(y), 3.5, 0, Math.PI * 2);
      c.fill();
      const row = h('div', 'an-tip-row');
      const dot = h('span', 'an-dot');
      dot.style.background = l.colour;
      row.append(dot, h('span', '', `${l.label ?? l.name}: ${short(v)} ${this.unit}`));
      if (this.focused === l.name) row.classList.add('lit');
      this.tip.append(row);
    }
    this.tip.classList.remove('hidden');
    // Beside the pointer, on whichever side has room.
    const flip = x > this.canvas.clientWidth / 2;
    this.tip.style.left = flip ? '' : `${x + 12}px`;
    this.tip.style.right = flip ? `${this.canvas.clientWidth - x + 12}px` : '';
  }
}
