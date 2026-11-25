// The research screen's frontier grid: ring columns left to right (Red .. Sky, the order the player opens them) and
// one row per discipline (`research-wedges.ts`). Only discovered techs show (`research-frontier.ts`): rows appear as
// their first tech is found, and columns stop at the first closed ring, drawn as a lock. Busy rings open wide with
// chips; the rest fold to dots (click a column head to swap). Prerequisite lines are drawn only for the hovered tech.
// Owns its DOM and the hover lines; what to show comes from `paint`, clicks go out through `GridHooks`.

import './research-grid.css';
import type { Game } from '../wasm/engine.js';
import { h } from './dom';
import { itemIcon, makeChip, paintChip, type Chip, type ChipState } from './research-chip';
import { autoWide, chainLinks, visibleRings, type Seen } from './research-frontier';
import { glyphSvg } from './research-kinds';
import { WEDGES } from './research-wedges';

const SVG_NS = 'http://www.w3.org/2000/svg';

export interface GridHooks {
  onSelect: (t: number) => void;
  onActivate: (t: number) => void;
  onQueue: (t: number) => void;
}

export class ResearchGrid {
  readonly el = h('div', 'rg-scroll');
  readonly chips: Chip[];
  private readonly grid = h('div', 'rg');
  private readonly svg = document.createElementNS(SVG_NS, 'svg');
  private seen: Seen[] = [];
  private ringOpen: boolean[] = [];
  private left: number[] = [];
  private wide = new Set<number>();
  /** The player's own column choice while the screen is open, or null to follow `autoWide`. */
  private chosenWide: Set<number> | null = null;
  private layout = '';
  private hovered = -1;
  private selected = -1;

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
    private readonly needs: number[][],
    /** `cells[wedge][ring]`: the techs in each place, prerequisites first (`bucket`). */
    private readonly cells: number[][][],
    private readonly ringColour: string[],
    hooks: GridHooks,
  ) {
    this.chips = needs.map((_, t) => makeChip(game, t, icon));
    this.svg.setAttribute('class', 'rg-lines');
    this.el.append(this.grid);
    for (const c of this.chips) {
      c.el.addEventListener('click', (e) => (e.shiftKey ? hooks.onQueue(c.tech) : hooks.onSelect(c.tech)));
      c.el.addEventListener('dblclick', () => hooks.onActivate(c.tech));
      c.dot.addEventListener('click', () => (this.reveal(c.tech), hooks.onSelect(c.tech)));
      for (const node of [c.el, c.dot]) {
        node.addEventListener('pointerenter', () => this.trace(c.tech));
        node.addEventListener('pointerleave', () => this.trace(-1));
      }
      c.el.addEventListener('focus', () => this.trace(c.tech));
      c.el.addEventListener('blur', () => this.trace(-1));
    }
    this.el.addEventListener('click', (e) => {
      if (!(e.target as HTMLElement).closest('.rc, .rc-dot, .rg-head')) hooks.onSelect(-1);
    });
    new ResizeObserver(() => this.drawLines()).observe(this.grid);
  }

  /** Forgets the player's column choice (on opening the screen). */
  reset(): void {
    this.chosenWide = null;
  }

  /** Repaints every chip and rebuilds the layout when discovery or the open rings changed. */
  paint(states: ChipState[], ringOpen: boolean[], left: number[]): void {
    this.seen = states.map((s) => s.seen);
    this.ringOpen = ringOpen;
    states.forEach((s, t) => paintChip(this.chips[t], s));
    this.left = left;
    this.relayout();
  }

  select(t: number): void {
    this.selected = t;
    this.mark();
  }

  /** Opens `t`'s ring wide if it is folded, and scrolls its chip into view. */
  reveal(t: number): void {
    const ring = this.game.tech_ring(t);
    if (!this.wide.has(ring)) {
      this.chosenWide = new Set(this.wide);
      this.chosenWide.add(ring);
      this.relayout();
    }
    this.chips[t].el.scrollIntoView({ block: 'nearest', inline: 'nearest' });
  }

  /** Marks search hits (null: no search). */
  search(hits: Set<number> | null): void {
    this.grid.classList.toggle('searching', hits !== null);
    for (const c of this.chips) c.el.classList.toggle('hit', hits?.has(c.tech) ?? false);
  }

  private toggleRing(ring: number): void {
    this.chosenWide = new Set(this.wide);
    if (this.chosenWide.has(ring)) this.chosenWide.delete(ring);
    else this.chosenWide.add(ring);
    this.relayout();
  }

  /** Rebuilds the grid when discovery, the open rings or the wide columns changed; marks it either way. */
  private relayout(): void {
    this.wide = this.chosenWide ?? autoWide(this.ringOpen, this.left);
    const counts = this.ringOpen.map((_, r) => this.game.ring_done(r));
    const key = `${this.seen.join('')}|${this.ringOpen.join()}|${[...this.wide].sort().join()}|${counts.join()}`;
    if (key !== this.layout) {
      this.layout = key;
      this.build();
    }
    this.mark();
  }

  private build(): void {
    const rings = visibleRings(this.ringOpen);
    const rows = WEDGES.filter((w) => rings.some((r) => this.cells[w.id][r].some((t) => this.seen[t] !== 'hidden')));
    const gate = rings.find((r) => !this.ringOpen[r]);
    this.grid.style.gridTemplateColumns = `minmax(118px, 150px) ${rings
      .map((r) => (r === gate ? '150px' : this.wide.has(r) ? 'minmax(206px, 1fr)' : '46px'))
      .join(' ')}`;
    this.hovered = -1; // a rebuilt grid gets no pointerleave from the chips it moved
    this.grid.replaceChildren(this.svg, h('div', 'rg-corner'));
    for (const r of rings) this.grid.append(this.head(r, r === gate));
    rows.forEach((w, row) => {
      const head = h('div', 'rg-row');
      head.style.setProperty('--kind', w.colour);
      head.title = w.blurb;
      const glyph = h('span', 'rg-row-icon');
      glyph.innerHTML = glyphSvg(w.kind);
      head.append(glyph, h('span', 'rg-row-name', w.name));
      this.grid.append(head);
      for (const r of rings) {
        if (r === gate) {
          if (row === 0) this.grid.append(this.lock(r, rows.length));
          continue;
        }
        const wide = this.wide.has(r);
        const cell = h('div', `rg-cell${wide ? '' : ' folded'}`);
        cell.style.setProperty('--era', this.ringColour[r]);
        for (const t of this.cells[w.id][r]) cell.append(wide ? this.chips[t].el : this.chips[t].dot);
        this.grid.append(cell);
      }
    });
    if (rows.length === 0 && gate !== undefined) this.grid.append(h('div', 'rg-row'), this.lock(gate, 1));
    this.drawLines();
  }

  private head(r: number, closed: boolean): HTMLElement {
    const g = this.game;
    const wide = this.wide.has(r);
    const el = h('button', `rg-head${closed ? ' closed' : wide ? '' : ' folded'}`) as HTMLButtonElement;
    el.type = 'button';
    el.style.setProperty('--era', this.ringColour[r]);
    const pack = g.ring_pack(r);
    if (pack >= 0) el.append(itemIcon(this.icon, pack, 'rg-pack'));
    const [done, total] = [g.ring_done(r), g.ring_total(r)];
    if (closed) {
      el.append(h('span', 'rg-head-name', `${g.ring_name(r)} ring`), h('span', 'rg-lock', '🔒'));
      el.title = 'Opens with the next milestone';
      el.disabled = true;
    } else if (wide) {
      el.append(h('span', 'rg-head-name', g.ring_name(r)), h('span', 'rg-head-count', `${done}/${total}`));
      el.title = `${g.ring_name(r)} ring: ${done} of ${total} researched. Click to fold.`;
    } else {
      el.append(h('span', 'rg-head-count', done === total ? '✓' : `${done}/${total}`));
      el.title = `${g.ring_name(r)} ring: ${done} of ${total} researched. Click to open.`;
    }
    const bar = h('span', 'rg-head-bar');
    bar.style.setProperty('--p', String(total > 0 ? done / total : 0));
    el.append(bar);
    el.addEventListener('click', () => this.toggleRing(r));
    return el;
  }

  private lock(r: number, rows: number): HTMLElement {
    const g = this.game;
    const el = h('div', 'rg-gate');
    el.style.gridRow = `span ${rows}`;
    el.style.setProperty('--era', this.ringColour[r]);
    el.append(h('span', 'rg-gate-lock', '🔒'), h('span', 'rg-gate-count', `${g.ring_total(r)} techs to discover`));
    return el;
  }

  /** The hovered tech's chain: classes on its chips and the lines between them. */
  private trace(t: number): void {
    this.hovered = t;
    this.mark();
    this.drawLines();
  }

  private mark(): void {
    const chain = new Set<number>();
    if (this.hovered >= 0) for (const l of chainLinks(this.needs, this.hovered)) chain.add(l.from).add(l.to);
    this.grid.classList.toggle('tracing', chain.size > 0);
    for (const c of this.chips) {
      c.el.classList.toggle('chain', chain.has(c.tech));
      c.el.classList.toggle('sel', c.tech === this.selected);
      c.dot.classList.toggle('sel', c.tech === this.selected);
    }
  }

  private anchor(t: number): DOMRect | null {
    if (this.seen[t] === 'hidden') return null;
    const node = this.wide.has(this.game.tech_ring(t)) ? this.chips[t].el : this.chips[t].dot;
    if (!node.isConnected) return null;
    const r = node.getBoundingClientRect();
    return r.width > 0 ? r : null;
  }

  private drawLines(): void {
    this.svg.replaceChildren();
    if (this.hovered < 0) return;
    const base = this.grid.getBoundingClientRect();
    this.svg.setAttribute('width', String(this.grid.scrollWidth));
    this.svg.setAttribute('height', String(this.grid.scrollHeight));
    for (const l of chainLinks(this.needs, this.hovered)) {
      const [a, b] = [this.anchor(l.from), this.anchor(l.to)];
      if (!a || !b) continue;
      const [ay, by] = [a.top + a.height / 2 - base.top, b.top + b.height / 2 - base.top];
      let d: string;
      if (b.left > a.right + 6) {
        const [x1, x2] = [a.right - base.left, b.left - base.left];
        const mid = (x1 + x2) / 2;
        d = `M${x1} ${ay} C${mid} ${ay} ${mid} ${by} ${x2} ${by}`;
      } else {
        const [x1, x2] = [a.left - base.left, b.left - base.left];
        const bend = Math.min(x1, x2) - 18;
        d = `M${x1} ${ay} C${bend} ${ay} ${bend} ${by} ${x2} ${by}`;
      }
      const path = document.createElementNS(SVG_NS, 'path');
      path.setAttribute('d', d);
      path.setAttribute('class', l.ahead ? 'ahead' : this.game.tech_done(l.from) ? 'met' : 'unmet');
      this.svg.append(path);
    }
  }
}
