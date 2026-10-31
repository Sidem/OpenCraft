// Overview tab of the analytics hub: the factory at a glance. Three tiles (power, machines, materials), a short list of
// what needs attention (each line opens the tab that explains it), a small power chart and the busiest materials.
// It only reads what the other tabs read (`analytics_*`) and works out the alerts here, in words.

import './analytics-panels.css';
import { hex, kw, readGraph, short } from './analytics-format';
import { LineChart, type Line } from './analytics-chart';
import { CAUSES } from './analytics-machines';
import { iconCanvas, pointsFor, type Ctx, type Tab } from './analytics-tab';
import { h } from './dom';
import { summary } from './analytics-power';

const TOTALS = 3;
const WANTED = '#f87171';
const SHORTFALLS = 3;
const BUSIEST = 6;

interface Stat {
  id: number;
  made: number;
  used: number;
  /** Everything it made and used since the world was opened. */
  total: number;
}

/** "1 kind made now", "3 kinds used up now". */
function plural(n: number, noun: string, rest: string): string {
  return `${n} ${noun}${n === 1 ? '' : 's'} ${rest}`;
}

interface Alert {
  bad: boolean;
  text: string;
  tab: string;
}

export class OverviewTab implements Tab {
  readonly el = h('div', 'an-overview');
  private readonly headline = h('p', 'an-summary');
  private readonly tiles = h('div', 'an-tiles');
  private readonly alerts = h('div', 'an-alerts');
  private readonly chart = new LineChart('kW', 170);
  private readonly busiest = h('div', 'an-busiest');
  private names: string[] = [];
  private colours: string[] = [];

  constructor() {
    const lower = h('div', 'an-two');
    const left = h('div');
    left.append(h('h3', '', 'Power'), this.chart.el);
    const right = h('div');
    right.append(h('h3', '', 'Busiest materials'), this.busiest);
    lower.append(left, right);
    this.el.append(this.headline, this.tiles, h('h3', '', 'Needs attention'), this.alerts, lower);
  }

  refresh(ctx: Ctx): void {
    const g = ctx.game;
    if (this.names.length === 0) {
      this.names = g.analytics_power_sources().split('\n');
      this.colours = Array.from(g.analytics_power_source_colours(), hex);
    }
    const power = g.analytics_power_now(-1);
    const [capacity, used, wanted] = [power[0], power[1], power[2]];
    const [working, full, ...held] = Array.from(g.analytics_machines());
    const stride = g.analytics_type_fields();
    const types = g.analytics_machine_types();
    let weighted = 0;
    for (let i = 0; i < types.length; i += stride) weighted += types[i + 1] * types[i + 3];
    const overall = working > 0 ? Math.round(weighted / working) : 0;
    const ids = Array.from(g.analytics_items());
    const now = g.analytics_item_now();
    const stat: Stat[] = ids.map((id, k) => ({ id, made: now[4 * k], used: now[4 * k + 1], total: now[4 * k + 2] + now[4 * k + 3] }));

    const alerts = this.collectAlerts(capacity, used, wanted, held, stat, g.item_name.bind(g));
    this.headline.textContent =
      alerts.length === 0
        ? 'Everything is running smoothly.'
        : `${alerts.length} ${alerts.length === 1 ? 'thing needs' : 'things need'} attention. ${summary(capacity, used, wanted)}`;
    this.paintTiles(capacity, used, wanted, working, full, overall, stat);
    this.paintAlerts(alerts, ctx);

    const graph = readGraph(g.analytics_power_graphs(ctx.secs, pointsFor(this.chart.width), -1));
    const areas: Line[] = this.names.map((name, s) => ({ name, colour: this.colours[s], values: graph.row(TOTALS + s) }));
    this.chart.set({ areas, lines: [{ name: 'Machines want', colour: WANTED, values: graph.row(2), dashed: true }], secs: ctx.secs, ticks: ctx.ticks });
    this.paintBusiest(stat, ctx);
  }

  private collectAlerts(
    capacity: number,
    used: number,
    wanted: number,
    held: number[],
    stat: Stat[],
    name: (id: number) => string,
  ): Alert[] {
    const out: Alert[] = [];
    if (wanted > used + 0.5) out.push({ bad: true, tab: 'power', text: `Power is short: machines want ${kw(wanted)} but get ${kw(used)}.` });
    else if (capacity < 0.5 && used > 0.5) out.push({ bad: true, tab: 'power', text: 'The generators are out of fuel.' });
    else if (capacity > 0 && used / capacity > 0.9) out.push({ bad: false, tab: 'power', text: `Power is nearly maxed out (${Math.round((used / capacity) * 100)}% in use).` });
    CAUSES.forEach((c, i) => {
      if (i === 0 || !(held[i] > 0)) return; // power is covered above
      out.push({ bad: c.bad, tab: 'machines', text: `${held[i]} ${held[i] === 1 ? 'machine is' : 'machines are'} ${c.what}.` });
    });
    // Materials used up faster than they are made: the supply is running down, or a machine is starved.
    stat
      .filter((s) => s.used >= 6 && s.made < s.used * 0.7)
      .sort((a, b) => b.used - b.made - (a.used - a.made))
      .slice(0, SHORTFALLS)
      .forEach((s) => out.push({ bad: false, tab: 'items', text: `${name(s.id)}: using ${short(s.used)} a minute but making ${short(s.made)}.` }));
    return out;
  }

  private paintTiles(
    capacity: number,
    used: number,
    wanted: number,
    working: number,
    full: number,
    overall: number,
    stat: Stat[],
  ): void {
    const short_ = wanted > used + 0.5;
    const tiles: [string, string, string, boolean][] = [
      ['Power', kw(used), capacity > 0 ? `of ${kw(capacity)} · ${Math.round((used / capacity) * 100)}% in use` : 'no power made', short_],
      ['Machines', `${working} working`, working > 0 ? `${full} at full speed · ${overall}% efficient` : 'all idle', working > 0 && overall < 60],
      [
        'Materials',
        plural(stat.filter((s) => s.made > 0).length, 'kind', 'made now'),
        `${plural(stat.filter((s) => s.used > 0).length, 'kind', 'used up now')} · ${stat.length} seen in all`,
        false,
      ],
    ];
    this.tiles.replaceChildren(
      ...tiles.map(([label, value, sub, bad]) => {
        const tile = h('div', `an-tile${bad ? ' bad' : ''}`);
        tile.append(h('span', 'an-tile-label', label), h('strong', 'an-tile-value', value), h('span', 'an-tile-sub', sub));
        return tile;
      }),
    );
  }

  private paintAlerts(alerts: Alert[], ctx: Ctx): void {
    if (alerts.length === 0) {
      this.alerts.replaceChildren(h('p', 'an-note', 'Nothing to fix: power covers the machines and nothing is starved.'));
      return;
    }
    this.alerts.replaceChildren(
      ...alerts.map((a) => {
        const row = h('button', `an-alert${a.bad ? ' bad' : ''}`);
        row.type = 'button';
        row.append(h('span', 'an-alert-text', a.text), h('span', 'an-alert-go', 'Details'));
        row.addEventListener('click', () => ctx.go(a.tab));
        return row;
      }),
    );
  }

  private paintBusiest(stat: Stat[], ctx: Ctx): void {
    const top = [...stat].sort((a, b) => b.made + b.used - (a.made + a.used)).slice(0, BUSIEST).filter((s) => s.made + s.used > 0);
    if (top.length === 0) {
      const text = stat.length === 0 ? 'Nothing has been made yet.' : 'Nothing is being made or used right now.';
      this.busiest.replaceChildren(h('p', 'an-note', text));
      return;
    }
    const peak = Math.max(...top.map((s) => Math.max(s.made, s.used)));
    this.busiest.replaceChildren(
      ...top.map((s) => {
        const row = h('div', 'an-bz');
        const bars = h('span', 'an-bz-bars');
        for (const [cls, v] of [['made', s.made], ['used', s.used]] as const) {
          const bar = h('i', `an-bz-bar ${cls}`);
          bar.style.width = `${(v / peak) * 100}%`;
          bars.append(bar);
        }
        row.append(iconCanvas(ctx.icon(s.id)), h('span', 'an-row-name', ctx.game.item_name(s.id)), bars, h('span', 'an-num', `${short(s.made)} / ${short(s.used)}`));
        return row;
      }),
      h('p', 'an-note', 'Made / used a minute: green bars are made, amber bars used up.'),
    );
  }
}
