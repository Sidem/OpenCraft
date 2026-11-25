// The selected tech, in full, in the research screen's rail: what it unlocks, what it costs, what it needs and what
// it leads to (both clickable), and the one action that fits its state (research, queue, stop, or the deed's job).
// With nothing selected it shows the three-line "how research works". Undiscovered techs it leads to stay unnamed.

import './research-detail.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import { itemIcon, medal } from './research-chip';
import { dependents } from './research-frontier';
import type { Seen } from './research-frontier';

export interface DetailHooks {
  onSelect: (t: number) => void;
  onResearch: (t: number) => void;
  onQueue: (t: number) => void;
  onUnqueue: (t: number) => void;
  onStop: () => void;
}

export class TechDetail {
  readonly el = h('section', 'rd');
  private tech = -1;
  private drawn = '';

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
    private readonly needs: number[][],
    private readonly hooks: DetailHooks,
  ) {}

  get selected(): number {
    return this.tech;
  }

  show(t: number): void {
    this.tech = t;
    this.drawn = '';
  }

  /** Redraws when the selected tech's state changed. `seen` is discovery, `queueable` whether a queue click works. */
  update(seen: Seen[], queued: number[], queueable: (t: number) => boolean): void {
    const g = this.game;
    const t = this.tech;
    const key = t < 0 ? '-' : `${t}|${g.current_research()}|${queued.join()}|${g.tech_progress(t)}|${g.tech_deed_progress(t)}|${seen.join('')}`;
    if (key === this.drawn) return;
    this.drawn = key;
    if (t < 0) this.el.replaceChildren(...this.intro());
    else this.el.replaceChildren(...this.body(t, seen, queued, queueable(t)));
  }

  private intro(): HTMLElement[] {
    const rows: [string, string][] = [
      ['🧪', 'Labs turn science packs into research.'],
      ['👆', 'Pick a glowing tech, then press Research.'],
      ['🔓', 'Finish a ring’s milestone to open the next ring and find new techs.'],
    ];
    const list = h('ul', 'rd-intro');
    for (const [icon, text] of rows) {
      const li = h('li', '');
      li.append(h('span', 'rd-intro-icon', icon), text);
      list.append(li);
    }
    return [h('h3', 'rd-title', 'How research works'), list];
  }

  private body(t: number, seen: Seen[], queued: number[], queueable: boolean): HTMLElement[] {
    const g = this.game;
    const done = g.tech_done(t);
    const chosen = g.current_research() === t;
    const place = queued.indexOf(t) + 1;
    const deed = g.tech_deed(t);
    const state = done ? 'Done' : chosen ? 'Researching' : place > 0 ? `Queued #${place}` : deed ? 'Do it in the world' : g.tech_available(t) ? 'Ready' : 'Locked';
    const head = h('div', 'rd-head');
    const title = h('div', 'rd-name');
    title.append(h('h3', 'rd-title', g.tech_name(t)), h('span', `rd-state ${state.split(' ')[0].toLowerCase()}`, state));
    head.append(medal(g.tech_name(t), 'rd-medal'), title);
    const out: HTMLElement[] = [head, h('p', 'rd-blurb', g.tech_blurb(t))];
    out.push(this.action(t, { done, chosen, place, deed, queueable }));

    const unlocks = Array.from(g.tech_unlocks(t));
    if (unlocks.length > 0) out.push(this.section('Unlocks', unlocks.map((i) => this.item(i, g.item_name(i)))));
    if (!deed && !done) {
      const cost = h('span', 'rd-cost');
      cost.append(`${g.tech_endless(t) ? 'Per level' : g.tech_units(t)} ×`);
      for (const p of g.tech_packs(t)) {
        const c = itemIcon(this.icon, p, 'rd-cost-icon');
        c.title = g.item_name(p);
        cost.append(c);
      }
      cost.append(h('span', 'rd-dim', `${Math.round(g.tech_seconds(t))} s each`));
      out.push(this.section('Cost', [cost]));
    }
    if (this.needs[t].length > 0) out.push(this.section('Needs', this.needs[t].map((n) => this.link(n, g.tech_done(n)))));
    const next = dependents(this.needs, t);
    const known = next.filter((d) => seen[d] !== 'hidden');
    const unknown = next.length - known.length;
    const leads = known.map((d) => this.link(d, g.tech_done(d)));
    if (unknown > 0) leads.push(h('span', 'rd-unknown', `+${unknown} to discover`));
    if (leads.length > 0) out.push(this.section('Leads to', leads));
    return out;
  }

  private action(t: number, s: { done: boolean; chosen: boolean; place: number; deed: boolean; queueable: boolean }): HTMLElement {
    const g = this.game;
    const el = h('div', 'rd-actions');
    if (s.done) return el;
    if (s.deed) {
      const target = Math.max(1, g.tech_deed_target(t));
      const job = h('div', 'rd-deed');
      job.append(h('span', '', g.tech_deed_goal(t)), h('span', 'rd-dim', target > 1 ? `${g.tech_deed_progress(t)} / ${target}` : ''));
      const bar = h('div', 'mp-bar');
      const fill = h('div', 'mp-bar-fill');
      fill.style.transform = `scaleX(${Math.min(1, g.tech_deed_progress(t) / target)})`;
      bar.append(fill);
      el.append(job, bar);
    } else if (s.chosen) {
      el.append(button('secondary-btn', 'Stop', () => this.hooks.onStop()));
    } else if (s.place > 0) {
      el.append(button('secondary-btn', 'Take out of queue', () => this.hooks.onUnqueue(t)));
    } else if (g.tech_available(t)) {
      el.append(button('secondary-btn rd-go', 'Research', () => this.hooks.onResearch(t)), button('secondary-btn', 'Add to queue', () => this.hooks.onQueue(t)));
    } else if (s.queueable) {
      el.append(button('secondary-btn rd-go', 'Queue with what it needs', () => this.hooks.onQueue(t)));
    } else {
      const ring = g.tech_ring(t);
      el.append(h('span', 'rd-dim', g.ring_open(ring) ? 'Needs a deed done first.' : `Opens with the ${g.ring_name(ring)} ring.`));
    }
    return el;
  }

  private section(label: string, items: HTMLElement[]): HTMLElement {
    const el = h('div', 'rd-section');
    const list = h('div', 'rd-list');
    list.append(...items);
    el.append(h('span', 'rd-label', label), list);
    return el;
  }

  private item(item: number, name: string): HTMLElement {
    const el = h('span', 'rd-item');
    el.append(itemIcon(this.icon, item, 'rd-item-icon'), name);
    return el;
  }

  private link(t: number, done: boolean): HTMLElement {
    const b = button(`rd-link${done ? ' done' : ''}`, '', () => this.hooks.onSelect(t));
    b.append(h('span', 'rd-link-mark', done ? '✓' : '○'), this.game.tech_name(t));
    return b;
  }
}
