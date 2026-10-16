// Map pins and the drawing both maps share. A pin is the player's own note on a world column (what,
// plus a few words), kept with the world's record in the browser (save/session.ts), never in the save:
// like prospected deposits, pins are the player's memory, not the world. Pin kinds are the ores of the
// engine's ore guide (`ore_guide`, so names and colours stay the engine's) plus Home and Note.
// `drawMark` / `drawPin` draw the engine's map marks and pins on a canvas for the minimap and the map.
// To add a pin kind: a row in `EXTRA_KINDS`.

import type { Game } from '../wasm/engine.js';

export interface Pin {
  x: number;
  z: number;
  /** A `PinKind.id`. */
  kind: string;
  note: string;
}

export interface PinKind {
  id: string;
  label: string;
  color: string;
}

/** Most pins kept per world. */
const MAX_PINS = 200;
/** Engine mark shapes (`minimap_marks`): a prospected deposit, a machine, ore seen at the surface, a site; the AI survey's guesses are `MARK_GUESS` plus 0 to 2 for how strong. */
export const MARK_DEPOSIT = 0;
export const MARK_MACHINE = 1;
export const MARK_ORE = 2;
export const MARK_SITE = 3;
export const MARK_GUESS = 4;
/** A satellite's footprint is `MARK_COVER` plus its kind; the colour field holds its reach in blocks. */
export const MARK_COVER = 8;
/** A long-distance pole's reach, shown while one is held; the colour field holds the reach in blocks. */
export const MARK_RANGE = 12;
/** A pod launcher or landing pad is `MARK_CHANNEL` plus its channel (0 north, 1 east, 2 south, 3 west); a launch site
 * is `MARK_LAUNCH`. */
export const MARK_CHANNEL = 16;
export const MARK_LAUNCH = 20;
/** Map direction of each channel (x right, z down: north is up). */
const CHANNEL_DIR = [[0, -1], [1, 0], [0, 1], [-1, 0]];
const EXTRA_KINDS: PinKind[] = [
  { id: 'home', label: 'Home', color: '#ffffff' },
  { id: 'note', label: 'Note', color: '#fa9549' },
];

export class Pins {
  readonly kinds: PinKind[];
  private pins: Pin[];
  private readonly listeners: (() => void)[] = [];

  constructor(game: Game, saved: Pin[] | undefined) {
    const ores = game.ore_guide().trim().split('\n').map((line) => line.split('\t'));
    this.kinds = [...ores.map(([id, color, name]) => ({ id: `ore-${id}`, label: name, color: hex(Number(color)) })), ...EXTRA_KINDS];
    this.pins = (saved ?? []).filter((p) => Number.isFinite(p.x) && Number.isFinite(p.z)).slice(0, MAX_PINS);
  }

  get list(): readonly Pin[] {
    return this.pins;
  }

  /** Calls `fn` after every change. */
  subscribe(fn: () => void): void {
    this.listeners.push(fn);
  }

  kind(id: string): PinKind {
    return this.kinds.find((k) => k.id === id) ?? EXTRA_KINDS[1];
  }

  /** What a pin says on the map: its note, else its kind. */
  label(p: Pin): string {
    return p.note || this.kind(p.kind).label;
  }

  add(pin: Pin): void {
    if (this.pins.length >= MAX_PINS) this.pins.shift();
    this.pins.push(pin);
    this.changed();
  }

  update(pin: Pin, kind: string, note: string): void {
    pin.kind = kind;
    pin.note = note;
    this.changed();
  }

  remove(pin: Pin): void {
    this.pins = this.pins.filter((p) => p !== pin);
    this.changed();
  }

  private changed(): void {
    this.listeners.forEach((fn) => fn());
  }
}

/** `0xRRGGBB` as a CSS colour. */
export function hex(color: number): string {
  return `#${color.toString(16).padStart(6, '0')}`;
}

/** One engine mark at canvas point (x, y): a ringed dot, a square, a diamond or a hollow square; `px` is one map pixel,
 * `perBlock` the canvas pixels in one block (for a satellite's footprint, whose size is in blocks). */
export function drawMark(
  c: CanvasRenderingContext2D, x: number, y: number, color: number, shape: number, px: number, perBlock = px,
): void {
  c.fillStyle = hex(color);
  c.beginPath();
  if (shape >= MARK_CHANNEL) {
    drawSpaceMark(c, x, y, shape, px);
    return;
  }
  if (shape === MARK_RANGE) {
    // A dashed ring: where the next long-distance pole can stand and still link to this one.
    c.arc(x, y, color * perBlock, 0, 2 * Math.PI);
    c.setLineDash([6 * px, 4 * px]);
    c.strokeStyle = 'rgba(255, 255, 255, 0.75)';
    c.lineWidth = Math.max(1.5, px);
    c.stroke();
    c.setLineDash([]);
    c.beginPath();
    c.arc(x, y, 2.5 * px, 0, 2 * Math.PI);
    c.fillStyle = '#fff';
    c.fill();
    return;
  }
  if (shape >= MARK_COVER) {
    // A satellite's footprint (the colour field is its reach in blocks): a faint disc and ring, gold for survey,
    // cyan for comms and pink for power.
    const tint = ['#f2c230', '#5ad1e0', '#f472b6', '#9be7ff'][shape - MARK_COVER] ?? '#ffffff';
    c.arc(x, y, color * perBlock, 0, 2 * Math.PI);
    c.globalAlpha = 0.07;
    c.fillStyle = tint;
    c.fill();
    c.globalAlpha = 0.5;
    c.strokeStyle = tint;
    c.lineWidth = Math.max(1.5, px);
    c.stroke();
    c.globalAlpha = 1;
    return;
  }
  if (shape >= MARK_GUESS) {
    // A faint ring, wider over more stained ground: a guess, not a find.
    c.arc(x, y, (4 + 2 * (shape - MARK_GUESS)) * px, 0, 2 * Math.PI);
    c.strokeStyle = hex(color);
    c.lineWidth = Math.max(1.5, px);
    c.globalAlpha = 0.55;
    c.stroke();
    c.globalAlpha = 1;
    return;
  }
  if (shape === MARK_SITE) {
    // A hollow square: the area, not a thing standing in it.
    c.rect(x - 3 * px, y - 3 * px, 6 * px, 6 * px);
    c.strokeStyle = hex(color);
    c.lineWidth = Math.max(2, 1.5 * px);
    c.stroke();
    return;
  }
  if (shape === MARK_DEPOSIT) {
    c.arc(x, y, 3.5 * px, 0, 2 * Math.PI);
    c.strokeStyle = '#fff';
  } else if (shape === MARK_MACHINE) {
    c.rect(x - 2 * px, y - 2 * px, 4 * px, 4 * px);
    c.strokeStyle = 'rgba(0, 0, 0, 0.7)';
  } else {
    const r = 3.5 * px;
    c.moveTo(x, y - r);
    c.lineTo(x + r, y);
    c.lineTo(x, y + r);
    c.lineTo(x - r, y);
    c.closePath();
    c.strokeStyle = '#fff';
  }
  c.fill();
  c.stroke();
}

/** A launch site as a rocket (a tall triangle), or a pod launcher or pad as a square with an arrow out of the side its
 * channel faces, so pads that match a launcher are easy to pick out. The fill colour is already set. */
function drawSpaceMark(c: CanvasRenderingContext2D, x: number, y: number, shape: number, px: number): void {
  c.strokeStyle = 'rgba(0, 0, 0, 0.8)';
  c.lineWidth = Math.max(1, px);
  if (shape === MARK_LAUNCH) {
    c.moveTo(x, y - 5 * px);
    c.lineTo(x + 3 * px, y + 4 * px);
    c.lineTo(x - 3 * px, y + 4 * px);
    c.closePath();
  } else {
    const [dx, dy] = CHANNEL_DIR[(shape - MARK_CHANNEL) % 4];
    const s = 2.5 * px, tip = 6 * px;
    c.rect(x - s, y - s, 2 * s, 2 * s);
    c.moveTo(x + dx * tip, y + dy * tip);
    c.lineTo(x + dx * s - dy * s, y + dy * s + dx * s);
    c.lineTo(x + dx * s + dy * s, y + dy * s - dx * s);
    c.closePath();
  }
  c.fill();
  c.stroke();
}

/** A pin whose point is at (x, y): a round head on a short stem, and its label when given. */
export function drawPin(c: CanvasRenderingContext2D, x: number, y: number, color: string, px: number, label = ''): void {
  const head = 5 * px, stem = 9 * px;
  c.lineWidth = Math.max(1, px);
  c.strokeStyle = 'rgba(0, 0, 0, 0.8)';
  c.beginPath();
  c.moveTo(x, y);
  c.lineTo(x, y - stem);
  c.stroke();
  c.beginPath();
  c.arc(x, y - stem - head, head, 0, 2 * Math.PI);
  c.fillStyle = color;
  c.fill();
  c.stroke();
  if (!label) return;
  c.font = `${Math.round(12 * px)}px 'Segoe UI', system-ui, sans-serif`;
  c.textBaseline = 'middle';
  c.lineWidth = 3 * px;
  c.strokeStyle = 'rgba(0, 0, 0, 0.75)';
  c.strokeText(label, x + head + 3 * px, y - stem - head);
  c.fillStyle = '#fff';
  c.fillText(label, x + head + 3 * px, y - stem - head);
}
