// The top of the research screen's rail: what the labs do now, the queue, what is ready to start, and the next
// milestone (the checklist that opens the next ring). Answers "what do I do next?" without reading the grid.
// Read from the engine each redraw; clicks on a name select that tech (`onSelect`).

import './research-now.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import { itemIcon, medal } from './research-chip';
import type { ResearchQueue } from './research-queue';

const READY_MAX = 8;

export class NowCard {
  readonly el = h('section', 'rn');
  private readonly doing = h('div', 'rn-doing');
  private readonly ready = h('div', 'rn-ready');
  private readonly goal = h('section', 'rn-goal');
  private drawn = '';

  constructor(
    private readonly game: Game,
    private readonly icon: (item: number) => HTMLCanvasElement,
    queue: ResearchQueue,
    private readonly onSelect: (t: number) => void,
  ) {
    this.el.append(this.doing, queue.el, this.ready);
  }

  /** The milestone card, placed by the screen under this one. */
  get goalEl(): HTMLElement {
    return this.goal;
  }

  /** Redraws when anything shown changed; `ready` lists available lab techs and undone deeds, `seenKey` discovery. */
  update(ready: number[], seenKey: string): void {
    const g = this.game;
    const cur = g.current_research();
    const rings = Array.from({ length: g.ring_count() }, (_, r) => `${g.ring_open(r)}${g.ring_pack_unlocked(r)}${g.ring_done(r)}${g.ring_parts_left(r).join()}`);
    const key = `${cur}|${cur >= 0 ? g.tech_progress(cur) : ''}|${ready.join()}|${ready.map((t) => g.tech_deed_progress(t)).join()}|${rings.join()}|${seenKey}`;
    if (key === this.drawn) return;
    this.drawn = key;
    this.paintDoing(cur, ready.some((t) => !g.tech_deed(t)));
    this.paintReady(ready);
    this.paintGoal();
  }

  /** The tech in the labs; with none, a nudge only when a lab tech is ready (a new player has no labs yet). */
  private paintDoing(cur: number, labReady: boolean): void {
    const g = this.game;
    if (cur < 0) {
      this.doing.className = `rn-doing idle${labReady ? '' : ' hidden'}`;
      this.doing.replaceChildren(h('span', 'rn-idle-dot'), h('span', 'rn-idle', 'Labs are idle'));
      return;
    }
    const endless = g.tech_endless(cur);
    const [done, units] = [g.tech_progress(cur), Math.max(1, g.tech_units(cur))];
    const row = button('rn-doing-row', '', () => this.onSelect(cur));
    const text = h('span', 'rn-doing-text');
    text.append(h('span', 'rn-label', 'Researching'), h('span', 'rn-name', g.tech_name(cur)));
    row.append(medal(g.tech_name(cur), 'rn-medal'), text, h('span', 'rn-count', endless ? `Lv ${done}` : `${done}/${units}`));
    const bar = h('div', 'mp-bar');
    const fill = h('div', 'mp-bar-fill');
    fill.style.transform = `scaleX(${endless ? 0 : done / units})`;
    bar.append(fill);
    this.doing.className = 'rn-doing';
    this.doing.replaceChildren(row, bar);
  }

  private paintReady(ready: number[]): void {
    const g = this.game;
    this.ready.replaceChildren();
    if (ready.length === 0) return;
    this.ready.append(h('span', 'rn-label', 'Ready'));
    const list = h('div', 'rn-ready-list');
    for (const t of ready.slice(0, READY_MAX)) {
      const deed = g.tech_deed(t);
      const b = button(`rn-pill${deed ? ' deed' : ''}`, '', () => this.onSelect(t));
      b.append(medal(g.tech_name(t), 'rn-pill-medal'), deed ? g.tech_deed_goal(t) : g.tech_name(t));
      if (deed && g.tech_deed_target(t) > 1) b.append(h('span', 'rn-pill-count', `${g.tech_deed_progress(t)}/${g.tech_deed_target(t)}`));
      b.title = deed ? `${g.tech_name(t)}: done by doing, in the world` : g.tech_name(t);
      list.append(b);
    }
    if (ready.length > READY_MAX) list.append(h('span', 'rn-more', `+${ready.length - READY_MAX}`));
    this.ready.append(list);
  }

  /** The first closed ring's checklist: techs of the ring before, its pack's parts, then the first pack. */
  private paintGoal(): void {
    const g = this.game;
    let r = 0;
    while (r < g.ring_count() && g.ring_open(r)) r++;
    this.goal.classList.toggle('hidden', r >= g.ring_count());
    if (r >= g.ring_count()) return;
    const pack = g.ring_pack(r);
    const head = h('div', 'rn-goal-head');
    if (pack >= 0) head.append(itemIcon(this.icon, pack, 'rn-goal-pack'));
    head.append(h('span', 'rn-goal-title', `Open the ${g.ring_name(r)} ring`));
    const list = h('ul', 'rn-steps');
    if (pack < 0) {
      list.append(step(false, 'Launch your first rocket'));
    } else {
      const unlocked = g.ring_pack_unlocked(r);
      const left = unlocked ? 0 : g.ring_threshold_left(r);
      const prev = g.ring_name(r - 1);
      const red = r === 1;
      const stepLabel =
        left > 0
          ? red
            ? `Finish ${left} more ${prev} tech or deed`
            : `Research ${left} more ${prev} tech${left === 1 ? '' : 's'}`
          : `Finish half the ${prev} ring`;
      list.append(step(left === 0, stepLabel));
      for (const t of g.ring_parts_left(r)) {
        const s = step(false, 'Research ');
        s.append(button('rn-step-link', g.tech_name(t), () => this.onSelect(t)));
        s.title = `Makes a part of the ${g.item_name(pack)}`;
        list.append(s);
      }
      const craft = step(false, 'Craft your first ');
      craft.append(itemIcon(this.icon, pack, 'rn-step-pack'), g.item_name(pack));
      craft.classList.toggle('waiting', !unlocked);
      list.append(craft);
    }
    this.goal.replaceChildren(head, list);
  }
}

function step(done: boolean, text: string): HTMLLIElement {
  const li = h('li', `rn-step${done ? ' done' : ''}`);
  li.append(h('span', 'rn-check', done ? '✓' : ''), text);
  return li;
}
