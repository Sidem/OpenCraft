// Research screen (T), built for a player who has never seen it (`docs/TECH_TREE.md` section 10). Left rail: what
// the labs do now, what is ready, the next ring's checklist (`research-now.ts`) and the selected tech in full
// (`research-detail.ts`). Right: the frontier grid (`research-grid.ts`), rings as columns, disciplines as rows, showing
// only what the player has discovered (`research-frontier.ts`). Click selects, double-click or the Research button
// starts, Shift-click queues. Also owns the HUD tracker and the "research done" / "ring open" notices.

import './inventory.css';
import './machine.css';
import './research.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import { itemIcon, type ChipState } from './research-chip';
import { TechDetail } from './research-detail';
import { bucket, discover, type Seen } from './research-frontier';
import { ResearchGrid } from './research-grid';
import { NowCard } from './research-now';
import { ResearchQueue } from './research-queue';
import { TechSearch } from './research-search';
import { WEDGE_COUNT } from './research-wedges';

const NOTICE_MS = 6000;
const SKY = '120, 190, 255';

export class ResearchPanel {
  onClose: (resume: boolean) => void = () => {};
  onDone: () => void = () => {};
  onCodex: () => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp rs');
  private readonly total = h('span', 'mp-keys');
  private readonly tracker = h('div', 'rs-tracker hidden');
  private readonly trackerText = h('span', '');
  private readonly trackerBar = h('div', 'mp-bar-fill');
  private readonly queue: ResearchQueue;
  private readonly now: NowCard;
  private readonly detail: TechDetail;
  private readonly grid: ResearchGrid;
  private readonly search: TechSearch;
  private readonly needs: number[][];
  private readonly deeds: boolean[];
  private readonly endless: boolean[];
  private readonly rings: number[];
  private done: boolean[];
  private readonly ringsOpen: boolean[];
  private seen: Seen[] = [];
  private noticeUntil = 0;
  private drawn = '';

  constructor(private readonly game: Game, icon: (item: number) => HTMLCanvasElement) {
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;
    const g = game;
    const count = g.tech_count();
    const each = <T>(f: (t: number) => T): T[] => Array.from({ length: count }, (_, t) => f(t));
    this.needs = each((t) => Array.from(g.tech_needs(t)));
    this.deeds = each((t) => g.tech_deed(t));
    this.endless = each((t) => g.tech_endless(t));
    this.rings = each((t) => g.tech_ring(t));
    this.done = each((t) => g.tech_done(t));
    this.ringsOpen = Array.from({ length: g.ring_count() }, (_, r) => g.ring_open(r));

    const colours = Array.from({ length: g.ring_count() }, (_, r) => (g.ring_pack(r) >= 0 ? average(itemIcon(icon, g.ring_pack(r), '')) : SKY));
    const cells = bucket(this.needs, this.rings, each((t) => g.tech_wedge(t)), g.ring_count(), WEDGE_COUNT);
    this.grid = new ResearchGrid(g, icon, this.needs, cells, colours, {
      onSelect: (t) => this.select(t, false),
      onActivate: (t) => this.activate(t),
      onQueue: (t) => this.queueTech(t),
    });
    this.queue = new ResearchQueue(g);
    this.now = new NowCard(g, icon, this.queue, (t) => this.select(t, true));
    this.detail = new TechDetail(g, icon, this.needs, {
      onSelect: (t) => this.select(t, true),
      onResearch: (t) => g.set_research(t),
      onQueue: (t) => this.queueTech(t),
      onUnqueue: (t) => g.unqueue_research(t),
      onStop: () => g.set_research(-1),
    });
    this.search = new TechSearch(this.grid.chips.map((c) => c.words), (t) => this.seen[t] !== 'hidden');
    this.search.onChange = () => this.grid.search(this.search.matches);
    this.search.onGo = (t) => this.select(t, true);

    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(h('h2', '', 'Research'), this.total, this.search.el, button('secondary-btn', 'Codex', () => this.onCodex()), close);
    const rail = h('div', 'rs-rail');
    rail.append(this.now.el, this.now.goalEl, this.detail.el);
    const body = h('div', 'mp-body');
    body.append(rail, this.grid.el);
    this.dialog.append(head, body);
    this.backdrop.append(this.dialog);

    const bar = h('div', 'mp-bar');
    bar.append(this.trackerBar);
    this.tracker.append(this.trackerText, bar);
    document.getElementById('hud')!.append(this.tracker);

    this.backdrop.addEventListener('pointerdown', (e) => { if (e.target === this.backdrop) this.close(true); });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => this.onKey(e));
    document.body.append(this.backdrop);
  }

  get isOpen(): boolean { return !this.backdrop.classList.contains('hidden'); }

  open(): void {
    this.backdrop.classList.remove('hidden');
    this.grid.reset();
    this.drawn = '';
    if (this.detail.selected < 0 && this.game.current_research() >= 0) this.detail.show(this.game.current_research());
    this.update(performance.now());
    this.grid.select(this.detail.selected);
    this.dialog.focus();
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.onClose(resume);
  }

  /** Call every frame: HUD tracker, notices for finished techs and open rings, and the screen when open. */
  update(now: number): void {
    const g = this.game;
    const current = g.current_research();
    for (let t = 0; t < this.done.length; t++) {
      if (this.done[t] || !g.tech_done(t)) continue;
      this.done[t] = true;
      this.notice(now, `${this.deeds[t] ? 'Deed done' : 'Research done'}: ${g.tech_name(t)}. ${this.unlockText(t)}`);
      this.onDone();
    }
    for (let r = 0; r < this.ringsOpen.length; r++) {
      if (this.ringsOpen[r] || !g.ring_open(r)) continue;
      this.ringsOpen[r] = true;
      this.notice(now, `${g.ring_name(r)} ring open: new techs to discover (T).`);
    }
    const notice = now < this.noticeUntil;
    this.tracker.classList.toggle('hidden', !notice && (current < 0 || this.isOpen));
    this.tracker.classList.toggle('rs-notice', notice);
    if (!notice && current >= 0) {
      const [done, units] = [g.tech_progress(current), g.tech_units(current)];
      const endless = g.tech_endless(current);
      this.trackerText.textContent = `Researching ${g.tech_name(current)} · ${endless ? `level ${done}` : `${done} of ${units}`}`;
      this.trackerBar.style.transform = `scaleX(${endless ? 0 : done / units})`;
    }
    if (this.isOpen) this.draw(current);
  }

  private notice(now: number, text: string): void {
    this.noticeUntil = now + NOTICE_MS;
    this.trackerText.textContent = text;
    this.trackerBar.style.transform = 'scaleX(1)';
  }

  private onKey(e: KeyboardEvent): void {
    if (!this.isOpen || e.repeat || (e.target as HTMLElement | null)?.tagName === 'INPUT') return;
    if (e.code === 'KeyT' || e.code === 'KeyE') { e.preventDefault(); this.close(true); }
    else if (e.key === 'Escape') { e.preventDefault(); this.close(false); }
    else if (e.key === '/') { e.preventDefault(); this.search.focus(); }
  }

  /** Shows `t` in the detail pane and marks it on the grid; `reveal` also opens its ring column and scrolls to it. */
  private select(t: number, reveal: boolean): void {
    this.detail.show(t);
    this.grid.select(t);
    if (reveal && t >= 0 && this.seen[t] !== 'hidden') this.grid.reveal(t);
    this.drawn = '';
  }

  /** Double-click: research it now, or queue it with what it needs. */
  private activate(t: number): void {
    const g = this.game;
    this.select(t, false);
    if (g.tech_done(t) || this.deeds[t] || g.current_research() === t) return;
    if (g.tech_available(t)) g.set_research(t);
    else if (this.queueable(t)) g.queue_research(t);
  }

  private queueTech(t: number): void {
    if (!this.game.tech_done(t) && this.queueable(t) && !this.queue.list().includes(t)) this.game.queue_research(t);
  }

  private draw(current: number): void {
    const g = this.game;
    const n = this.needs.length;
    const queued = this.queue.list();
    const avail = Array.from({ length: n }, (_, t) => g.tech_available(t));
    const prog = Array.from({ length: n }, (_, t) => (this.deeds[t] ? g.tech_deed_progress(t) : g.tech_progress(t)));
    const open = this.ringsOpen.map((_, r) => g.ring_open(r));
    this.queue.update();
    const key = `${current}|${queued.join()}|${prog.join()}|${avail.join()}|${open.join()}|${this.done.join()}`;
    if (key !== this.drawn) {
      this.drawn = key;
      this.paint(current, queued, avail, prog, open);
    }
    this.now.update(this.ready(current, queued, avail), this.seen.join(''));
    this.detail.update(this.seen, queued, (t) => this.queueable(t));
  }

  private paint(current: number, queued: number[], avail: boolean[], prog: number[], open: boolean[]): void {
    const g = this.game;
    const deedOpen = (t: number) => this.deeds[t] && open[this.rings[t]] && !this.done[t];
    this.seen = discover(this.needs.map((needs, t) => ({
      done: this.done[t],
      available: avail[t] || deedOpen(t),
      planned: t === current || queued.includes(t),
      ring: this.rings[t],
      needs,
    })), open);
    const states: ChipState[] = this.needs.map((needs, t) => ({
      seen: this.seen[t],
      done: this.done[t],
      available: avail[t] || deedOpen(t),
      chosen: t === current,
      place: queued.indexOf(t) + 1,
      progress: prog[t],
      units: g.tech_units(t),
      endless: this.endless[t],
      deed: this.deeds[t] ? g.tech_deed_goal(t) : '',
      deedProgress: prog[t],
      deedTarget: this.deeds[t] ? g.tech_deed_target(t) : 0,
      need: g.tech_name(needs.find((p) => !this.done[p]) ?? t),
    }));
    const left = open.map((_, r) => this.needs.filter((_, t) => this.rings[t] === r && !this.done[t] && !this.endless[t]).length);
    this.grid.paint(states, open, left);
    this.search.search();
    const finite = this.done.filter((d, t) => d && !this.endless[t]).length;
    const ready = this.ready(current, queued, avail).length;
    this.total.textContent = `${finite} researched${ready > 0 ? ` · ${ready} ready` : ''}`;
  }

  /** Undone deeds in open rings first, then available lab techs not yet chosen or queued, cheapest first. */
  private ready(current: number, queued: number[], avail: boolean[]): number[] {
    const g = this.game;
    const deeds = this.needs.flatMap((_, t) => (this.deeds[t] && !this.done[t] && g.ring_open(this.rings[t]) ? [t] : []));
    const labs = this.needs.flatMap((_, t) => (avail[t] && !this.deeds[t] && t !== current && !queued.includes(t) ? [t] : []));
    labs.sort((a, b) => this.rings[a] - this.rings[b] || g.tech_units(a) - g.tech_units(b) || a - b);
    return [...deeds, ...labs];
  }

  /** Whether a queue click works: its whole chain lies in open rings and holds no unfinished deed. */
  private queueable(t: number): boolean {
    const g = this.game;
    const seen = new Set<number>();
    const ok = (o: number): boolean => {
      if (seen.has(o) || g.tech_done(o)) return true;
      seen.add(o);
      return !this.deeds[o] && g.ring_open(this.rings[o]) && this.needs[o].every(ok);
    };
    return ok(t);
  }

  private unlockText(t: number): string {
    const names = Array.from(this.game.tech_unlocks(t), (item) => this.game.item_name(item));
    return names.length > 0 ? `You can now build: ${names.join(', ')}.` : '';
  }
}

/** Mean colour of the icon's saturated pixels as "r, g, b" (a ring's colour, from its science pack). */
function average(c: HTMLCanvasElement): string {
  const data = c.getContext('2d')!.getImageData(0, 0, c.width, c.height).data;
  let [r, g, b, w] = [0, 0, 0, 0];
  for (let i = 0; i < data.length; i += 4) {
    const [pr, pg, pb] = [data[i], data[i + 1], data[i + 2]];
    const sat = Math.max(pr, pg, pb) - Math.min(pr, pg, pb);
    if (data[i + 3] < 200 || sat < 40) continue;
    r += pr * sat;
    g += pg * sat;
    b += pb * sat;
    w += sat;
  }
  return w > 0 ? `${Math.round(r / w)}, ${Math.round(g / w)}, ${Math.round(b / w)}` : '160, 170, 190';
}
