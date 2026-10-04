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
/** Engine mark shapes (`minimap_marks`): a prospected deposit, a machine, ore seen at the surface, a site. */
export const MARK_DEPOSIT = 0;
export const MARK_MACHINE = 1;
export const MARK_ORE = 2;
export const MARK_SITE = 3;
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

/** One engine mark at canvas point (x, y): a ringed dot, a square, a diamond or a hollow square; `px` is one map pixel. */
export function drawMark(c: CanvasRenderingContext2D, x: number, y: number, color: number, shape: number, px: number): void {
  c.fillStyle = hex(color);
  c.beginPath();
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
