// Prospecting panel: the scanner's list of nearby deposits and the core drill's exact figures, from the
// engine's latest reading (`prospect_*`; the record layout is in crates/engine/src/prospect.rs). Shown
// while a scanner or core drill is in hand. A scan's distances, arrows and depths follow the player live
// from where it was taken. To show another figure: append it to the engine's record and read it below.

import './prospect.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

/** `held_device` / `prospect_kind` of the scanner (the core drill is 2). */
const SCAN = 1;
/** Rows listed: the nearest lodes and veins, then the nearest outcrops; the rest are only counted. */
const MAX_DEEP = 7;
const MAX_OUTCROPS = 4;
const OUTCROP_TIER = 2;
const SIZES = ['small', 'medium', 'large'];
const COMPASS = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW'];

/** A listed deposit, redrawn every frame relative to the player. */
interface Mark {
  x: number;
  y: number;
  z: number;
  arrow: HTMLElement;
  where: HTMLElement;
}

export class ProspectPanel {
  /** Called when a new reading arrives (the host plays the sweep). */
  onReading: (() => void) | null = null;
  private readonly el = h('div', 'prospect hidden');
  private readonly title = h('div', 'prospect-title');
  private readonly list = h('div', 'prospect-list');
  private marks: Mark[] = [];
  private seq: number;
  /** What the panel shows now: `${device}:${seq}`. */
  private shown = '';

  constructor(private readonly game: Game) {
    this.el.append(this.title, this.list);
    document.getElementById('hud')!.append(this.el);
    this.seq = game.prospect_seq();
  }

  /** Call every frame. */
  update(): void {
    const g = this.game;
    const seq = g.prospect_seq();
    if (seq !== this.seq) {
      this.seq = seq;
      this.onReading?.();
    }
    const device = g.held_device();
    this.el.classList.toggle('hidden', device === 0);
    if (device === 0) return;
    const key = `${device}:${seq}`;
    if (key !== this.shown) {
      this.shown = key;
      if (g.prospect_kind() !== device) this.showHowTo(device);
      else if (device === SCAN) this.showScan();
      else this.showDrill();
    }
    this.follow();
  }

  private showHowTo(device: number): void {
    this.marks = [];
    this.title.textContent = device === SCAN ? 'Scanner' : 'Core drill';
    const how =
      device === SCAN
        ? `Right-click to list the ore deposits within ${this.game.scan_range()} blocks.`
        : 'Hold right-click on the ground for 3 seconds to see exactly what lies beneath it.';
    this.list.replaceChildren(h('p', 'prospect-note', how));
  }

  private showScan(): void {
    const g = this.game;
    const [ox, oy, oz] = g.prospect_origin();
    const r = g.prospect_records();
    const n = g.prospect_fields();
    this.title.textContent = `Scan · ${g.scan_range()} blocks around`;
    this.marks = [];
    const rows: HTMLElement[] = [];
    let deep = 0, outcrops = 0;
    for (let i = 0; i < r.length; i += n) {
      const [ore, tier, dx, dz, depth, band] = r.subarray(i, i + n);
      if (tier === OUTCROP_TIER ? ++outcrops > MAX_OUTCROPS : ++deep > MAX_DEEP) continue;
      const arrow = h('span', 'prospect-arrow', '↑');
      const where = h('span', 'prospect-where');
      const row = h('div', 'prospect-row');
      row.append(arrow, h('span', 'prospect-name', g.deposit_label(ore, tier)), where, h('span', 'prospect-size', SIZES[band]));
      rows.push(row);
      this.marks.push({ x: ox + dx + 0.5, y: oy - depth, z: oz + dz + 0.5, arrow, where });
    }
    const more = [
      deep > MAX_DEEP ? `${deep - MAX_DEEP} veins or lodes` : '',
      outcrops > MAX_OUTCROPS ? `${outcrops - MAX_OUTCROPS} outcrops` : '',
    ].filter(Boolean);
    if (more.length) rows.push(h('p', 'prospect-note', `and ${more.join(' and ')} further off`));
    if (rows.length === 0) rows.push(h('p', 'prospect-note', 'No ore anywhere near. Try further on.'));
    this.list.replaceChildren(...rows);
  }

  private showDrill(): void {
    const g = this.game;
    const [x, y, z] = g.prospect_origin();
    const r = g.prospect_records();
    const n = g.prospect_fields();
    this.title.textContent = `Core sample · x ${x}, z ${z}, from y ${y} down`;
    this.marks = [];
    const rows: HTMLElement[] = [];
    for (let i = 0; i < r.length; i += n) {
      const [ore, tier, left, total, units, top, bottom] = r.subarray(i, i + n);
      const row = h('div', 'prospect-row prospect-sample');
      const figures = `${left.toLocaleString()} of ${total.toLocaleString()} blocks · ${units.toLocaleString()} units`;
      const depth = top === bottom ? `y ${top}` : `y ${bottom} to ${top}`;
      row.append(
        h('span', 'prospect-name', g.deposit_label(ore, tier)),
        h('span', 'prospect-where', depth),
        h('span', 'prospect-figures', left === 0 ? 'worked out' : figures),
      );
      rows.push(row);
    }
    if (rows.length === 0) rows.push(h('p', 'prospect-note', 'No ore beneath this spot.'));
    this.list.replaceChildren(...rows);
  }

  /** Points each listed deposit's arrow and distance from where the player stands and faces now. */
  private follow(): void {
    if (this.marks.length === 0) return;
    const g = this.game;
    const px = g.player_x(), py = g.player_y(), pz = g.player_z(), yaw = g.yaw();
    for (const m of this.marks) {
      const dx = m.x - px, dz = m.z - pz;
      // North is -z; bearings turn clockwise, like yaw.
      const bearing = Math.atan2(dx, -dz);
      m.arrow.style.transform = `rotate(${bearing - yaw}rad)`;
      const compass = COMPASS[((Math.round(bearing / (Math.PI / 4)) % 8) + 8) % 8];
      const depth = Math.round(py - m.y);
      const vertical = depth >= 0 ? `${depth} down` : `${-depth} up`;
      const text = `${Math.round(Math.hypot(dx, dz))} m ${compass} · ${vertical}`;
      if (m.where.textContent !== text) m.where.textContent = text;
    }
  }
}
