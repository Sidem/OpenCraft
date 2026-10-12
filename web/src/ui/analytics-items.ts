// Materials tab of the analytics hub: what is made and what is used up, per item, with the items grouped by family
// (raw, ingots, parts, science...). A chart plots the picked items (up to one per palette colour: made as a solid
// line, used as a dashed one, or the balance); the list below is the legend and the picker. Picking: click a row,
// Shift-click to show only that one, "Busiest" for the most active, a group header's "Plot" for its busiest. Hovering a
// row lights its line. The list keeps its row elements between refreshes and only reorders when the pointer is off it.
// Everything shown is read from the engine (`analytics_item_*`); only the picks and filters live here.

import './analytics-items.css';
import { hex, Palette, PALETTE, readGraph, short } from './analytics-format';
import { LineChart, type Line } from './analytics-chart';
import { dot, iconCanvas, pointsFor, segmented, type Ctx, type Tab } from './analytics-tab';
import { button, h } from './dom';

const BUSIEST = 6;
const MODES = ['Made', 'Used', 'Both', 'Balance'];
type SortKey = 'group' | 'name' | 'made' | 'used' | 'net';
const COLUMNS: { key: SortKey; label: string }[] = [
  { key: 'name', label: 'Item' },
  { key: 'made', label: 'Made /min' },
  { key: 'used', label: 'Used /min' },
  { key: 'net', label: 'Balance /min' },
];

interface Row {
  id: number;
  name: string;
  group: number;
  el: HTMLElement;
  swatch: HTMLElement;
  made: HTMLElement;
  used: HTMLElement;
  net: HTMLElement;
  bar: HTMLElement;
  /// Latest figures.
  rate: { made: number; used: number; total: number };
}

interface Header {
  el: HTMLElement;
  note: HTMLElement;
}

export class ItemsTab implements Tab {
  readonly el = h('div', 'an-items');
  private readonly chart = new LineChart('a minute', 250);
  private readonly palette = new Palette();
  private readonly rows = new Map<number, Row>();
  private readonly headers: Header[] = [];
  private readonly list = h('div', 'an-list');
  private readonly groupChips = h('div', 'an-group-chips');
  private readonly note = h('p', 'an-note');
  private readonly modeSeg = segmented(MODES, (i) => this.pickMode(i), 'What to plot');
  private readonly sortHeads = new Map<SortKey, HTMLButtonElement>();
  private groupNames: string[] = [];
  private groupColours: string[] = [];
  private picked: number[] = [];
  private auto = true;
  private mode = 2;
  private filter = -1;
  private search = '';
  private sort: { key: SortKey; dir: 1 | -1 } = { key: 'group', dir: 1 };
  private readonly collapsed = new Set<number>();
  private hoveringList = false;
  private shownOrder = '';
  private ctx: Ctx | null = null;
  /// Whether the busiest items are chosen again at the next refresh (each time the tab is shown).
  private repick = true;

  constructor() {
    const find = h('input', 'an-search');
    find.type = 'search';
    find.placeholder = 'Find a material';
    find.addEventListener('input', () => {
      this.search = find.value.trim().toLowerCase();
      this.again();
    });
    const picks = h('div', 'an-picks');
    picks.append(
      button('secondary-btn', 'Busiest', () => this.pickBusiest()),
      button('secondary-btn', 'Clear', () => {
        this.auto = false;
        this.picked.forEach((id) => this.palette.release(id));
        this.picked = [];
        this.again();
      }),
    );
    const tools = h('div', 'an-tools');
    tools.append(find, this.modeSeg.el, picks);

    const head = h('div', 'an-list-head');
    head.append(h('span'), h('span'));
    for (const { key, label } of COLUMNS) {
      const b = button('an-sort', label, () => this.sortBy(key));
      this.sortHeads.set(key, b);
      head.append(b);
    }
    this.list.addEventListener('pointerenter', () => (this.hoveringList = true));
    this.list.addEventListener('pointerleave', () => {
      this.hoveringList = false;
      this.chart.focus(null);
    });
    const scroller = h('div', 'an-scroll');
    scroller.append(this.list);
    this.el.append(this.groupChips, tools, this.chart.el, this.note, head, scroller);
    this.modeSeg.set(this.mode);
  }

  open(): void {
    this.repick = true;
  }

  refresh(ctx: Ctx): void {
    this.ctx = ctx;
    const g = ctx.game;
    if (this.groupNames.length === 0) {
      this.groupNames = g.analytics_groups().split('\n');
      this.groupColours = Array.from(g.analytics_group_colours(), hex);
    }
    const ids = Array.from(g.analytics_items());
    const groups = g.analytics_item_groups();
    const now = g.analytics_item_now();
    ids.forEach((id, k) => {
      const row = this.rows.get(id) ?? this.makeRow(id, groups[k], g.item_name(id), ctx);
      row.rate = { made: now[4 * k], used: now[4 * k + 1], total: now[4 * k + 2] + now[4 * k + 3] };
    });
    if (this.auto && (this.picked.length === 0 || this.repick)) this.pickBusiest(false);
    this.repick = false;

    const graph = readGraph(g.analytics_item_graphs(ctx.secs, pointsFor(this.chart.width)));
    const lines: Line[] = [];
    this.picked.forEach((id) => {
      const k = ids.indexOf(id);
      const colour = this.palette.colour(id);
      const row = this.rows.get(id);
      if (k < 0 || !colour || !row || 2 * k + 1 >= graph.rows) return;
      const [made, used] = [graph.row(2 * k), graph.row(2 * k + 1)];
      const both = this.mode === 2;
      if (this.mode !== 1 && this.mode !== 3) lines.push({ name: row.name, colour, values: made, label: both ? `${row.name} made` : row.name });
      if (this.mode === 1 || both) lines.push({ name: row.name, colour, values: used, dashed: true, label: both ? `${row.name} used` : row.name });
      if (this.mode === 3) lines.push({ name: row.name, colour, values: made.map((v, i) => v - used[i]), label: row.name });
    });
    this.chart.set({ lines, secs: ctx.secs, ticks: ctx.ticks, labels: lines.length <= 10 });
    this.note.textContent = this.noteText(ids.length, lines.length);
    this.paintList();
    this.paintChips(ids, groups);
  }

  private noteText(items: number, lines: number): string {
    if (items === 0) return 'Nothing has been made yet. Miners, smelters and other machines show up here as they work.';
    if (lines === 0) return 'Pick materials from the list to graph them.';
    if (this.palette.full) return `Showing the most lines that stay readable (${PALETTE.length}); clear one to add another.`;
    return this.mode === 3
      ? 'Balance is made minus used: above the line the material piles up, below it the supply runs down.'
      : this.mode === 2
        ? 'Solid lines are what is made, dashed lines what is used up.'
        : '';
  }

  private again(): void {
    if (this.ctx) this.refresh(this.ctx);
  }

  private pickMode(i: number): void {
    this.mode = i;
    this.modeSeg.set(i);
    this.again();
  }

  private sortBy(key: SortKey): void {
    this.sort = this.sort.key === key ? { key, dir: this.sort.dir === 1 ? -1 : 1 } : { key, dir: key === 'name' ? 1 : -1 };
    this.shownOrder = '';
    this.again();
  }

  /** Picks the busiest items (of the group filter), by what they make and use. */
  private pickBusiest(redraw = true): void {
    this.auto = true;
    this.setPicks(this.matching(this.filter).slice(0, BUSIEST).map((r) => r.id));
    if (redraw) this.again();
  }

  private setPicks(ids: number[]): void {
    this.picked.filter((id) => !ids.includes(id)).forEach((id) => this.palette.release(id));
    this.picked = ids.filter((id) => this.palette.take(id) !== null);
  }

  /** The rows in a group (-1: all) that ever made or used anything: busiest now first, then by what they handled in all. */
  private matching(group: number): Row[] {
    return [...this.rows.values()]
      .filter((r) => (group < 0 || r.group === group) && r.rate.made + r.rate.used + r.rate.total > 0)
      .sort((a, b) => b.rate.made + b.rate.used - (a.rate.made + a.rate.used) || b.rate.total - a.rate.total);
  }

  private toggle(id: number, solo: boolean): void {
    this.auto = false;
    if (solo) this.setPicks([id]);
    else if (this.picked.includes(id)) this.setPicks(this.picked.filter((p) => p !== id));
    else if (this.palette.take(id)) this.picked.push(id);
    this.again();
  }

  private makeRow(id: number, group: number, name: string, ctx: Ctx): Row {
    const el = h('div', 'an-row');
    el.tabIndex = 0;
    el.setAttribute('role', 'button');
    const swatch = h('span', 'an-swatch');
    const [made, used, net] = [h('span', 'an-num'), h('span', 'an-num'), h('span', 'an-num')];
    const bar = h('i', 'an-net-bar');
    const [netCell, track] = [h('span', 'an-net'), h('span', 'an-net-track')];
    track.append(bar);
    netCell.append(net, track);
    el.append(swatch, iconCanvas(ctx.icon(id)), h('span', 'an-row-name', name), made, used, netCell);
    const row: Row = { id, name, group, el, swatch, made, used, net, bar, rate: { made: 0, used: 0, total: 0 } };
    el.style.setProperty('--group', this.groupColours[group] ?? '#888');
    el.addEventListener('click', (e) => this.toggle(id, e.shiftKey || e.altKey));
    el.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        this.toggle(id, e.shiftKey);
      }
    });
    el.addEventListener('pointerenter', () => this.chart.focus(this.picked.includes(id) ? name : null));
    this.rows.set(id, row);
    return row;
  }

  /** Updates the numbers of every row and, unless the pointer is on the list, puts them in order. */
  private paintList(): void {
    const peak = Math.max(1, ...[...this.rows.values()].map((r) => Math.abs(r.rate.made - r.rate.used)));
    for (const r of this.rows.values()) {
      const colour = this.palette.colour(r.id);
      r.el.classList.toggle('picked', colour !== null);
      r.el.setAttribute('aria-pressed', String(colour !== null));
      r.swatch.style.background = colour ?? '';
      const net = r.rate.made - r.rate.used;
      r.made.textContent = r.rate.made > 0 ? short(r.rate.made) : '-';
      r.used.textContent = r.rate.used > 0 ? short(r.rate.used) : '-';
      r.net.textContent = r.rate.made + r.rate.used === 0 ? '-' : net === 0 ? '0' : `${net > 0 ? '+' : '−'}${short(Math.abs(net))}`;
      r.net.className = `an-num ${net > 0 ? 'up' : net < 0 ? 'down' : ''}`;
      r.bar.style.width = `${(Math.abs(net) / peak) * 50}%`;
      r.bar.className = `an-net-bar ${net > 0 ? 'up' : 'down'}`;
    }
    for (const [key, b] of this.sortHeads) {
      const on = this.sort.key === key;
      b.classList.toggle('on', on);
      b.dataset.dir = on ? (this.sort.dir === 1 ? '▲' : '▼') : '';
    }
    const order = this.order();
    const signature = order.map((e) => (typeof e === 'number' ? `g${e}` : e.id)).join(',') + `|${this.search}|${this.filter}`;
    if (signature !== this.shownOrder && !(this.hoveringList && this.sameMembers(order))) {
      this.shownOrder = signature;
      this.list.replaceChildren(...order.map((e) => (typeof e === 'number' ? this.header(e).el : e.el)));
    }
    this.paintHeaders();
  }

  /** Whether the list already shows exactly these rows (so a re-sort alone can wait for the pointer to leave). */
  private sameMembers(order: (Row | number)[]): boolean {
    const shown = new Set(Array.from(this.list.querySelectorAll<HTMLElement>('.an-row')));
    const want = order.filter((e): e is Row => typeof e !== 'number');
    return want.length === shown.size && want.every((r) => shown.has(r.el));
  }

  /** The rows to show in order, with a group number before each group's rows when sorting by group. */
  private order(): (Row | number)[] {
    const rows = [...this.rows.values()].filter(
      (r) => (this.filter < 0 || r.group === this.filter) && (!this.search || r.name.toLowerCase().includes(this.search)),
    );
    const { key, dir } = this.sort;
    const value = (r: Row) =>
      key === 'name' ? 0 : key === 'made' ? r.rate.made : key === 'used' ? r.rate.used : key === 'net' ? r.rate.made - r.rate.used : r.rate.total;
    rows.sort((a, b) =>
      key === 'name' ? dir * a.name.localeCompare(b.name) : key === 'group' ? b.rate.total - a.rate.total : dir * (value(a) - value(b)),
    );
    if (key !== 'group') return rows;
    const out: (Row | number)[] = [];
    for (let g = 0; g < this.groupNames.length; g++) {
      const mine = rows.filter((r) => r.group === g);
      if (mine.length === 0) continue;
      out.push(g, ...(this.collapsed.has(g) ? [] : mine));
    }
    return out;
  }

  private header(group: number): Header {
    let head = this.headers[group];
    if (!head) {
      const el = h('div', 'an-group');
      el.style.setProperty('--group', this.groupColours[group] ?? '#888');
      const note = h('span', 'an-group-note');
      const toggle = h('button', 'an-group-name', this.groupNames[group] ?? 'Other');
      toggle.type = 'button';
      toggle.addEventListener('click', () => {
        if (!this.collapsed.delete(group)) this.collapsed.add(group);
        this.shownOrder = '';
        this.again();
      });
      const plot = button('an-group-plot', 'Plot busiest', () => {
        this.auto = false;
        this.setPicks(this.matching(group).slice(0, BUSIEST).map((r) => r.id));
        this.again();
      });
      el.append(toggle, note, plot);
      head = this.headers[group] = { el, note };
    }
    return head;
  }

  private paintHeaders(): void {
    this.headers.forEach((head, g) => {
      if (!head) return;
      const mine = [...this.rows.values()].filter((r) => r.group === g);
      const made = mine.reduce((s, r) => s + r.rate.made, 0), used = mine.reduce((s, r) => s + r.rate.used, 0);
      head.note.textContent = `${mine.length} ${mine.length === 1 ? 'item' : 'items'} · made ${short(made)} · used ${short(used)} /min`;
      head.el.classList.toggle('collapsed', this.collapsed.has(g));
    });
  }

  /** Group filter chips: All, then each family that has items. */
  private paintChips(ids: number[], groups: ArrayLike<number>): void {
    const counts = new Array<number>(this.groupNames.length).fill(0);
    ids.forEach((_, k) => counts[groups[k]]++);
    const wanted = [-1, ...counts.map((n, g) => (n > 0 ? g : -2)).filter((g) => g >= 0)];
    const key = `${wanted.join(',')}|${this.filter}|${counts.join(',')}`;
    if (this.groupChips.dataset.key === key) return;
    this.groupChips.dataset.key = key;
    this.groupChips.replaceChildren(
      ...wanted.map((g) => {
        const chip = h('button', 'an-chip');
        chip.type = 'button';
        chip.setAttribute('aria-pressed', String(g === this.filter));
        if (g >= 0) chip.append(dot(this.groupColours[g]));
        chip.append(h('span', '', g < 0 ? 'All materials' : this.groupNames[g]));
        if (g >= 0) chip.append(h('span', 'an-chip-value', String(counts[g])));
        chip.addEventListener('click', () => {
          this.filter = g;
          if (this.auto) this.pickBusiest(false);
          this.shownOrder = '';
          this.again();
        });
        return chip;
      }),
    );
  }
}
