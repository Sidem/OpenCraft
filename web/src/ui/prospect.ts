// Prospecting panel: the scanner's list of nearby deposits and the core drill's exact figures, from the
// engine's latest reading (`prospect_*`; the record layout is in crates/engine/src/prospect.rs). Shown
// while a scanner or core drill is in hand. A scan's distances, arrows and depths follow the player live
// from where it was taken. An advanced scanner (`scanner_advanced`) also lists each deposit's reserve and
// time to work out, shows its ore filter (the engine filters the records; R steps it), and leaves a pointer
// to the nearest match on screen after it is put away (U steps it to the next deposit, nearest first; the pointer
// says "Nearest" or "Chosen" and the list marks its row). To show another figure: append it to the engine's
// record and read it below.

import './prospect.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';

/** `held_device` / `prospect_kind` of the scanner (the core drill is 2). */
const SCAN = 1;
/** Rows listed: the nearest lodes and veins, then the nearest outcrops; the rest are only counted. */
const MAX_DEEP = 7;
const MAX_OUTCROPS = 4;
/** The same for an advanced scanner. */
const MAX_DEEP_ADVANCED = 8;
const MAX_OUTCROPS_ADVANCED = 4;
const OUTCROP_TIER = 2;
/** Within this many blocks (horizontally) of the tracked deposit the pointer says it is below you. */
const ARRIVED = 4;
const SIZES = ['small', 'medium', 'large'];
const COMPASS = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW'];
const COMPASS_LONG = ['north', 'north-east', 'east', 'south-east', 'south', 'south-west', 'west', 'north-west'];
/** Within this many blocks of far ground (a bearing, not a deposit) the pointer says to scan again. */
const ARRIVED_FAR = 48;

/** A listed deposit, redrawn every frame relative to the player. */
interface Mark {
  x: number;
  y: number;
  z: number;
  arrow: HTMLElement;
  where: HTMLElement;
}

/** The deposit an advanced scanner's pointer leads to, or (`y` null) far ground that can hold the filtered ore. */
interface Tracked {
  name: string;
  x: number;
  y: number | null;
  z: number;
}

/** About `units` ore units, in a few figures: "2,200", "134k", "2.9M". */
function units(n: number): string {
  if (n >= 1e6) return `${(n / 1e6).toFixed(1)}M`;
  if (n >= 1000) return `${Math.round(n / 1000)}k`;
  return Math.round(n / 10) * 10 === 0 ? '<10' : `${Math.round(n / 10) * 10}`;
}

/** How long a full-speed mine takes to work a deposit out: "40 min", "9 h", "2 days". */
function duration(minutes: number): string {
  if (minutes < 90) return `${Math.max(1, minutes)} min`;
  if (minutes < 48 * 60) return `${Math.round(minutes / 60)} h`;
  return `${Math.round(minutes / 1440)} days`;
}

export class ProspectPanel {
  /** Called when a new reading arrives (the host plays the sweep). */
  onReading: (() => void) | null = null;
  private readonly el = h('div', 'prospect hidden');
  private readonly title = h('div', 'prospect-title');
  private readonly list = h('div', 'prospect-list');
  /** The pointer to the nearest match of an advanced scan, shown whatever is in hand. */
  private readonly track = h('div', 'prospect-track hidden');
  private readonly trackArrow = h('span', 'prospect-arrow', '↑');
  private readonly trackText = h('span', 'prospect-track-text');
  private tracked: Tracked | null = null;
  /** The latest advanced scan's veins and lodes, then outcrops, nearest first (what U steps through). */
  private candidates: Tracked[] = [];
  /** The one the player chose, kept across scans while it is still found; null for "nearest". */
  private pinned: Tracked | null = null;
  private chosen: Tracked | null = null;
  private marks: Mark[] = [];
  private seq: number;
  /** What the panel shows now: `${device}:${seq}:${filter}`. */
  private shown = '';

  constructor(private readonly game: Game) {
    this.el.append(this.title, this.list);
    this.track.append(this.trackArrow, this.trackText);
    document.getElementById('hud-left')!.append(this.el);
    document.getElementById('hud-top')!.append(this.track);
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
    if (device !== 0) {
      const key = `${device}:${seq}:${g.scan_filter()}`;
      if (key !== this.shown) {
        this.shown = key;
        if (g.prospect_kind() !== device) this.showHowTo(device);
        else if (device === SCAN) this.showScan();
        else this.showDrill();
      }
      this.follow();
    }
    this.followTracked();
  }

  private showHowTo(device: number): void {
    this.marks = [];
    const advanced = device === SCAN && this.game.scanner_advanced();
    this.title.textContent = device === SCAN ? (advanced ? 'Scanner Mk2' : 'Scanner') : 'Core drill';
    const how =
      device === SCAN
        ? `Right-click to list the ore deposits within ${this.game.scan_range()} blocks.` +
          (advanced ? ' Press R to show one ore only.' : '')
        : 'Hold right-click on the ground for 3 seconds to see exactly what lies beneath it.';
    this.list.replaceChildren(h('p', 'prospect-note', how));
  }

  private showScan(): void {
    const g = this.game;
    const [ox, oy, oz] = g.prospect_origin();
    const r = g.prospect_records();
    const n = g.prospect_fields();
    const advanced = g.scanner_advanced();
    const filter = g.scan_filter();
    const [maxDeep, maxOutcrops] = advanced ? [MAX_DEEP_ADVANCED, MAX_OUTCROPS_ADVANCED] : [MAX_DEEP, MAX_OUTCROPS];
    this.title.textContent =
      `Scan · ${g.scan_range()} blocks around` + (advanced ? ` · ${filter ? `${g.ore_name(filter)} only` : 'all ores'}` : '');
    this.marks = [];
    const rows: HTMLElement[] = [];
    let deep = 0, outcrops = 0;
    const found: { dist: number; deposit: Tracked }[] = [];
    const rowOf = new Map<Tracked, HTMLElement>();
    for (let i = 0; i < r.length; i += n) {
      const [ore, tier, dx, dz, depth, band, ore_units, minutes] = r.subarray(i, i + n);
      const deposit = { name: g.deposit_label(ore, tier), x: ox + dx + 0.5, y: oy - depth, z: oz + dz + 0.5 };
      // The pointer goes to the nearest vein or lode (an outcrop only when there is none), unless one is chosen.
      const dist = Math.hypot(dx, dz) + (tier === OUTCROP_TIER ? 1e4 : 0);
      if (advanced) found.push({ dist, deposit });
      if (tier === OUTCROP_TIER ? ++outcrops > maxOutcrops : ++deep > maxDeep) continue;
      const arrow = h('span', 'prospect-arrow', '↑');
      const where = h('span', 'prospect-where');
      const row = h('div', 'prospect-row');
      const size = advanced ? `${units(ore_units)} · ${duration(minutes)}` : SIZES[band];
      row.append(arrow, h('span', 'prospect-name', deposit.name), where, h('span', 'prospect-size', size));
      rows.push(row);
      rowOf.set(deposit, row);
      this.marks.push({ x: deposit.x, y: deposit.y, z: deposit.z, arrow, where });
    }
    if (advanced) {
      this.candidates = found.sort((a, b) => a.dist - b.dist).map((f) => f.deposit);
      this.aim();
      if (this.tracked) rowOf.get(this.tracked)?.classList.add('pointed');
    }
    const far = advanced && filter !== 0 && this.candidates.length === 0 ? this.farGround(filter, ox, oz) : null;
    const more = [
      deep > maxDeep ? `${deep - maxDeep} veins or lodes` : '',
      outcrops > maxOutcrops ? `${outcrops - maxOutcrops} outcrops` : '',
    ].filter(Boolean);
    if (more.length) rows.push(h('p', 'prospect-note', `and ${more.join(' and ')} further off`));
    if (rows.length === 0) rows.push(h('p', 'prospect-note', far ?? 'No ore anywhere near. Try further on.'));
    if (advanced) {
      const note =
        'Right column: ore units left, and the time a full-speed mine takes to work it out. R: next ore. ' +
        'The marked row is the pointer\'s target (the nearest one unless you press U to pick the next).';
      rows.push(h('p', 'prospect-note', note));
    }
    this.el.classList.toggle('prospect-wide', advanced);
    this.list.replaceChildren(...rows);
  }

  /** Sets the pointer's target: the chosen deposit if the latest scan still has it, else the nearest. */
  private aim(): void {
    const p = this.pinned;
    const same = p && this.candidates.find((c) => c.name === p.name && Math.hypot(c.x - p.x, c.z - p.z) < 3);
    this.chosen = same ?? null;
    this.tracked = same ?? this.candidates[0] ?? null;
  }

  /** U: points at the next deposit of the scan, nearest first; after the last it is back to the nearest. */
  nextTarget(): void {
    if (this.candidates.length < 2 || !this.tracked) return;
    const next = (this.candidates.indexOf(this.tracked) + 1) % this.candidates.length;
    this.pinned = next === 0 ? null : this.candidates[next];
    this.shown = '';
    this.aim();
    this.update();
  }

  /**
   * With one ore filtered and none of it in range: the way to the nearest ground that can hold it (the engine reads
   * the world's biome map, so this works for unexplored ground). Aims the pointer there and returns the note.
   */
  private farGround(ore: number, ox: number, oz: number): string {
    const g = this.game;
    const name = g.ore_name(ore).toLowerCase();
    const [x, z, dist] = g.ore_bearing(ore);
    if (x === undefined) return `No ${name} anywhere near, and none known within about 3,000 blocks.`;
    this.tracked = { name: `${g.ore_name(ore)} ground`, x: x + 0.5, y: null, z: z + 0.5 };
    if (dist === 0) return `This ground can hold ${name}, but none lies within ${g.scan_range()} blocks. Move on a little and scan again.`;
    const lo = Math.floor(dist / 200) * 200;
    const bearing = Math.atan2(x - ox, -(z - oz));
    const way = COMPASS_LONG[((Math.round(bearing / (Math.PI / 4)) % 8) + 8) % 8];
    return (
      `No ${name} within ${g.scan_range()} blocks. The nearest ground that holds it lies ${way}, ` +
      `${lo} to ${lo + 200} blocks away: follow the pointer, and scan again on the way.`
    );
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

  /** Points the on-screen pointer at the tracked deposit, from where the player stands and faces now. */
  private followTracked(): void {
    const t = this.tracked;
    this.track.classList.toggle('hidden', !t);
    if (!t) return;
    const g = this.game;
    const dx = t.x - g.player_x(), dz = t.z - g.player_z();
    const dist = Math.hypot(dx, dz);
    const near = dist < (t.y === null ? ARRIVED_FAR : ARRIVED);
    const bearing = Math.atan2(dx, -dz);
    this.trackArrow.style.visibility = near ? 'hidden' : 'visible';
    this.trackArrow.style.transform = `rotate(${bearing - g.yaw()}rad)`;
    const compass = COMPASS[((Math.round(bearing / (Math.PI / 4)) % 8) + 8) % 8];
    let text: string;
    if (t.y === null) {
      text = near ? `${t.name} · you are there: scan again` : `${t.name} · about ${Math.round(dist / 10) * 10} m ${compass}`;
    } else {
      const depth = Math.round(g.player_y() - t.y);
      const vertical = depth >= 0 ? `${depth} down` : `${-depth} up`;
      text = near ? `${t.name} · right below you, ${vertical}` : `${t.name} · ${Math.round(dist)} m ${compass} · ${vertical}`;
    }
    // Say how the target was picked, and that U picks another.
    const mode = t.y === null ? '' : this.chosen ? 'Chosen' : 'Nearest';
    const more = this.candidates.length > 1 && t.y !== null ? ` · U: next of ${this.candidates.length}` : '';
    if (mode) text = `${mode}: ${text}${more}`;
    if (this.trackText.textContent !== text) this.trackText.textContent = text;
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
