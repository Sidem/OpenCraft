// Machines tab of the analytics hub: how well the machines run and what holds them back. A bar splits the working
// machines by what costs them most (full speed, short of power, waiting for input...); a table gives each kind of
// machine its count, average efficiency and causes; a list names the slowest machines with where they stand and how
// far away. Every row has "Show on map" (the world map rings that kind, map-focus-layer.ts). Efficiency is the engine's
// average of the last 10 seconds (`analytics_machine*`, `analytics_worst_*`).

import './analytics-panels.css';
import { dot, iconCanvas, type Ctx, type Tab } from './analytics-tab';
import { button, h } from './dom';

// What holds machines back, in the order of the engine's causes (`analytics_machines` after the first two counts).
// `bad` marks the causes that are a real fault (the overview shows them in red) rather than a thing worth a look.
export const CAUSES = [
  { short: 'power', what: 'short of power', colour: '#fbbf24', bad: true },
  { short: 'input', what: 'waiting for input', colour: '#7dd3fc', bad: false },
  { short: 'output', what: 'stopped by a full output', colour: '#c4b5fd', bad: false },
  { short: 'fuel', what: 'out of fuel', colour: '#fb923c', bad: true },
  { short: 'water', what: 'short of water', colour: '#38bdf8', bad: true },
  { short: 'deposit', what: 'on a thin or empty deposit', colour: '#a8a29e', bad: false },
  { short: 'blocked', what: 'blocked (flooded or overheated)', colour: '#f87171', bad: true },
];
const FULL = '#4ade80';
const WORST = 10;

/** Green when healthy, amber when slowed, red when badly held back. */
export function healthColour(percent: number): string {
  return percent >= 90 ? FULL : percent >= 60 ? '#fbbf24' : '#f87171';
}

export class MachinesTab implements Tab {
  readonly el = h('div', 'an-machines');
  private readonly summary = h('p', 'an-summary');
  private readonly split = h('div', 'an-split');
  private readonly legend = h('div', 'an-legend');
  private readonly table = h('div', 'an-mtable');
  private readonly slowest = h('div', 'an-slowest');
  private readonly slowHead = h('h3', '', 'Slowest machines');

  constructor() {
    this.el.append(this.summary, this.split, this.legend, h('h3', '', 'By kind of machine'), this.table, this.slowHead, this.slowest);
  }

  refresh(ctx: Ctx): void {
    const g = ctx.game;
    const [working, full, ...held] = Array.from(g.analytics_machines());
    const stride = g.analytics_type_fields();
    const types = g.analytics_machine_types();
    this.el.classList.toggle('empty', working === 0 && types.length === 0);
    if (types.length === 0) {
      this.summary.textContent = 'No machine is working yet. Build miners, smelters and assemblers and they show up here.';
      this.split.replaceChildren();
      this.legend.replaceChildren();
      this.table.replaceChildren();
      this.slowest.replaceChildren();
      this.slowHead.classList.add('hidden');
      return;
    }
    let weighted = 0;
    for (let i = 0; i < types.length; i += stride) weighted += types[i + 1] * types[i + 3];
    const overall = working > 0 ? Math.round(weighted / working) : 0;
    this.summary.textContent =
      working === 0
        ? 'Every machine is idle at the moment.'
        : `${working} ${working === 1 ? 'machine' : 'machines'} working, ${full} at full speed: ${overall}% efficient overall.`;
    this.paintSplit(working, full, held);
    this.paintTable(g, ctx, types, stride);
    this.paintSlowest(g, ctx);
  }

  /** The bar of working machines split by their main cause, and its legend. */
  private paintSplit(working: number, full: number, held: number[]): void {
    const parts = [{ label: 'at full speed', colour: FULL, n: full }, ...CAUSES.map((c, i) => ({ label: c.what, colour: c.colour, n: held[i] ?? 0 }))];
    this.split.replaceChildren(
      ...parts
        .filter((p) => p.n > 0)
        .map((p) => {
          const seg = h('span', 'an-seg-part');
          seg.style.flex = String(p.n);
          seg.style.background = p.colour;
          seg.title = `${p.n} ${p.label}`;
          return seg;
        }),
    );
    this.legend.replaceChildren(
      ...parts
        .filter((p) => p.n > 0 && working > 0)
        .map((p) => {
          const item = h('span', 'an-cause');
          item.append(dot(p.colour), h('span', '', `${p.n} ${p.label}`));
          return item;
        }),
    );
  }

  /** One row per kind of machine, the least efficient first. */
  private paintTable(g: Ctx['game'], ctx: Ctx, types: Uint32Array, stride: number): void {
    const rows: { block: number; row: number[] }[] = [];
    for (let i = 0; i < types.length; i += stride) rows.push({ block: types[i], row: Array.from(types.subarray(i, i + stride)) });
    // Kinds with machines working come first, worst average first; idle kinds close the list.
    rows.sort((a, b) => (b.row[1] > 0 ? 1 : 0) - (a.row[1] > 0 ? 1 : 0) || a.row[3] - b.row[3]);
    const head = h('div', 'an-mrow an-mhead');
    head.append(h('span'), h('span', '', 'Machine'), h('span', 'an-num', 'Working'), h('span', 'an-num', 'Idle'), h('span', '', 'Efficiency'), h('span', '', 'Held back by'), h('span'));
    this.table.replaceChildren(head);
    for (const { block, row } of rows) {
      const [, working, full, percent, idle] = row;
      const el = h('div', 'an-mrow');
      const meter = h('span', 'an-meter');
      const fill = h('i', 'an-meter-fill');
      fill.style.width = `${working > 0 ? percent : 0}%`;
      fill.style.background = healthColour(percent);
      meter.append(fill, h('b', '', working > 0 ? `${percent}%` : '–'));
      const causes = h('span', 'an-causes');
      CAUSES.forEach((c, i) => {
        if (!(row[5 + i] > 0)) return;
        const chip = h('span', 'an-cause-chip');
        chip.append(dot(c.colour), h('span', '', `${row[5 + i]} ${c.short}`));
        chip.title = `${row[5 + i]} ${c.what}`;
        causes.append(chip);
      });
      if (working > 0 && full === working) causes.append(h('span', 'an-faint', 'all at full speed'));
      el.append(iconCanvas(ctx.icon(block)), h('span', 'an-row-name', g.item_name(block)), h('span', 'an-num', String(working)), h('span', 'an-num an-faint', String(idle)), meter, causes, mapButton(ctx, block));
      this.table.append(el);
    }
  }

  private paintSlowest(g: Ctx['game'], ctx: Ctx): void {
    const worst = g.analytics_worst_machines(WORST);
    this.slowHead.classList.toggle('hidden', worst.length === 0);
    this.slowest.replaceChildren();
    const [px, pz] = [g.player_x(), g.player_z()];
    for (let i = 0; i < worst.length; i += 6) {
      const [x, y, z, block, percent, cause] = worst.subarray(i, i + 6);
      const el = h('div', 'an-slow');
      const away = Math.round(Math.hypot(x - px, z - pz));
      const why = CAUSES[cause]?.what ?? 'held back';
      el.append(
        iconCanvas(ctx.icon(block)),
        h('span', 'an-row-name', g.item_name(block)),
        h('strong', 'an-slow-pct', `${percent}%`),
        h('span', 'an-slow-why', why),
        h('span', 'an-faint', `at ${x}, ${y}, ${z} · ${away} blocks away`),
        mapButton(ctx, block, { x, z }),
      );
      el.querySelector<HTMLElement>('.an-slow-pct')!.style.color = healthColour(percent);
      this.slowest.append(el);
    }
  }
}

/** The row's "Show on map" button: every machine of the kind, centred on one if `at` is given. */
function mapButton(ctx: Ctx, block: number, at?: { x: number; z: number }): HTMLButtonElement {
  return button('secondary-btn an-map-btn', 'Show on map', () => ctx.showOnMap(block, at));
}
