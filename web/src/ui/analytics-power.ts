// Power tab of the analytics hub: how much power the grids make, from which kind of source, and which kinds of
// machine use it. Two stacked charts (supply by source, demand by machine kind) over the shared timescale, each with a
// list that doubles as legend: click a row to leave it out of the chart, hover it to light its area. Source names and
// colours come from the engine (`analytics_power_*`), machine kinds are blocks (names and icons as for items).

import './analytics-panels.css';
import { hex, kw, lastOf, Palette, readGraph } from './analytics-format';
import { LineChart, type Line } from './analytics-chart';
import { dot, iconCanvas, pointsFor, type Ctx, type Tab } from './analytics-tab';
import { h } from './dom';

const TOTALS = 3; // rows before the sources: could give, given, asked for
const WANTED = '#f87171';
const CAPACITY = '#4ade80';

interface Entry {
  key: number;
  name: string;
  colour: string;
  el: HTMLElement;
  now: HTMLElement;
  share: HTMLElement;
  extra: HTMLElement;
  bar: HTMLElement;
}

/** One list of rows (sources or machine kinds) next to its chart: the hidden set and how a row is made. */
class Breakdown {
  readonly list = h('div', 'an-breakdown');
  readonly entries = new Map<number, Entry>();
  readonly hidden = new Set<number>();

  constructor(
    private readonly chart: LineChart,
    private readonly again: () => void,
  ) {}

  add(key: number, name: string, colour: string, icon?: HTMLCanvasElement): Entry {
    const el = h('div', 'an-brow');
    el.tabIndex = 0;
    el.setAttribute('role', 'button');
    const [now, share, extra] = [h('span', 'an-num'), h('span', 'an-num an-faint'), h('span', 'an-num an-faint')];
    const bar = h('i', 'an-share-bar');
    bar.style.background = colour;
    const track = h('span', 'an-share');
    track.append(bar);
    el.append(dot(colour));
    if (icon) el.append(iconCanvas(icon));
    el.append(h('span', 'an-row-name', name), now, share, extra, track);
    const entry = { key, name, colour, el, now, share, extra, bar };
    const toggle = () => {
      if (!this.hidden.delete(key)) this.hidden.add(key);
      this.again();
    };
    el.addEventListener('click', toggle);
    el.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        toggle();
      }
    });
    el.addEventListener('pointerenter', () => this.chart.focus(name));
    el.addEventListener('pointerleave', () => this.chart.focus(null));
    this.entries.set(key, entry);
    this.list.append(el);
    return entry;
  }

  /** Fills a row's figures: kW now, its share of `total`, and an optional third column. */
  paint(e: Entry, value: number, total: number, extra = ''): void {
    e.now.textContent = kw(value);
    e.share.textContent = total > 0 ? `${Math.round((value / total) * 100)}%` : '-';
    e.extra.textContent = extra;
    e.bar.style.width = `${total > 0 ? (value / total) * 100 : 0}%`;
    e.el.classList.toggle('off', this.hidden.has(e.key));
    e.el.setAttribute('aria-pressed', String(!this.hidden.has(e.key)));
  }
}

export class PowerTab implements Tab {
  readonly el = h('div', 'an-power');
  private readonly summary = h('p', 'an-summary');
  private readonly tiles = h('div', 'an-tiles');
  private readonly supplyChart = new LineChart('kW', 230);
  private readonly useChart = new LineChart('kW', 200);
  private readonly sources = new Breakdown(this.supplyChart, () => this.again());
  private readonly users = new Breakdown(this.useChart, () => this.again());
  private readonly palette = new Palette();
  private readonly other = h('p', 'an-note');
  private names: string[] = [];
  private colours: string[] = [];
  private ctx: Ctx | null = null;

  constructor() {
    const key = h('p', 'an-key');
    key.append(dot(WANTED), h('span', '', 'Machines want'), dot(CAPACITY), h('span', '', 'Could make'));
    this.el.append(
      this.summary,
      this.tiles,
      h('h3', '', 'Where the power comes from'),
      this.supplyChart.el,
      key,
      this.sources.list,
      h('h3', '', 'Where the power goes'),
      this.useChart.el,
      this.users.list,
      this.other,
    );
  }

  private again(): void {
    if (this.ctx) this.refresh(this.ctx);
  }

  refresh(ctx: Ctx): void {
    this.ctx = ctx;
    const g = ctx.game;
    if (this.names.length === 0) {
      this.names = g.analytics_power_sources().split('\n');
      this.colours = Array.from(g.analytics_power_source_colours(), hex);
    }
    const points = pointsFor(this.supplyChart.width);
    const graph = readGraph(g.analytics_power_graphs(ctx.secs, points));
    const now = g.analytics_power_now();
    const could = g.analytics_power_could();
    const sourceCount = this.names.length;
    const [capacity, used, wanted] = [now[0], now[1], now[2]];
    this.summary.textContent = summary(capacity, used, wanted);
    this.paintTiles(capacity, used, wanted);

    // Sources: every kind that has ever given or could give power gets a row.
    const areas: Line[] = [];
    this.names.forEach((name, s) => {
      const values = graph.row(TOTALS + s);
      const seen = could[s] > 0 || values.some((v) => v > 0);
      const entry = this.sources.entries.get(s) ?? this.sources.add(s, name, this.colours[s]);
      entry.el.classList.toggle('hidden', !seen);
      this.sources.paint(entry, now[TOTALS + s], used, could[s] > 0 ? `of ${kw(could[s])}` : 'idle');
      if (seen && !this.sources.hidden.has(s)) areas.push({ name, colour: this.colours[s], values });
    });
    const lines: Line[] = [
      { name: 'Machines want', colour: WANTED, values: graph.row(2), dashed: true },
      { name: 'Could make', colour: CAPACITY, values: graph.row(0), dashed: true },
    ];
    this.supplyChart.set({ areas, lines, secs: ctx.secs, ticks: ctx.ticks });

    // Users: one row per kind of machine; kinds past the palette are summed as "Other".
    const blocks = Array.from(g.analytics_power_users());
    const useAreas: Line[] = [];
    let otherNow = 0;
    const rest: Float32Array[] = [];
    blocks.forEach((block, k) => {
      const values = graph.row(TOTALS + sourceCount + k);
      const colour = this.palette.take(block);
      if (!colour) {
        otherNow += lastOf(values);
        rest.push(values);
        return;
      }
      const name = g.item_name(block);
      const entry = this.users.entries.get(block) ?? this.users.add(block, name, colour, ctx.icon(block));
      this.users.paint(entry, now[TOTALS + sourceCount + k], wanted);
      if (!this.users.hidden.has(block)) useAreas.push({ name, colour, values });
    });
    if (rest.length > 0) useAreas.push({ name: 'Other machines', colour: '#6b7280', values: sum(rest) });
    this.other.textContent = rest.length > 0 ? `Other machines: ${kw(otherNow)} (${rest.length} more kinds).` : '';
    this.useChart.set({ areas: useAreas, lines: [], secs: ctx.secs, ticks: ctx.ticks });
    this.users.list.classList.toggle('hidden', blocks.length === 0);
  }

  private paintTiles(capacity: number, used: number, wanted: number): void {
    const short_ = wanted > used + 0.5;
    const tiles: [string, string, string][] = [
      ['Using', kw(used), wanted > 0 ? `of ${kw(wanted)} wanted` : 'nothing wanted'],
      ['Could make', kw(capacity), capacity > 0 ? `${Math.round((used / capacity) * 100)}% in use` : 'no fuel or sun'],
      [short_ ? 'Short by' : 'Spare', kw(short_ ? wanted - used : Math.max(0, capacity - used)), short_ ? 'machines slow down' : 'before machines slow'],
    ];
    this.tiles.replaceChildren(
      ...tiles.map(([label, value, sub], i) => {
        const tile = h('div', `an-tile${i === 2 && short_ ? ' bad' : ''}`);
        tile.append(h('span', 'an-tile-label', label), h('strong', 'an-tile-value', value), h('span', 'an-tile-sub', sub));
        return tile;
      }),
    );
  }
}

function sum(rows: Float32Array[]): Float32Array {
  const out = new Float32Array(rows[0].length);
  for (const row of rows) for (let i = 0; i < out.length; i++) out[i] += Number.isNaN(row[i]) ? 0 : row[i];
  return out;
}

export function summary(capacity: number, used: number, wanted: number): string {
  if (wanted > used + 0.5) return `Short of power: machines want ${kw(wanted)} but only get ${kw(used)}, so they slow down.`;
  if (capacity < 0.5) return used < 0.5 ? 'No power is being made or used.' : `Using ${kw(used)}, with no fuel left in the generators.`;
  return `Using ${kw(used)} of the ${kw(capacity)} the generators could make (${Math.round((used / capacity) * 100)}%).`;
}
