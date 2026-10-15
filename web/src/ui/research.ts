// Research screen (T): the whole tech tree on a pan and zoom canvas (research-view.ts). Techs are grouped in eras
// by the science packs they cost, each a tinted band with its progress (research-eras.ts); a tech is a node in a
// column inside its era (layout: tech-tree.ts), joined to its prerequisites by curves. A node says its state by
// colour: done (green, what the rest is built on), researching (orange, with progress), available (bright and
// pulsing: the horizon) and locked (dim). It opens on the horizon; era buttons, a minimap (research-map.ts), a
// search (research-search.ts) and Fit/Where-I-am buttons get anywhere else. Zooming out trades detail for
// overview (level of detail, research.css). Hovering or focusing a node shows the full card (what it unlocks,
// what a unit costs, progress) and lights its chain of prerequisites and dependents; clicking an available tech
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
import { Eras, geometry, NODE_H, NODE_W } from './research-eras';
import { TreeMap } from './research-map';
import { makeTech, paintTech, type TechView } from './research-node';
import { ResearchQueue } from './research-queue';
import { TechSearch } from './research-search';
import { PanZoom } from './research-view';
import { layoutTree, related } from './tech-tree';

const NOTICE_MS = 6000;
const SVG_NS = 'http://www.w3.org/2000/svg';
/** Zoom of the first view: nodes at full detail with a few columns around the horizon. */
const HORIZON_ZOOM = 0.9;

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
  private readonly total = h('span', 'mp-keys');
  private readonly tracker = h('div', 'rs-tracker hidden');
  private readonly trackerText = h('span', '');
  private readonly trackerBar = h('div', 'mp-bar-fill');
  private readonly tip = h('div', 'rs-tip hidden');
  private readonly queue: ResearchQueue;
  private readonly views: TechView[];
  private readonly needs: number[][];
  private readonly edges: Edge[] = [];
  private readonly view: PanZoom;
  private readonly map: TreeMap;
  private readonly eras: Eras;
  private readonly search: TechSearch;
  private readonly at: (t: number) => { x: number; y: number };
  private done: boolean[];
  private noticeUntil = 0;
  private drawn = '';
  private centring = false;
  private hovered = -1;

  constructor(
    private readonly game: Game,
    icon: (item: number) => HTMLCanvasElement,
  ) {
    this.dialog.setAttribute('role', 'dialog');
    this.dialog.setAttribute('aria-modal', 'true');
    this.dialog.tabIndex = -1;

    this.queue = new ResearchQueue(game);
    const count = game.tech_count();
    this.needs = Array.from({ length: count }, (_, t) => Array.from(game.tech_needs(t)));
    this.views = Array.from({ length: count }, (_, t) => makeTech(game, t, icon));
    this.done = Array.from({ length: count }, (_, t) => game.tech_done(t));
    this.search = new TechSearch(this.views);

    const lay = layoutTree(this.needs, this.needs.map((_, t) => game.tech_packs(t).length));
    const geo = geometry(lay);
    this.at = geo.at;
    const stage = h('div', 'rs-stage');
    stage.style.width = `${geo.width}px`;
    stage.style.height = `${geo.height}px`;
    this.view = new PanZoom(stage, { w: geo.width, h: geo.height });
    this.eras = new Eras(game, lay, geo, stage, icon, (band) => this.view.showBand(band));
    this.buildTree(stage, geo.width, geo.height);
    this.map = new TreeMap(this.view, geo, this.eras.colour, count);
    this.view.onChange = () => this.map.moved();
    this.search.onChange = () => this.applySearch();
    this.search.onGo = (t) => this.showTech(t);
    for (const [t, v] of this.views.entries()) this.wire(t, v);

    const head = h('div', 'mp-head');
    const close = button('close-btn', '×', () => this.close(true));
    close.setAttribute('aria-label', 'Close');
    head.append(h('h2', '', 'Research'), this.total, this.eras.tabs, this.search.el, close);
    const hud = h('div', 'rs-hud rs-hud-left');
    hud.append(this.legend(), this.controls());
    const hudMap = h('div', 'rs-hud rs-hud-right');
    hudMap.append(this.map.el);
    this.view.el.append(hud, hudMap);
    const body = h('div', 'mp-body');
    body.append(this.current, this.queue.el, this.view.el);
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
    window.addEventListener('keydown', (e) => this.onKey(e));
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
      this.map.update();
    }
    // Just opened: the dialog has no size for the first moments, so place the camera once it is laid out.
    if (this.isOpen && this.centring && this.view.width > NODE_W) {
      this.centring = false;
      this.focusHorizon(false);
    }
  }

  private onKey(e: KeyboardEvent): void {
    if (!this.isOpen || e.repeat) return;
    // Typing in the search box must not close the screen (it handles Esc itself).
    if ((e.target as HTMLElement | null)?.tagName === 'INPUT') return;
    if (e.code === 'KeyT' || e.code === 'KeyE') {
      e.preventDefault();
      this.close(true);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      this.close(false);
    } else if (e.key === '/') {
      e.preventDefault();
      this.search.focus();
    }
  }

  /** Places the nodes and draws the curves between them; states are set by `draw`. */
  private buildTree(stage: HTMLElement, width: number, height: number): void {
    const svg = document.createElementNS(SVG_NS, 'svg');
    svg.setAttribute('class', 'rs-edges');
    svg.setAttribute('width', String(width));
    svg.setAttribute('height', String(height));
    this.needs.forEach((parents, to) => {
      for (const from of parents) {
        const [a, b] = [this.at(from), this.at(to)];
        const [x1, y1, x2, y2] = [a.x + NODE_W, a.y + NODE_H / 2, b.x, b.y + NODE_H / 2];
        const path = document.createElementNS(SVG_NS, 'path');
        const bend = Math.max((x2 - x1) / 2, 40);
        path.setAttribute('d', `M${x1} ${y1} C${x1 + bend} ${y1} ${x2 - bend} ${y2} ${x2} ${y2}`);
        svg.append(path);
        this.edges.push({ from, to, path });
      }
    });
    stage.append(svg);
    this.views.forEach((v, t) => {
      v.node.style.left = `${this.at(t).x}px`;
      v.node.style.top = `${this.at(t).y}px`;
      stage.append(v.node);
    });
  }

  /** A node's clicks and hover. */
  private wire(t: number, v: TechView): void {
    const g = this.game;
    v.node.addEventListener('click', (e) => {
      if (g.tech_done(t)) return;
      if (g.current_research() === t) g.set_research(-1);
      else if (this.queue.list().includes(t)) g.unqueue_research(t);
      else if (g.tech_available(t) && !e.shiftKey) g.set_research(t);
      else g.queue_research(t);
    });
    for (const on of ['pointerenter', 'focus']) v.node.addEventListener(on, () => this.showCard(t));
    for (const off of ['pointerleave', 'blur']) v.node.addEventListener(off, () => this.hideCard());
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
    const states = this.views.map((v, t) => {
      const [done, available] = [g.tech_done(t), g.tech_available(t)];
      const chosen = t === current;
      const place = queued.indexOf(t) + 1;
      paintTech(v, {
        done,
        available,
        chosen,
        place,
        progress: progress[t],
        units: g.tech_units(t),
        endless: g.tech_endless(t),
        seconds: g.tech_seconds(t),
        needs: this.needsText(t),
      });
      return done ? 'done' : chosen ? 'chosen' : available ? 'available' : 'locked';
    });
    for (const e of this.edges) {
      const [from, to] = [g.tech_done(e.from), g.tech_done(e.to)];
      e.path.setAttribute('class', from && to ? 'built' : from ? 'open' : 'closed');
    }
    this.total.textContent = `${states.filter((s) => s === 'done').length} of ${states.length} researched`;
    this.eras.update(this.done);
    this.map.update(states);
    this.applySearch();
    if (this.hovered >= 0) this.highlight(this.hovered);
  }

  /** Lights the techs the search found and dims the rest. */
  private applySearch(): void {
    const hits = this.search.matches;
    this.view.el.classList.toggle('searching', hits !== null);
    this.views.forEach((v, t) => v.node.classList.toggle('hit', hits?.has(t) ?? false));
  }

  /** Shows tech `t`'s card beside its node and lights the chain of techs it belongs to. */
  private showCard(t: number): void {
    this.hovered = t;
    this.highlight(t);
    this.tip.replaceChildren(this.views[t].card);
    this.tip.classList.remove('hidden');
    const [n, box] = [this.views[t].node.getBoundingClientRect(), this.tip.getBoundingClientRect()];
    const right = n.right + 12 + box.width <= window.innerWidth - 8;
    const x = right ? n.right + 12 : Math.max(8, n.left - 12 - box.width);
    const y = Math.min(Math.max(8, n.top - 8), Math.max(8, window.innerHeight - box.height - 8));
    this.tip.style.left = `${x}px`;
    this.tip.style.top = `${y}px`;
  }

  private highlight(t: number): void {
    const chain = related(this.needs, t);
    this.view.el.classList.add('rs-focus');
    this.views.forEach((v, i) => v.node.classList.toggle('rel', chain.has(i)));
    for (const e of this.edges) e.path.classList.toggle('hot', chain.has(e.from) && chain.has(e.to));
  }

  private hideCard(): void {
    this.hovered = -1;
    this.tip.classList.add('hidden');
    this.view.el.classList.remove('rs-focus');
    this.views.forEach((v) => v.node.classList.remove('rel'));
    for (const e of this.edges) e.path.classList.remove('hot');
  }

  /** Centres the camera on the tech being researched, else the first available one, else shows everything. */
  private focusHorizon(glide: boolean): void {
    const g = this.game;
    const t = g.current_research() >= 0 ? g.current_research() : this.views.findIndex((_, i) => g.tech_available(i));
    if (t < 0) this.view.fit();
    else this.showTech(t, glide);
  }

  /** Flies the camera to tech `t` at a zoom where its node reads in full. */
  private showTech(t: number, glide = true): void {
    const p = this.at(t);
    this.view.centreOn(p.x + NODE_W / 2, p.y + NODE_H / 2, glide, Math.max(this.view.k, HORIZON_ZOOM));
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

  /** The camera buttons and a reminder of the mouse controls. */
  private controls(): HTMLDivElement {
    const el = h('div', 'rs-controls');
    el.append(
      button('secondary-btn', 'Where I am', () => this.focusHorizon(true)),
      button('secondary-btn', 'Fit all', () => this.view.fit()),
      button('secondary-btn', '−', () => this.view.zoomBy(1 / 1.4)),
      button('secondary-btn', '+', () => this.view.zoomBy(1.4)),
      h('span', 'rs-hint', 'Drag to pan · scroll to zoom · / to search'),
    );
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
}
