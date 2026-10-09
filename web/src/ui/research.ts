// Research screen (T): the whole tech tree at a glance. Each tech is a small node in a column by depth
// (layout: tech-tree.ts), joined to its prerequisites by curves. A node says its state by colour: done
// (green, what the rest is built on), researching (orange, with progress), available (bright and pulsing:
// the horizon) and locked (dim). Hovering or focusing a node shows the full card (what it unlocks, what a
// unit costs, progress) and lights its chain of prerequisites and dependents; clicking an available tech
// sets what every lab in the world works on now (again to give it up), Shift-click or a click on a locked tech
// adds it, with the prerequisites it lacks, to the research queue (again to take it out); the queue is listed
// above the tree (research-queue.ts). Also the HUD tracker (only the tech being researched and its progress)
// and a short notice when a tech is done. Everything is read from the engine (`tech_*`, `current_research`,
// `research_queue_*`); a new tech in the engine needs no change here.
// The dialog frame reuses the machine panel's styles (machine.css), the item chips the build menu's
// (inventory.css).

import './inventory.css';
import './machine.css';
import './research.css';
import './research-card.css';
import type { Game } from '../wasm/engine.js';
import { button, h } from './dom';
import { ResearchQueue } from './research-queue';
import { layoutTree, related } from './tech-tree';

const ICON_PX = 64;
const NOTICE_MS = 6000;
const NODE_W = 176;
const NODE_H = 58;
const COL_GAP = 64;
const ROW_GAP = 16;
const SVG_NS = 'http://www.w3.org/2000/svg';

interface TechView {
  node: HTMLButtonElement;
  nodeState: HTMLSpanElement;
  nodeBar: HTMLElement;
  /** The hover card. */
  card: HTMLDivElement;
  state: HTMLSpanElement;
  bar: HTMLDivElement;
  count: HTMLSpanElement;
  /** The lab time of the next unit (it grows with an endless tech's level). */
  seconds: HTMLSpanElement;
  action: HTMLParagraphElement;
}

interface Edge {
  from: number;
  to: number;
  path: SVGPathElement;
}

export class ResearchPanel {
  /** Called after the screen closes. `resume` is true when play should continue (R, E, or a click outside). */
  onClose: (resume: boolean) => void = () => {};
  /** Called when a tech is done. */
  onDone: () => void = () => {};

  private readonly backdrop = h('div', 'mp-backdrop hidden');
  private readonly dialog = h('div', 'mp rs');
  private readonly current = h('p', 'mp-status');
  private readonly tracker = h('div', 'rs-tracker hidden');
  private readonly trackerText = h('span', '');
  private readonly trackerBar = h('div', 'mp-bar-fill');
  private readonly tree = h('div', 'rs-tree');
  private readonly tip = h('div', 'rs-tip hidden');
  private readonly queue: ResearchQueue;
  private readonly views: TechView[];
  private readonly needs: number[][];
  private readonly edges: Edge[] = [];
  private done: boolean[];
  private noticeUntil = 0;
  private drawn = '';
  private centring = false;

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
    head.append(h('h2', '', 'Research'), h('span', 'mp-keys', 'T, E or click outside to return'), close);

    this.queue = new ResearchQueue(game);
    const count = game.tech_count();
    this.needs = Array.from({ length: count }, (_, t) => Array.from(game.tech_needs(t)));
    this.views = Array.from({ length: count }, (_, t) => this.makeTech(t));
    this.done = Array.from({ length: count }, (_, t) => game.tech_done(t));
    this.buildTree();
    this.enablePan();
    const body = h('div', 'mp-body');
    body.append(this.current, this.queue.el, this.legend(), this.tree);
    this.dialog.append(head, body);
    this.backdrop.append(this.dialog, this.tip);

    const bar = h('div', 'mp-bar');
    bar.append(this.trackerBar);
    this.tracker.append(this.trackerText, bar);
    document.getElementById('hud')!.append(this.tracker);

    this.backdrop.addEventListener('pointerdown', (e) => {
      if (e.target === this.backdrop) this.close(true);
    });
    this.backdrop.addEventListener('contextmenu', (e) => e.preventDefault());
    window.addEventListener('keydown', (e) => {
      if (!this.isOpen || e.repeat) return;
      if (e.code === 'KeyT' || e.code === 'KeyE') {
        e.preventDefault();
        this.close(true);
      } else if (e.key === 'Escape') {
        e.preventDefault();
        this.close(false);
      }
    });
    document.body.append(this.backdrop);
  }

  get isOpen(): boolean {
    return !this.backdrop.classList.contains('hidden');
  }

  open(): void {
    this.backdrop.classList.remove('hidden');
    this.drawn = '';
    this.update(performance.now());
    this.dialog.focus();
    this.centring = true;
  }

  close(resume: boolean): void {
    if (!this.isOpen) return;
    this.backdrop.classList.add('hidden');
    this.hideCard();
    this.onClose(resume);
  }

  /** Call every frame: the HUD tracker, notices for finished techs, and the screen when open. */
  update(now: number): void {
    const g = this.game;
    const current = g.current_research();
    for (let t = 0; t < this.done.length; t++) {
      if (this.done[t] || !g.tech_done(t)) continue;
      this.done[t] = true;
      this.noticeUntil = now + NOTICE_MS;
      this.trackerText.textContent = `Research done: ${g.tech_name(t)}. ${this.unlockText(t)}`;
      this.trackerBar.style.transform = 'scaleX(1)';
      this.onDone();
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
    if (this.isOpen) {
      this.queue.update();
      this.draw(current);
    }
    // Just opened: the dialog has no width for the first moments, so centre once it is laid out.
    if (this.isOpen && this.centring && this.tree.clientWidth > NODE_W) {
      this.centring = false;
      this.scrollToHorizon();
    }
  }

  /** Places the nodes and draws the curves between them; states are set by `draw`. */
  private buildTree(): void {
    const lay = layoutTree(this.needs);
    const pos = (t: number) => ({ x: lay.col[t] * (NODE_W + COL_GAP), y: lay.row[t] * (NODE_H + ROW_GAP) });
    const width = lay.columns * (NODE_W + COL_GAP) - COL_GAP;
    const height = lay.rows * (NODE_H + ROW_GAP) - ROW_GAP;
    const stage = h('div', 'rs-stage');
    stage.style.width = `${width}px`;
    stage.style.height = `${height}px`;
    const svg = document.createElementNS(SVG_NS, 'svg');
    svg.setAttribute('class', 'rs-edges');
    svg.setAttribute('width', String(width));
    svg.setAttribute('height', String(height));
    this.needs.forEach((parents, to) => {
      for (const from of parents) {
        const [a, b] = [pos(from), pos(to)];
        const [x1, y1, x2, y2] = [a.x + NODE_W, a.y + NODE_H / 2, b.x, b.y + NODE_H / 2];
        const path = document.createElementNS(SVG_NS, 'path');
        const bend = (x2 - x1) / 2;
        path.setAttribute('d', `M${x1} ${y1} C${x1 + bend} ${y1} ${x2 - bend} ${y2} ${x2} ${y2}`);
        svg.append(path);
        this.edges.push({ from, to, path });
      }
    });
    stage.append(svg);
    this.views.forEach((v, t) => {
      v.node.style.left = `${pos(t).x}px`;
      v.node.style.top = `${pos(t).y}px`;
      stage.append(v.node);
    });
    this.tree.append(stage);
  }

  /** Drag the background to pan the tree; the wheel scrolls sideways when the tree fits vertically. */
  private enablePan(): void {
    const tree = this.tree;
    let from: { x: number; y: number; left: number; top: number } | null = null;
    tree.addEventListener('pointerdown', (e) => {
      if ((e.target as HTMLElement).closest('.rs-node')) return;
      from = { x: e.clientX, y: e.clientY, left: tree.scrollLeft, top: tree.scrollTop };
      tree.setPointerCapture(e.pointerId);
      tree.classList.add('panning');
    });
    tree.addEventListener('pointermove', (e) => {
      if (!from) return;
      tree.scrollLeft = from.left - (e.clientX - from.x);
      tree.scrollTop = from.top - (e.clientY - from.y);
    });
    for (const end of ['pointerup', 'pointercancel']) {
      tree.addEventListener(end, () => {
        from = null;
        tree.classList.remove('panning');
      });
    }
    tree.addEventListener('wheel', (e) => {
      if (tree.scrollHeight > tree.clientHeight || Math.abs(e.deltaX) > Math.abs(e.deltaY)) return;
      tree.scrollLeft += e.deltaY;
      e.preventDefault();
    }, { passive: false });
  }

  /** Redraws the states when the current tech or any progress changed. */
  private draw(current: number): void {
    const g = this.game;
    const progress = this.views.map((_, t) => g.tech_progress(t));
    const queued = this.queue.list();
    const key = `${current}|${queued.join(',')}|${progress.join(',')}|${this.views.map((_, t) => g.tech_available(t)).join(',')}`;
    if (key === this.drawn) return;
    this.drawn = key;
    this.current.textContent = current >= 0
      ? `Labs are researching ${g.tech_name(current)}.`
      : 'Nothing is being researched. Click a glowing tech to start, or Shift-click to queue.';
    this.views.forEach((v, t) => {
      const [done, available, units, endless] = [g.tech_done(t), g.tech_available(t), g.tech_units(t), g.tech_endless(t)];
      const amount = endless ? `level ${progress[t]}` : `${progress[t]} of ${units}`;
      const chosen = t === current;
      const place = queued.indexOf(t) + 1;
      const state = done ? 'done' : available ? 'available' : 'locked';
      const label = done ? 'Done' : chosen ? 'Researching' : place > 0 ? `Queued #${place}` : available ? 'Available' : 'Locked';
      const extra = `${chosen ? ' chosen' : ''}${place > 0 ? ' queued' : ''}`;
      v.node.className = `rs-node ${state}${extra}`;
      v.card.className = `rs-card ${state}${extra}`;
      v.nodeState.textContent = chosen || (available && progress[t] > 0) ? amount : label;
      v.state.textContent = label;
      const fill = `scaleX(${endless ? 0 : progress[t] / units})`;
      v.nodeBar.style.transform = fill;
      v.bar.style.transform = fill;
      v.count.textContent = endless ? `${amount} · endless` : `${amount} units`;
      v.seconds.textContent = `${Math.round(g.tech_seconds(t))} s in a ${endless ? 'AI lab, using compute' : 'lab'}`;
      v.action.textContent = done
        ? 'Researched: what comes after it builds on this.'
        : chosen
          ? 'Click to give this up and move on to the next in the queue.'
          : place > 0
            ? 'Click to take this out of the queue.'
            : available
              ? 'Click to research this now, Shift-click to add it to the queue.'
              : `${this.needsText(t)} Click to queue it with everything it needs.`;
    });
    for (const e of this.edges) {
      const [from, to] = [g.tech_done(e.from), g.tech_done(e.to)];
      e.path.setAttribute('class', from && to ? 'built' : from ? 'open' : 'closed');
    }
  }

  private makeTech(t: number): TechView {
    const g = this.game;
    const node = h('button', 'rs-node') as HTMLButtonElement;
    node.type = 'button';
    node.setAttribute('aria-label', g.tech_name(t));
    const meta = h('span', 'rs-node-meta');
    const packs = h('span', 'rs-packs');
    for (const item of g.tech_packs(t)) packs.append(this.packIcon(item, g.item_name(item)));
    const nodeState = h('span', 'rs-node-state');
    meta.append(packs, nodeState);
    const track = h('span', 'rs-node-track');
    const nodeBar = h('i', '');
    track.append(nodeBar);
    node.append(h('span', 'rs-node-name', g.tech_name(t)), meta, track);
    node.addEventListener('click', (e) => {
      if (g.tech_done(t)) return;
      if (g.current_research() === t) g.set_research(-1);
      else if (this.queue.list().includes(t)) g.unqueue_research(t);
      else if (g.tech_available(t) && !e.shiftKey) g.set_research(t);
      else g.queue_research(t);
    });
    for (const on of ['pointerenter', 'focus']) node.addEventListener(on, () => this.showCard(t));
    for (const off of ['pointerleave', 'blur']) node.addEventListener(off, () => this.hideCard());

    const card = h('div', 'rs-card');
    const title = h('div', 'rs-title');
    const state = h('span', 'rs-state');
    title.append(h('h4', '', g.tech_name(t)), state);
    const unlocks = h('div', 'rs-chips');
    unlocks.append(h('span', 'rs-label', 'Unlocks'));
    for (const item of g.tech_unlocks(t)) unlocks.append(this.chip(item, g.item_name(item)));
    const cost = h('div', 'rs-chips');
    cost.append(h('span', 'rs-label', 'Each unit'));
    for (const item of g.tech_packs(t)) cost.append(this.chip(item, `1 ${g.item_name(item)}`));
    const seconds = h('span', 'rs-label', `${g.tech_seconds(t)} s in a lab`);
    cost.append(seconds);
    const progress = h('div', 'rs-progress');
    const barBox = h('div', 'mp-bar');
    const bar = h('div', 'mp-bar-fill');
    barBox.append(bar);
    const count = h('span', 'rs-count');
    progress.append(barBox, count);
    const action = h('p', 'rs-action');
    card.append(title, h('p', 'rs-blurb', g.tech_blurb(t)), unlocks, cost, progress, action);
    return { node, nodeState, nodeBar, card, state, bar, count, seconds, action };
  }

  /** Shows tech `t`'s card beside its node and lights the chain of techs it belongs to. */
  private showCard(t: number): void {
    const chain = related(this.needs, t);
    this.tree.classList.add('rs-focus');
    this.views.forEach((v, i) => v.node.classList.toggle('rel', chain.has(i)));
    for (const e of this.edges) e.path.classList.toggle('hot', chain.has(e.from) && chain.has(e.to));
    this.tip.replaceChildren(this.views[t].card);
    this.tip.classList.remove('hidden');
    const [n, box] = [this.views[t].node.getBoundingClientRect(), this.tip.getBoundingClientRect()];
    const right = n.right + 12 + box.width <= window.innerWidth - 8;
    const x = right ? n.right + 12 : Math.max(8, n.left - 12 - box.width);
    const y = Math.min(Math.max(8, n.top - 8), Math.max(8, window.innerHeight - box.height - 8));
    this.tip.style.left = `${x}px`;
    this.tip.style.top = `${y}px`;
  }

  private hideCard(): void {
    this.tip.classList.add('hidden');
    this.tree.classList.remove('rs-focus');
    this.views.forEach((v) => v.node.classList.remove('rel'));
    for (const e of this.edges) e.path.classList.remove('hot');
  }

  /** Scrolls the tree so the tech being researched, else the first available one, is in view. */
  private scrollToHorizon(): void {
    const g = this.game;
    const t = g.current_research() >= 0
      ? g.current_research()
      : this.views.findIndex((_, i) => g.tech_available(i));
    if (t < 0) return;
    const node = this.views[t].node;
    this.tree.scrollLeft = Math.max(0, node.offsetLeft - (this.tree.clientWidth - NODE_W) / 2);
    this.tree.scrollTop = Math.max(0, node.offsetTop - (this.tree.clientHeight - NODE_H) / 2);
  }

  private legend(): HTMLDivElement {
    const el = h('div', 'rs-legend');
    for (const [cls, text] of [
      ['done', 'Done: built upon'],
      ['chosen', 'Being researched'],
      ['queued', 'In the queue'],
      ['available', 'On the horizon: click to research, Shift-click to queue'],
      ['locked', 'Locked'],
    ]) {
      el.append(h('span', `rs-key ${cls}`, text));
    }
    return el;
  }

  private needsText(t: number): string {
    const g = this.game;
    const missing = this.needs[t].filter((n) => !g.tech_done(n));
    return `Needs ${missing.map((n) => g.tech_name(n)).join(' and ')} first.`;
  }

  private unlockText(t: number): string {
    const names = Array.from(this.game.tech_unlocks(t), (item) => this.game.item_name(item));
    return names.length > 0 ? `You can now build: ${names.join(', ')}.` : '';
  }

  private packIcon(item: number, name: string): HTMLCanvasElement {
    const c = h('canvas', 'rs-pack') as HTMLCanvasElement;
    c.width = c.height = ICON_PX;
    c.title = name;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    return c;
  }

  private chip(item: number, text: string): HTMLSpanElement {
    const el = h('span', 'chip');
    const c = h('canvas', 'chip-icon');
    c.width = c.height = ICON_PX;
    c.getContext('2d')!.drawImage(this.icon(item), 0, 0);
    el.append(c, text);
    return el;
  }
}
