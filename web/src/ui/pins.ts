// Map pins and the drawing both maps share. A pin is the player's own note on a world column (what,
// plus a few words), kept with the world's record in the browser (save/session.ts), never in the save:
// like prospected deposits, pins are the player's memory, not the world. Pin kinds are the ores of the
// engine's ore guide (`ore_guide`, so names and colours stay the engine's) plus Home and Note.
// `drawMark` / `drawPin` / `arrow` draw the engine's map marks, pins and players on a canvas for the minimap and the
// map. Which pins and marks show is map-filter.ts's call; hovering one is map-info.ts. To add a pin kind: a row in
// `EXTRA_KINDS` (it gets a filter chip by itself).

import type { Game } from '../wasm/engine.js';

export interface Pin {
  x: number;
  /** The ground's top block, when the telescope set it; older pins have none and the engine looks the ground up. */
  y?: number;
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
/** A landing pad is `MARK_CHANNEL` and a pod launcher the next shape, both in their channel's colour; a launch site is
 * `MARK_LAUNCH`. */
export const MARK_CHANNEL = 16;
export const MARK_LAUNCH = 20;
/** A town is `MARK_TOWN` plus its state (`town_states`: 0 supplied, 1 stalled, 2 dormant), in its kind's colour. */
export const MARK_TOWN = 24;
const TOWN_STALLED = 1;
const TOWN_DORMANT = 2;
const EXTRA_KINDS: PinKind[] = [
  { id: 'home', label: 'Home', color: '#ffffff' },
  { id: 'note', label: 'Note', color: '#fa9549' },
];

export class Pins {
  readonly kinds: PinKind[];
  private pins: Pin[];
  /** Most pins kept per world: the engine draws that many as columns (telescope.ts). */
  private readonly limit: number;
  private readonly listeners: (() => void)[] = [];

  constructor(game: Game, saved: Pin[] | undefined) {
    const ores = game.ore_guide().trim().split('\n').map((line) => line.split('\t'));
    this.kinds = [...ores.map(([id, color, name]) => ({ id: `ore-${id}`, label: name, color: hex(Number(color)) })), ...EXTRA_KINDS];
    this.limit = game.scope_pin_limit();
    this.pins = (saved ?? []).filter((p) => Number.isFinite(p.x) && Number.isFinite(p.z)).slice(0, this.limit);
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
    if (this.pins.length >= this.limit) this.pins.shift();
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

/** One engine mark at canvas point (x, y): a ringed dot, a square, a diamond, a hollow square or a house (a town); `px` is one map pixel,
 * `perBlock` the canvas pixels in one block (for a satellite's footprint, whose size is in blocks). */
export function drawMark(
  c: CanvasRenderingContext2D, x: number, y: number, color: number, shape: number, px: number, perBlock = px,
): void {
  c.fillStyle = hex(color);
  c.beginPath();
  if (shape >= MARK_TOWN) {
    drawTown(c, x, y, shape - MARK_TOWN, px);
    return;
  }
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

/** A town as a house in its kind's colour (already set): ringed red while stalled, faded while dormant. */
function drawTown(c: CanvasRenderingContext2D, x: number, y: number, state: number, px: number): void {
  const w = 4 * px, wall = 3 * px, roof = 4 * px;
  c.moveTo(x - w, y + wall);
  c.lineTo(x - w, y - wall / 2);
  c.lineTo(x, y - wall / 2 - roof);
  c.lineTo(x + w, y - wall / 2);
  c.lineTo(x + w, y + wall);
  c.closePath();
  c.globalAlpha = state === TOWN_DORMANT ? 0.45 : 1;
  c.strokeStyle = 'rgba(0, 0, 0, 0.8)';
  c.lineWidth = Math.max(1, px);
  c.fill();
  c.stroke();
  c.globalAlpha = 1;
  if (state !== TOWN_STALLED) return;
  c.beginPath();
  c.arc(x, y - px, 7.5 * px, 0, 2 * Math.PI);
  c.strokeStyle = '#ff4a3d';
  c.lineWidth = Math.max(2, 1.5 * px);
  c.stroke();
}

/** A launch site as a rocket (a tall triangle), a landing pad as a square and a pod launcher as a square with an arrow
 * up out of it. The fill colour (the channel's) is already set, so matching pads and launchers are easy to pick out. */
function drawSpaceMark(c: CanvasRenderingContext2D, x: number, y: number, shape: number, px: number): void {
  c.strokeStyle = 'rgba(0, 0, 0, 0.8)';
  c.lineWidth = Math.max(1, px);
  if (shape === MARK_LAUNCH) {
    c.moveTo(x, y - 5 * px);
    c.lineTo(x + 3 * px, y + 4 * px);
    c.lineTo(x - 3 * px, y + 4 * px);
    c.closePath();
  } else {
    const s = 2.5 * px;
    c.rect(x - s, y - s, 2 * s, 2 * s);
    if (shape > MARK_CHANNEL) {
      c.moveTo(x, y - 6 * px);
      c.lineTo(x + s, y - s);
      c.lineTo(x - s, y - s);
      c.closePath();
    }
  }
  c.fill();
  c.stroke();
}

/** A player arrow at canvas point (x, y), turned by `yaw` (0 faces north, up the map). */
export function arrow(c: CanvasRenderingContext2D, x: number, y: number, yaw: number, dpr: number, color: string): void {
  c.save();
  c.translate(x, y);
  c.rotate(yaw);
  c.beginPath();
  c.moveTo(0, -8 * dpr);
  c.lineTo(6 * dpr, 7 * dpr);
  c.lineTo(0, 3 * dpr);
  c.lineTo(-6 * dpr, 7 * dpr);
  c.closePath();
  c.fillStyle = color;
  c.strokeStyle = 'rgba(0, 0, 0, 0.8)';
  c.fill();
  c.stroke();
  c.restore();
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
