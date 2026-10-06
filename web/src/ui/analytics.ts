// Analytics screen (P): how the factory is doing. Three parts, all read from the engine (`analytics_*`), none kept
// here but which items' lines are switched off and the timescale:
//   - Electricity: a graph of the power the grids could give, the power used and the power machines asked for;
//   - Production: a graph of each item made a minute (miners, smelters, assemblers... anything that makes items),
//     one coloured line per item, taken from its icon; click an item's chip to hide or show its line;
//   - Machines: how many are working, how many at full speed, and what holds the others back.
// The timescale buttons (1m to 3h) apply to both graphs. The data refreshes once a second while the screen is open.
// History starts when the world is opened (it is not saved). Dialog frame styles: machine.css.

import './machine.css';
import './analytics.css';
import type { Game } from '../wasm/engine.js';
import { lineColour, LineChart, short, type Line } from './analytics-chart';
import { button, h } from './dom';

// `ticks`: how many time-axis divisions, so the labels fall on round times (15 s, 1 min, 5 min, 10 min, 30 min).
const SCALES = [
  { label: '1m', secs: 60, ticks: 4 },
  { label: '5m', secs: 300, ticks: 5 },
  { label: '30m', secs: 1800, ticks: 6 },
  { label: '1h', secs: 3600, ticks: 6 },
  { label: '3h', secs: 10800, ticks: 6 },
];
const DEFAULT_SCALE = 1;
const REFRESH_MS = 1000;
const POINT_PX = 3; // pixels across per point, at most
const CHIP_ICON_PX = 22;
const POWER_ROWS = 3;

const POWER: Line[] = [
  { name: 'Could produce', colour: '#4ade80', values: [] },
  { name: 'Used', colour: '#fbbf24', values: [] },
  { name: 'Machines wanted', colour: '#f87171', values: [], dashed: true },
];

// What holds machines back, in the order of `analytics_machines` after the first two counts.
const HELD_BACK = [
  { what: 'short of power', colour: '#fbbf24' },
  { what: 'waiting for input', colour: '#7dd3fc' },
  { what: 'stopped by a full output', colour: '#c4b5fd' },
  { what: 'out of fuel', colour: '#fb923c' },
  { what: 'short of water', colour: '#38bdf8' },
  { what: 'on a thin or empty deposit', colour: '#a8a29e' },
  { what: 'blocked (flooded or overheated)', colour: '#f87171' },
];

/** A legend entry: a colour swatch (or an item icon), a name and the newest value. */
class Chip {
  readonly el = h('button', 'an-chip');
  private readonly value = h('span', 'an-chip-value');

  constructor(name: string, colour: string, icon: HTMLCanvasElement | null, onClick: (() => void) | null) {
    this.el.type = 'button';
    const swatch = h('span', 'an-dot');
    swatch.style.background = colour;
    this.el.append(swatch);
    if (icon) {
      const canvas = h('canvas', 'an-chip-icon');
      canvas.width = canvas.height = CHIP_ICON_PX;
      canvas.getContext('2d')!.drawImage(icon, 0, 0, CHIP_ICON_PX, CHIP_ICON_PX);
      this.el.append(canvas);
    }
    this.el.append(h('span', 'an-chip-name', name), this.value);
    if (onClick) this.el.addEventListener('click', onClick);
    else this.el.disabled = true;
  }

  set(text: string, off = false): void {
    if (this.value.textContent !== text) this.value.textContent = text;
    this.el.classList.toggle('off', off);
    this.el.setAttribute('aria-pressed', String(!off));
  }
}

export class AnalyticsPanel {
  /** Called after the screen closes. `resume` is true when play should continue (P, E, or a click outside). */
  onClose: (resume: boolean) => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp an');
  private readonly powerChart = new LineChart('kW');
  private readonly itemChart = new LineChart('a minute');
  private readonly powerNote = h('p', 'mp-status');
  private readonly powerChips = POWER.map((l) => new Chip(l.name, l.colour, null, null));
  private readonly itemChips = h('div', 'an-legend');
  private readonly itemNote = h('p', 'an-note', 'Nothing has been made yet. Miners, smelters and other machines show up here as they work.');
  private readonly machines = h('div', 'an-machines');
  private readonly scaleButtons: HTMLButtonElement[];
  private readonly chips = new Map<number, { chip: Chip; colour: string }>();
  private readonly hidden = new Set<number>();
  private scale = DEFAULT_SCALE;
  private last = -Infinity;

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
  ) {
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;
    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(h('h2', '', 'Analytics'), h('span', 'mp-keys', 'P, E or click outside to return'), close);

    const scales = h('div', 'an-scales');
    scales.setAttribute('role', 'group');
    scales.setAttribute('aria-label', 'Timescale');
    this.scaleButtons = SCALES.map((s, i) =>
      button('an-scale', s.label, () => {
        this.scale = i;
        this.refresh();
      }),
    );
    scales.append(h('span', 'an-scales-label', 'Last'), ...this.scaleButtons);

    const powerLegend = h('div', 'an-legend');
    powerLegend.append(...this.powerChips.map((c) => c.el));
    const body = h('div', 'mp-body');
    body.append(
      scales,
      h('h3', '', 'Electricity'),
      this.powerNote,
      this.powerChart.el,
      powerLegend,
      h('h3', '', 'Production'),
      this.itemChart.el,
      this.itemChips,
      this.itemNote,
      h('h3', '', 'Machines'),
      this.machines,
    );
    this.dialog.append(head, body);
    this.backdrop.append(this.dialog);

    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen || e.repeat) return;
      if (e.code === 'KeyP' || e.code === 'KeyE') {
        e.preventDefault();
        this.close(true);
      } else if (e.key === 'Escape') {
        e.preventDefault();
        this.close(false);
      }
    });
    window.addEventListener('resize', () => {
      if (this.isOpen) this.refresh();
    });
    document.body.append(this.backdrop);
  }

  get isOpen(): boolean {
    return !this.backdrop.classList.contains('hidden');
  }

  open(): void {
    this.backdrop.classList.remove('hidden');
    this.refresh();
    this.dialog.focus();
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  /** Call every frame: refreshes once a second while open. */
  update(now: number): void {
    if (this.isOpen && now - this.last >= REFRESH_MS) this.refresh(now);
  }

  private refresh(now = performance.now()): void {
    this.last = now;
    const g = this.game;
    const { secs, ticks } = SCALES[this.scale];
    this.scaleButtons.forEach((b, i) => b.setAttribute('aria-pressed', String(i === this.scale)));
    const points = Math.max(30, Math.floor(this.powerChart.width / POINT_PX));
    const data = g.analytics_graphs(secs, points);
    const n = data[0], rows = data[1];
    const row = (i: number) => data.subarray(2 + i * n, 2 + (i + 1) * n);
    const latest = g.analytics_now();

    this.powerChart.set(POWER.map((l, i) => ({ ...l, values: row(i) })), secs, ticks);
    this.powerChips.forEach((c, i) => c.set(`${short(latest[i])} kW`));
    this.powerNote.textContent = this.powerSummary(latest[0], latest[1], latest[2]);

    const ids = Array.from(g.analytics_items());
    const lines: Line[] = [];
    ids.forEach((id, k) => {
      if (POWER_ROWS + k >= rows) return;
      const entry = this.chipFor(id);
      entry.chip.set(`${short(latest[POWER_ROWS + k])} /min`, this.hidden.has(id));
      if (!this.hidden.has(id)) lines.push({ name: g.item_name(id), colour: entry.colour, values: row(POWER_ROWS + k) });
    });
    this.itemChart.set(lines, secs, ticks);
    this.itemNote.classList.toggle('hidden', ids.length > 0);
    this.drawMachines(Array.from(g.analytics_machines()));
  }

  private powerSummary(capacity: number, used: number, wanted: number): string {
    const kw = (v: number) => `${Math.round(v)} kW`;
    if (wanted > used + 0.5) return `Short of power: machines want ${kw(wanted)} but only get ${kw(used)}, so they slow down.`;
    if (capacity < 0.5) return used < 0.5 ? 'No power is being made or used.' : `Using ${kw(used)}, with no fuel left in the generators.`;
    return `Using ${kw(used)} of the ${kw(capacity)} the generators could make (${Math.round((used / capacity) * 100)}%).`;
  }

  /** The legend chip and line colour of an item, made the first time it is seen. */
  private chipFor(id: number): { chip: Chip; colour: string } {
    let entry = this.chips.get(id);
    if (!entry) {
      const icon = this.icon(id);
      const colour = lineColour(icon, Array.from(this.chips.values(), (e) => e.colour));
      const chip = new Chip(this.game.item_name(id), colour, icon, () => {
        if (!this.hidden.delete(id)) this.hidden.add(id);
        this.refresh();
      });
      entry = { chip, colour };
      this.chips.set(id, entry);
      this.itemChips.append(chip.el);
    }
    return entry;
  }

  private drawMachines(counts: number[]): void {
    const [working, full, ...held] = counts;
    this.machines.replaceChildren();
    if (working === 0) {
      this.machines.append(h('p', 'an-note', 'No machine is working yet.'));
      return;
    }
    const plural = (k: number) => (k === 1 ? 'machine' : 'machines');
    this.machines.append(h('p', 'mp-status', `${working} ${plural(working)} in use, ${full} at full speed.`));
    const list = h('div', 'an-legend');
    HELD_BACK.forEach((r, i) => {
      if (!(held[i] > 0)) return;
      const item = h('span', 'an-cause');
      const dot = h('span', 'an-dot');
      dot.style.background = r.colour;
      item.append(dot, h('span', '', `${held[i]} ${r.what}`));
      list.append(item);
    });
    this.machines.append(list);
    if (list.childElementCount > 0) {
      this.machines.append(h('p', 'an-note', 'Look at a machine, or open its panel, to see its own efficiency (an average of the last 10 seconds).'));
    }
  }
}
