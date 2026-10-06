// Line chart for the analytics screen (analytics.ts): a canvas with a value axis, a time axis, one line per series
// (gaps where a value is NaN: before the world was opened) and a hover readout of every line at the pointer.
// `lineColour` picks a series colour from an item's icon, so a line looks like what it counts.
// To chart something else: pass `Line`s to `LineChart.set`; nothing here knows what they are.

import { h } from './dom';

export interface Line {
  name: string;
  colour: string;
  values: ArrayLike<number>;
  dashed?: boolean;
}

const PAD = { left: 46, right: 10, top: 10, bottom: 22 };
const GRID_LINES = 4;
const TEXT = '#a3a9b6';
const GRID = 'rgba(255, 255, 255, 0.08)';

/** A short number: 950, 1.2k, 34k, 2.5M. */
export function short(v: number): string {
  if (v >= 1e6) return `${(v / 1e6).toFixed(1)}M`;
  if (v >= 1e4) return `${Math.round(v / 1000)}k`;
  if (v >= 1000) return `${(v / 1000).toFixed(1)}k`;
  return v < 10 ? String(Math.round(v * 10) / 10) : String(Math.round(v));
}

/** "now", "45 s", "5 min", "1.5 h": how long before now. */
export function ago(secs: number): string {
  if (secs < 1) return 'now';
  if (secs < 90) return `${Math.round(secs)} s`;
  if (secs < 5400) return `${Math.round(secs / 60)} min`;
  return `${(secs / 3600).toFixed(1).replace(/\.0$/, '')} h`;
}

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
  private lines: Line[] = [];
  private secs = 60;
  /** Time-axis divisions (each `secs / ticks` long). */
  private ticks = 4;
  private hover = -1;

  constructor(private readonly unit: string) {
    this.el.append(this.canvas, this.tip);
    this.canvas.addEventListener('pointermove', (e) => {
      const n = this.points();
      const rect = this.canvas.getBoundingClientRect();
      const plot = rect.width - PAD.left - PAD.right;
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

  /** Shows `lines` over the last `secs` seconds, with a time label every `secs / ticks`. */
  set(lines: Line[], secs: number, ticks: number): void {
    this.lines = lines;
    this.secs = secs;
    this.ticks = ticks;
    this.draw();
  }

  private points(): number {
    return this.lines.reduce((n, l) => Math.max(n, l.values.length), 0);
  }

  draw(): void {
    const w = this.canvas.clientWidth, hgt = this.canvas.clientHeight;
    if (w === 0 || hgt === 0) return;
    const dpr = window.devicePixelRatio || 1;
    this.canvas.width = Math.round(w * dpr);
    this.canvas.height = Math.round(hgt * dpr);
    const c = this.canvas.getContext('2d')!;
    c.setTransform(dpr, 0, 0, dpr, 0, 0);
    const x0 = PAD.left, x1 = w - PAD.right, y0 = hgt - PAD.bottom, y1 = PAD.top;
    let max = 0;
    for (const l of this.lines) for (let i = 0; i < l.values.length; i++) if (l.values[i] > max) max = l.values[i];
    const top = Math.max(4, niceCeil(max * 1.05)); // at least 4, so an empty chart still has whole-number gridlines
    const n = this.points();
    const xAt = (i: number) => x0 + (n < 2 ? 0 : (i / (n - 1)) * (x1 - x0));
    const yAt = (v: number) => y0 - (v / top) * (y0 - y1);

    c.font = '11px system-ui, sans-serif';
    c.fillStyle = TEXT;
    c.strokeStyle = GRID;
    c.lineWidth = 1;
    c.textAlign = 'right';
    c.textBaseline = 'middle';
    for (let k = 0; k <= GRID_LINES; k++) {
      const y = Math.round(yAt((top * k) / GRID_LINES)) + 0.5;
      c.beginPath();
      c.moveTo(x0, y);
      c.lineTo(x1, y);
      c.stroke();
      c.fillText(short((top * k) / GRID_LINES), x0 - 6, y);
    }
    c.textBaseline = 'top';
    for (let k = 0; k <= this.ticks; k++) {
      c.textAlign = k === 0 ? 'left' : k === this.ticks ? 'right' : 'center';
      c.fillText(ago(this.secs * (1 - k / this.ticks)), x0 + (k / this.ticks) * (x1 - x0), y0 + 6);
    }

    c.lineWidth = 2;
    c.lineJoin = 'round';
    for (const l of this.lines) {
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
        if (pen) c.lineTo(xAt(i), yAt(v));
        else c.moveTo(xAt(i), yAt(v));
        pen = true;
      }
      c.stroke();
    }
    c.setLineDash([]);
    this.drawHover(c, xAt, yAt, y0, y1);
  }

  private drawHover(
    c: CanvasRenderingContext2D,
    xAt: (i: number) => number,
    yAt: (v: number) => number,
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
    this.tip.append(h('div', 'an-tip-time', ago(this.secs * (1 - i / Math.max(1, n - 1)))));
    for (const l of this.lines) {
      const v = l.values[i];
      if (v === undefined || Number.isNaN(v)) continue;
      c.fillStyle = l.colour;
      c.beginPath();
      c.arc(x, yAt(v), 3.5, 0, Math.PI * 2);
      c.fill();
      const row = h('div', 'an-tip-row');
      const dot = h('span', 'an-dot');
      dot.style.background = l.colour;
      row.append(dot, h('span', '', `${l.name}: ${short(v)} ${this.unit}`));
      this.tip.append(row);
    }
    this.tip.classList.remove('hidden');
    // Beside the pointer, on whichever side has room.
    const flip = x > this.canvas.clientWidth / 2;
    this.tip.style.left = flip ? '' : `${x + 12}px`;
    this.tip.style.right = flip ? `${this.canvas.clientWidth - x + 12}px` : '';
  }
}

/** A line colour that looks like the icon: its average colour, made bright and saturated enough to read on the dark
 * panel, and nudged lighter or darker if another series already uses nearly that colour. */
export function lineColour(icon: HTMLCanvasElement, taken: string[]): string {
  let [r, g, b, n] = [0, 0, 0, 0];
  const data = icon.getContext('2d')?.getImageData(0, 0, icon.width, icon.height).data;
  for (let i = 0; data && i < data.length; i += 4) {
    if (data[i + 3] < 128) continue;
    r += data[i];
    g += data[i + 1];
    b += data[i + 2];
    n++;
  }
  const [hue, sat, light] = rgbToHsl(n ? r / n : 200, n ? g / n : 200, n ? b / n : 200);
  let l = Math.min(0.7, Math.max(0.52, light));
  const s = sat < 0.12 ? sat : Math.max(0.45, sat); // greys stay grey
  let colour = hsl(hue, s, l);
  for (let tries = 0; tries < 4 && taken.some((t) => near(t, colour)); tries++) {
    l = tries % 2 === 0 ? Math.max(0.3, l - 0.18 * (tries + 1)) : Math.min(0.9, l + 0.3 * tries);
    colour = hsl(hue, s, l);
  }
  return colour;
}

const near = (a: string, b: string) => {
  const rgb = (s: string) => (s.match(/\d+/g) ?? []).map(Number);
  const [p, q] = [rgb(a), rgb(b)];
  return Math.hypot(p[0] - q[0], p[1] - q[1], p[2] - q[2]) < 42;
};

function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  [r, g, b] = [r / 255, g / 255, b / 255];
  const max = Math.max(r, g, b), min = Math.min(r, g, b), d = max - min;
  const l = (max + min) / 2;
  if (d === 0) return [0, 0, l];
  const s = d / (1 - Math.abs(2 * l - 1));
  const hue = max === r ? ((g - b) / d) % 6 : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return [(hue * 60 + 360) % 360, s, l];
}

/** An `rgb()` string (so `near` can read it back). */
function hsl(hue: number, s: number, l: number): string {
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => {
    const k = (n + hue / 30) % 12;
    return Math.round(255 * (l - a * Math.max(-1, Math.min(k - 3, 9 - k, 1))));
  };
  return `rgb(${f(0)}, ${f(8)}, ${f(4)})`;
}
